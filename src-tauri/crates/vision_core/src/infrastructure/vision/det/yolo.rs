use crate::domain::vision::result::{BoundingBox, DetResult};
use crate::infrastructure::core::{Deserialize, HashMap, Serialize};
use crate::infrastructure::vision::base_model::{BaseModel, ModelType};
use crate::infrastructure::vision::base_traits::{ModelHandler, TextDetector};
use crate::infrastructure::vision::tensor_view::squeeze_singleton_axes_to_2d;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use ndarray::{Array4, ArrayD, ArrayView2, ArrayView4, ArrayViewD, Axis};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tokio::fs::read_to_string;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct YoloDet {
    pub base_model: BaseModel,
    pub class_count: usize,
    #[serde(skip)]
    #[ts(skip)]
    pub class_labels: HashMap<u16, String>,
    pub confidence_thresh: f32,
    pub iou_thresh: f32,
    #[ts(as = "Option<String>")]
    pub label_path: Option<PathBuf>,
    pub txt_idx: Option<u16>,
    #[serde(skip, default)]
    #[ts(skip)]
    postprocess_kind: YoloPostprocessKind,
    #[serde(skip, default)]
    #[ts(skip)]
    output_layout: OnceLock<YoloOutputLayout>,
    #[serde(skip, default = "YoloDet::default_preprocess_buffer")]
    #[ts(skip)]
    preprocess_buffer: Mutex<Vec<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YoloPostprocessKind {
    LegacyNms,
    EndToEnd,
}

impl Default for YoloPostprocessKind {
    fn default() -> Self {
        Self::LegacyNms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YoloOutputLayout {
    LegacyCandidatesRows,
    LegacyCandidatesCols,
    EndToEndRows,
    EndToEndCols,
}

#[derive(Debug, Clone)]
struct YoloCandidate {
    coords: [f32; 4],
    box_format: YoloBoxFormat,
    class_id: usize,
    score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YoloBoxFormat {
    Xywh,
    Xyxy,
}

const MAX_NMS_CANDIDATES_PER_CLASS: usize = 512;
const INV_255: f32 = 1.0 / 255.0;

impl YoloDet {
    /// 创建 YOLO 检测器内部使用的预处理缓存。
    fn default_preprocess_buffer() -> Mutex<Vec<f32>> {
        Mutex::new(Vec::new())
    }

    /// 根据模型类型初始化一次运行期策略，避免每次推理都判断后处理分支。
    pub fn refresh_runtime_config(&mut self) {
        self.postprocess_kind = match self.base_model.model_type {
            ModelType::Yolo26 => YoloPostprocessKind::EndToEnd,
            _ => YoloPostprocessKind::LegacyNms,
        };
        self.output_layout = OnceLock::new();
    }

    /// 加载 YOLO 标签文件；未提供标签时保持为空并回退为 `class_x` 标签。
    pub async fn load_labels(&mut self) -> VisionResult<()> {
        let Some(label_path) = self.label_path.clone() else {
            self.class_labels.clear();
            return Ok(());
        };

        let content = read_to_string(&label_path)
            .await
            .map_err(|e| VisionError::IoError {
                path: label_path.to_string_lossy().to_string(),
                e: e.to_string(),
            })?;

        let values: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|_e| VisionError::IoError {
                path: label_path.to_string_lossy().to_string(),
                e: "反序列化Yolo标签文件失败".to_string(),
            })?;
        match values.get("names") {
            Some(val) => {
                self.class_labels =
                    serde_yaml::from_value(val.clone()).map_err(|e| VisionError::IoError {
                        path: label_path.to_string_lossy().to_string(),
                        e: format!("解析标签names失败: {}", e),
                    })?;
            }
            None => {
                return Err(VisionError::IoError {
                    path: label_path.to_string_lossy().to_string(),
                    e: "Yolo标签读取names属性值失败！".to_string(),
                })
            }
        }
        Ok(())
    }

    fn model_file_stem(&self) -> &'static str {
        match self.base_model.model_type {
            ModelType::Yolo26 => "det_yolo26",
            _ => "det_yolo",
        }
    }

    /// 去掉输出张量中的单维度，并统一为二维矩阵视图供后处理消费。
    fn squeeze_output<'a>(&self, output: ArrayViewD<'a, f32>) -> VisionResult<ArrayView2<'a, f32>> {
        squeeze_singleton_axes_to_2d(output, "yolo_squeeze_output")
    }

    fn detect_output_layout(&self, matrix: ArrayView2<'_, f32>) -> YoloOutputLayout {
        let shape = matrix.shape();
        let rows = shape[0];
        let cols = shape[1];

        match self.postprocess_kind {
            YoloPostprocessKind::LegacyNms => {
                let expected_attr_count = self.class_count + 4;
                let expected_with_objectness = self.class_count + 5;
                if cols == expected_attr_count || cols == expected_with_objectness {
                    YoloOutputLayout::LegacyCandidatesRows
                } else if rows == expected_attr_count || rows == expected_with_objectness {
                    YoloOutputLayout::LegacyCandidatesCols
                } else if rows > cols {
                    YoloOutputLayout::LegacyCandidatesRows
                } else {
                    YoloOutputLayout::LegacyCandidatesCols
                }
            }
            YoloPostprocessKind::EndToEnd => {
                if cols == 6 {
                    YoloOutputLayout::EndToEndRows
                } else if rows == 6 {
                    YoloOutputLayout::EndToEndCols
                } else if rows >= cols {
                    YoloOutputLayout::EndToEndRows
                } else {
                    YoloOutputLayout::EndToEndCols
                }
            }
        }
    }

    fn resolve_output_layout(&self, matrix: ArrayView2<'_, f32>) -> YoloOutputLayout {
        *self
            .output_layout
            .get_or_init(|| self.detect_output_layout(matrix))
    }

    fn resolve_label(&self, class_id: usize) -> String {
        self.class_labels
            .get(&(class_id as u16))
            .cloned()
            .unwrap_or_else(|| format!("class_{}", class_id))
    }

    /// 将 RGB 像素连续写入 CHW 缓冲区，避免 4D 下标逐点赋值的额外开销。
    fn fill_chw_buffer(raw_pixels: &[u8], input_buffer: &mut [f32], plane_len: usize) {
        let (r_plane, rest) = input_buffer.split_at_mut(plane_len);
        let (g_plane, b_plane) = rest.split_at_mut(plane_len);

        for (pixel_index, pixel) in raw_pixels.chunks_exact(3).enumerate() {
            r_plane[pixel_index] = pixel[0] as f32 * INV_255;
            g_plane[pixel_index] = pixel[1] as f32 * INV_255;
            b_plane[pixel_index] = pixel[2] as f32 * INV_255;
        }
    }

    /// 将 NMS 之后的轻量候选框转成最终业务结果，并在这里完成坐标恢复与标签解析。
    fn finalize_candidates(
        &self,
        candidates: Vec<YoloCandidate>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> Vec<DetResult> {
        candidates
            .into_iter()
            .map(|candidate| {
                DetResult::new(
                    self.candidate_to_bounding_box(&candidate, scale_factor, origin_shape),
                    candidate.class_id as i32,
                    self.resolve_label(candidate.class_id),
                    candidate.score,
                    8,
                )
            })
            .collect()
    }

    /// 在模型坐标系下读取候选框的 XYXY 形式，供 NMS 计算 IoU 使用。
    fn candidate_xyxy(candidate: &YoloCandidate) -> [f32; 4] {
        match candidate.box_format {
            YoloBoxFormat::Xywh => {
                let [xc, yc, w, h] = candidate.coords;
                [xc - w / 2.0, yc - h / 2.0, xc + w / 2.0, yc + h / 2.0]
            }
            YoloBoxFormat::Xyxy => candidate.coords,
        }
    }

    /// 将模型坐标中的候选框转换回原图坐标，并做边界裁剪。
    fn candidate_to_bounding_box(
        &self,
        candidate: &YoloCandidate,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> BoundingBox {
        match candidate.box_format {
            YoloBoxFormat::Xywh => self.build_xywh_box(
                candidate.coords[0],
                candidate.coords[1],
                candidate.coords[2],
                candidate.coords[3],
                scale_factor,
                origin_shape,
            ),
            YoloBoxFormat::Xyxy => self.build_xyxy_box(
                candidate.coords[0],
                candidate.coords[1],
                candidate.coords[2],
                candidate.coords[3],
                scale_factor,
                origin_shape,
            ),
        }
    }

    fn allow_class(&self, class_id: usize) -> bool {
        self.txt_idx
            .map(|idx| idx as usize == class_id)
            .unwrap_or(true)
    }

    fn clamp_x(&self, value: f32, origin_shape: [u32; 2]) -> i32 {
        value.clamp(0.0, origin_shape[1] as f32).round() as i32
    }

    fn clamp_y(&self, value: f32, origin_shape: [u32; 2]) -> i32 {
        value.clamp(0.0, origin_shape[0] as f32).round() as i32
    }

    fn build_xywh_box(
        &self,
        xc: f32,
        yc: f32,
        w: f32,
        h: f32,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> BoundingBox {
        let xc = xc * scale_factor[0];
        let yc = yc * scale_factor[1];
        let w = w * scale_factor[0];
        let h = h * scale_factor[1];

        BoundingBox::new(
            self.clamp_x(xc - w / 2.0, origin_shape),
            self.clamp_y(yc - h / 2.0, origin_shape),
            self.clamp_x(xc + w / 2.0, origin_shape),
            self.clamp_y(yc + h / 2.0, origin_shape),
        )
    }

    fn build_xyxy_box(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> BoundingBox {
        BoundingBox::new(
            self.clamp_x(x1 * scale_factor[0], origin_shape),
            self.clamp_y(y1 * scale_factor[1], origin_shape),
            self.clamp_x(x2 * scale_factor[0], origin_shape),
            self.clamp_y(y2 * scale_factor[1], origin_shape),
        )
    }

    fn legacy_score_and_class<F>(&self, len: usize, mut value_at: F) -> Option<(usize, f32)>
    where
        F: FnMut(usize) -> f32,
    {
        if len <= 4 {
            return None;
        }

        let (class_offset, objectness) = if len >= self.class_count + 5 {
            (5, value_at(4))
        } else {
            (4, 1.0)
        };

        if class_offset >= len {
            return None;
        }

        if let Some(target_class_id) = self.txt_idx.map(|idx| idx as usize) {
            if target_class_id >= self.class_count {
                return None;
            }

            let target_index = class_offset + target_class_id;
            if target_index >= len {
                return None;
            }

            return Some((target_class_id, objectness * value_at(target_index)));
        }

        let mut class_id = 0usize;
        let mut class_prob = f32::MIN;
        for idx in class_offset..len {
            let score = value_at(idx);
            if score > class_prob {
                class_prob = score;
                class_id = idx - class_offset;
            }
        }

        Some((class_id, objectness * class_prob))
    }

    /// 解析 Yolo11/Yolo8 这类 raw head 输出。
    ///
    /// 执行顺序：
    /// 1. 规整输出布局到 2D
    /// 2. 逐候选框计算置信度
    /// 3. 收集轻量候选框
    /// 4. 本地 NMS
    /// 5. 对保留下来的候选框恢复原图坐标并生成 `DetResult`
    fn postprocess_legacy(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>> {
        // Yolo11/8 这条链路走 raw head: 先筛候选框，再做本地 NMS，最后再生成完整 DetResult。
        let matrix = self.squeeze_output(output)?;
        let layout = self.resolve_output_layout(matrix);
        let candidate_capacity = match layout {
            YoloOutputLayout::LegacyCandidatesRows => matrix.nrows(),
            YoloOutputLayout::LegacyCandidatesCols => matrix.ncols(),
            _ => 0,
        };
        let mut candidates = Vec::with_capacity(candidate_capacity);

        match layout {
            YoloOutputLayout::LegacyCandidatesRows => {
                for row in matrix.axis_iter(Axis(0)) {
                    let Some((class_id, prob)) =
                        self.legacy_score_and_class(row.len(), |idx| row[idx])
                    else {
                        continue;
                    };

                    if prob < self.confidence_thresh
                        || class_id >= self.class_count
                        || !self.allow_class(class_id)
                    {
                        continue;
                    }

                    candidates.push(YoloCandidate {
                        coords: [row[0], row[1], row[2], row[3]],
                        box_format: YoloBoxFormat::Xywh,
                        class_id,
                        score: prob,
                    });
                }
            }
            YoloOutputLayout::LegacyCandidatesCols => {
                for col in matrix.axis_iter(Axis(1)) {
                    let Some((class_id, prob)) =
                        self.legacy_score_and_class(col.len(), |idx| col[idx])
                    else {
                        continue;
                    };

                    if prob < self.confidence_thresh
                        || class_id >= self.class_count
                        || !self.allow_class(class_id)
                    {
                        continue;
                    }

                    candidates.push(YoloCandidate {
                        coords: [col[0], col[1], col[2], col[3]],
                        box_format: YoloBoxFormat::Xywh,
                        class_id,
                        score: prob,
                    });
                }
            }
            _ => {
                return Err(VisionError::DataProcessingErr {
                    method: "yolo_postprocess_legacy".to_string(),
                    e: "YOLO raw 输出布局与后处理策略不匹配".to_string(),
                });
            }
        }

        Ok(self.finalize_candidates(
            apply_nms(candidates, self.iou_thresh),
            scale_factor,
            origin_shape,
        ))
    }

    /// 解析 Yolo26 这类 end-to-end 输出。
    ///
    /// 这条链路假设模型已经完成 NMS，因此这里只做阈值过滤和坐标恢复。
    fn postprocess_end_to_end(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>> {
        let matrix = self.squeeze_output(output)?;
        let layout = self.resolve_output_layout(matrix);
        let candidate_capacity = match layout {
            YoloOutputLayout::EndToEndRows => matrix.nrows(),
            YoloOutputLayout::EndToEndCols => matrix.ncols(),
            _ => 0,
        };
        let mut candidates = Vec::with_capacity(candidate_capacity);

        match layout {
            YoloOutputLayout::EndToEndRows => {
                for row in matrix.axis_iter(Axis(0)) {
                    if row.len() < 6 {
                        continue;
                    }

                    let class_id = row[5].round().max(0.0) as usize;
                    let score = row[4];

                    if score < self.confidence_thresh
                        || class_id >= self.class_count
                        || !self.allow_class(class_id)
                    {
                        continue;
                    }

                    candidates.push(YoloCandidate {
                        coords: [row[0], row[1], row[2], row[3]],
                        box_format: YoloBoxFormat::Xyxy,
                        class_id,
                        score,
                    });
                }
            }
            YoloOutputLayout::EndToEndCols => {
                for col in matrix.axis_iter(Axis(1)) {
                    if col.len() < 6 {
                        continue;
                    }

                    let class_id = col[5].round().max(0.0) as usize;
                    let score = col[4];

                    if score < self.confidence_thresh
                        || class_id >= self.class_count
                        || !self.allow_class(class_id)
                    {
                        continue;
                    }

                    candidates.push(YoloCandidate {
                        coords: [col[0], col[1], col[2], col[3]],
                        box_format: YoloBoxFormat::Xyxy,
                        class_id,
                        score,
                    });
                }
            }
            _ => {
                return Err(VisionError::DataProcessingErr {
                    method: "yolo_postprocess_end_to_end".to_string(),
                    e: "YOLO end-to-end 输出布局与后处理策略不匹配".to_string(),
                });
            }
        }

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(self.finalize_candidates(candidates, scale_factor, origin_shape))
    }
}

impl ModelHandler for YoloDet {
    fn load_model(&mut self) -> VisionResult<()> {
        self.refresh_runtime_config();
        self.base_model
            .load_model_base::<Self>(self.model_file_stem())
    }

    fn get_input_size(&self) -> (u32, u32) {
        (self.base_model.input_width, self.base_model.input_height)
    }

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])> {
        // 兼容通用调用路径，仍然返回拥有所有权的输入张量。
        let (w, h) = self.get_input_size();
        let (origin_w, origin_h) = image.dimensions();
        let scale_x = origin_w as f32 / w as f32;
        let scale_y = origin_h as f32 / h as f32;

        let img = image.resize_exact(w, h, FilterType::Triangle);
        let (width, height) = img.dimensions();
        let img_buffer = img.to_rgb8();
        let raw_pixels = img_buffer.into_raw();
        let width_usize = width as usize;
        let height_usize = height as usize;
        let mut input = Array4::<f32>::zeros((1, 3, height_usize, width_usize));
        let plane_len = width_usize * height_usize;

        if let Some(buffer) = input.as_slice_mut() {
            Self::fill_chw_buffer(&raw_pixels, buffer, plane_len);
        } else {
            return Err(VisionError::DataProcessingErr {
                method: "yolo_preprocess".to_string(),
                e: "输入张量不是连续内存".to_string(),
            });
        }

        Ok((input.into_dyn(), [scale_x, scale_y], [origin_h, origin_w]))
    }

    fn inference(&self, input: ArrayViewD<f32>) -> VisionResult<ArrayD<f32>> {
        // 使用通用推理方法，消除代码重复
        self.base_model.inference_base(
            input,
            self.get_input_node_name(),
            self.get_output_node_name(),
        )
    }

    fn get_input_node_name(&self) -> &'static str {
        "images"
    }

    fn get_output_node_name(&self) -> &'static str {
        "output0"
    }

    fn get_target_width(&self) -> u32 {
        self.base_model.input_width
    }

    fn get_target_height(&self) -> u32 {
        self.base_model.input_height
    }
}

impl TextDetector for YoloDet {
    /// 执行 YOLO 检测主链路。
    ///
    /// 这条实现会复用预处理缓冲区，并直接消费 ORT 输出 view，
    /// 避免通用 `inference_base` 的输出整块复制。
    fn detect(&self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        // 检测主链路：预处理 -> ORT 推理 -> 直接消费输出 view 做后处理，避免整块输出复制。
        let (w, h) = self.get_input_size();
        let (origin_w, origin_h) = image.dimensions();
        let scale_factor = [origin_w as f32 / w as f32, origin_h as f32 / h as f32];
        let origin_shape = [origin_h, origin_w];

        let img = image.resize_exact(w, h, FilterType::Triangle);
        let img_buffer = img.to_rgb8();
        let raw_pixels = img_buffer.as_raw();
        let width_usize = w as usize;
        let height_usize = h as usize;
        let plane_len = width_usize * height_usize;
        let input_len = plane_len * 3;

        let mut input_buffer =
            self.preprocess_buffer
                .lock()
                .map_err(|_| VisionError::DataProcessingErr {
                    method: "yolo_detect".to_string(),
                    e: "获取YOLO预处理缓存失败".to_string(),
                })?;
        if input_buffer.len() != input_len {
            input_buffer.resize(input_len, 0.0);
        }
        Self::fill_chw_buffer(raw_pixels, input_buffer.as_mut_slice(), plane_len);
        let input_view =
            ArrayView4::from_shape((1, 3, height_usize, width_usize), input_buffer.as_slice())
                .map_err(|e| VisionError::DataProcessingErr {
                    method: "yolo_detect".to_string(),
                    e: e.to_string(),
                })?;

        self.base_model.inference_with_output_view(
            input_view.into_dyn(),
            self.get_input_node_name(),
            self.get_output_node_name(),
            |output| self.postprocess(output, scale_factor, origin_shape),
        )
    }

    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>> {
        match self.postprocess_kind {
            YoloPostprocessKind::LegacyNms => {
                self.postprocess_legacy(output, scale_factor, origin_shape)
            }
            YoloPostprocessKind::EndToEnd => {
                self.postprocess_end_to_end(output, scale_factor, origin_shape)
            }
        }
    }
}

fn intersection(box1: &YoloCandidate, box2: &YoloCandidate) -> f32 {
    let [x1_a, y1_a, x2_a, y2_a] = YoloDet::candidate_xyxy(box1);
    let [x1_b, y1_b, x2_b, y2_b] = YoloDet::candidate_xyxy(box2);

    (x2_a.min(x2_b) - x1_a.max(x1_b)).max(0.0) * (y2_a.min(y2_b) - y1_a.max(y1_b)).max(0.0)
}

fn union(box1: &YoloCandidate, box2: &YoloCandidate) -> f32 {
    let [x1_a, y1_a, x2_a, y2_a] = YoloDet::candidate_xyxy(box1);
    let [x1_b, y1_b, x2_b, y2_b] = YoloDet::candidate_xyxy(box2);

    ((x2_a - x1_a).max(0.0) * (y2_a - y1_a).max(0.0))
        + ((x2_b - x1_b).max(0.0) * (y2_b - y1_b).max(0.0))
        - intersection(box1, box2)
}

/// 计算两个模型坐标候选框的 IoU。
fn calculate_iou(box1: &YoloCandidate, box2: &YoloCandidate) -> f32 {
    let intersection_area = intersection(box1, box2);
    let union_area = union(box1, box2);

    if union_area <= 0.0 {
        return 0.0;
    }

    intersection_area / union_area
}

/// 对候选框执行按类别分段的本地 NMS。
///
/// 进入 NMS 前会先按分数排序，并截断每类尾部低分候选框，
/// 避免大量低质量框拖慢 O(n^2) 比较。
fn apply_nms(mut boxes: Vec<YoloCandidate>, iou_thresh: f32) -> Vec<YoloCandidate> {
    if boxes.is_empty() {
        return boxes;
    }

    boxes.sort_by(|a, b| {
        a.class_id.cmp(&b.class_id).then_with(|| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let mut final_results = Vec::with_capacity(boxes.len());
    let mut start = 0;
    while start < boxes.len() {
        let class_id = boxes[start].class_id;
        let mut end = start + 1;
        while end < boxes.len() && boxes[end].class_id == class_id {
            end += 1;
        }

        // 这里先截断低分尾部，避免 O(n^2) NMS 被海量低置信度候选框拖慢。
        let class_end = (start + MAX_NMS_CANDIDATES_PER_CLASS).min(end);
        let class_boxes = &boxes[start..class_end];
        let mut keep = Vec::new();
        let mut suppress = vec![false; class_boxes.len()];

        for i in 0..class_boxes.len() {
            if suppress[i] {
                continue;
            }

            keep.push(class_boxes[i].clone());

            // 抑制与当前框 IoU 过高的其他框
            for j in (i + 1)..class_boxes.len() {
                if suppress[j] {
                    continue;
                }

                let iou = calculate_iou(&class_boxes[i], &class_boxes[j]);
                if iou > iou_thresh {
                    suppress[j] = true;
                }
            }
        }

        final_results.extend(keep);
        start = end;
    }

    final_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::ort::execution_provider_mgr::InferenceBackend;
    use crate::infrastructure::vision::base_model::ModelSource;
    use ndarray::arr2;

    fn build_detector(model_type: ModelType, txt_idx: Option<u16>) -> YoloDet {
        let mut class_labels = HashMap::new();
        class_labels.insert(0, "button".to_string());
        class_labels.insert(1, "text".to_string());

        YoloDet {
            base_model: BaseModel::new(
                640,
                640,
                ModelSource::Custom,
                PathBuf::new(),
                InferenceBackend::CPU,
                1,
                false,
                1,
                false,
                model_type,
            ),
            class_count: 2,
            class_labels,
            confidence_thresh: 0.25,
            iou_thresh: 0.45,
            label_path: None,
            txt_idx,
            postprocess_kind: match model_type {
                ModelType::Yolo26 => YoloPostprocessKind::EndToEnd,
                _ => YoloPostprocessKind::LegacyNms,
            },
            output_layout: OnceLock::new(),
            preprocess_buffer: YoloDet::default_preprocess_buffer(),
        }
    }

    #[test]
    fn parses_yolo11_raw_head_output() {
        let detector = build_detector(ModelType::Yolo11, None);
        let output = arr2(&[
            [320.0, 120.0],
            [300.0, 100.0],
            [100.0, 80.0],
            [60.0, 40.0],
            [0.90, 0.10],
            [0.05, 0.85],
        ])
        .into_dyn();

        let mut results = detector
            .postprocess(output.view(), [1.0, 1.0], [640, 640])
            .unwrap();
        results.sort_by_key(|item| item.index);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].index, 0);
        assert_eq!(results[0].label, "button");
        assert_eq!(results[1].index, 1);
        assert_eq!(results[1].label, "text");
    }

    #[test]
    fn parses_yolo26_end_to_end_output_without_nms() {
        let detector = build_detector(ModelType::Yolo26, None);
        let output = arr2(&[
            [10.0, 20.0, 110.0, 120.0, 0.80, 1.0],
            [20.0, 30.0, 40.0, 50.0, 0.10, 0.0],
        ])
        .into_dyn();

        let results = detector
            .postprocess(output.view(), [1.0, 1.0], [640, 640])
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].index, 1);
        assert_eq!(results[0].label, "text");
        assert_eq!(results[0].bounding_box, BoundingBox::new(10, 20, 110, 120));
    }

    #[test]
    fn filters_by_txt_idx_for_text_detection() {
        let detector = build_detector(ModelType::Yolo26, Some(1));
        let output = arr2(&[
            [10.0, 20.0, 110.0, 120.0, 0.80, 0.0],
            [12.0, 22.0, 112.0, 122.0, 0.75, 1.0],
        ])
        .into_dyn();

        let results = detector
            .postprocess(output.view(), [1.0, 1.0], [640, 640])
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].index, 1);
        assert_eq!(results[0].label, "text");
    }
}
