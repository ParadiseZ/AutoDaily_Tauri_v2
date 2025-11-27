use crate::domain::vision::result::{BoundingBox, DetResult};
use crate::infrastructure::ort::execution_provider_mgr::InferenceBackend;
use crate::infrastructure::vision::base_model::{BaseModel, ModelType};
use crate::infrastructure::vision::base_traits::{ModelHandler, TextDetector};
use crate::infrastructure::vision::ocr_service::DetectionConfig;
use crate::infrastructure::vision::vision_error::VisionResult;
use async_trait::async_trait;
use image::{DynamicImage, GenericImageView, ImageBuffer};
use imageproc::contours::find_contours;
use imageproc::point::Point;
use memmap2::Mmap;
use ndarray::{s, Array3, Array4, Axis};

/// dbNet通常值
const MIN_AREA: f32 = 3.0;
/// 经验值
const MIN_AREA_AFTER: f32 = 5.0; //MIN_AREA+2.0

#[derive(Debug)]
pub struct PaddleDetDbNet<'a> {
    pub base_model: BaseModel<'a>,
    pub db_thresh: f32,
    pub db_box_thresh: f32,
    pub unclip_ratio: f32,
    pub use_dilation: bool,
}

impl PaddleDetDbNet {
    pub fn new(
        input_width: u32,
        input_height: u32,
        intra_thread_num: usize,
        intra_spinning: bool,
        inter_thread_num: usize,
        inter_spinning: bool,
        model_bytes_map: Mmap,
        execution_provider: InferenceBackend,
        db_thresh: f32,
        db_box_thresh: f32,
        unclip_ratio: f32,
        use_dilation: bool,
    ) -> Self {
        Self {
            base_model: BaseModel::new(
                input_width,
                input_height,
                model_bytes_map,
                execution_provider,
                intra_thread_num,
                intra_spinning,
                inter_thread_num,
                inter_spinning,
                ModelType::PaddleDet5,
            ),
            db_thresh,
            db_box_thresh,
            unclip_ratio,
            use_dilation,
        }
    }
}

#[async_trait]
impl ModelHandler for PaddleDetDbNet {
    fn load_model(&mut self) -> VisionResult<()> {
        tokio::runtime::Handle::current()
            .block_on(async {
                self.base_model
                    .load_model_base::<Self>("paddle_det_dbnet")
                    .await
            })
    }
    fn get_input_size(&self) -> (u32, u32) {
        (self.base_model.input_width, self.base_model.input_height)
    }

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(Array4<f32>, [f32; 2], [u32; 2])> {
        // 实现DBNet特有的预处理逻辑
        // 1. 图像解码
        // 2. 尺寸调整 (保持长宽比, padding)
        // 3. 归一化 (ImageNet标准化)
        // 4. 通道顺序调整 (HWC -> CHW)
        // 获取原始图像尺寸
        let (origin_w, origin_h) = (image.width(), image.height());
        let origin_shape = [origin_h, origin_w];

        // 计算调整大小比例
        let scale = self.get_target_height() as f32 / origin_h as f32;
        let width = (origin_w as f32 * scale).round() as u32;

        // DBNet要求输入宽度为32的倍数
        let target_width = if width % 32 != 0 {
            (width / 32 + 1) * 32
        } else {
            width
        };

        // 调整图像大小
        let resized_img = image.resize_exact(
            width,
            self.get_target_height(),
            image::imageops::FilterType::Triangle,
        );

        // 初始化输入数组 (使用带填充的宽度)
        let mut input =
            Array3::<f32>::zeros((3, self.get_target_height() as usize, target_width as usize));

        // 转换图像格式并归一化
        for y in 0..self.get_target_height() {
            for x in 0..width {
                let pixel = resized_img.get_pixel(x, y);
                // 标准化: (pixel / 255.0 - mean) / std
                // PaddleOCR使用的标准均值和标准差
                input[[0, y as usize, x as usize]] = (pixel[0] as f32 / 255.0 - 0.485) / 0.229;
                input[[1, y as usize, x as usize]] = (pixel[1] as f32 / 255.0 - 0.456) / 0.224;
                input[[2, y as usize, x as usize]] = (pixel[2] as f32 / 255.0 - 0.406) / 0.225;
            }
            // 对多余的宽度部分进行填充 (padding)
            // 使用归一化后的0值填充
            for x in width..target_width {
                input[[0, y as usize, x as usize]] = -0.485 / 0.229;
                input[[1, y as usize, x as usize]] = -0.456 / 0.224;
                input[[2, y as usize, x as usize]] = -0.406 / 0.225;
            }
        }

        // 扩展到批次维度 (1, C, H, W)
        let input = input.insert_axis(Axis(0));
        let scale_factor = [scale, scale]; // [h_scale, w_scale]

        Ok((input, scale_factor, origin_shape))
    }

    async fn inference(&mut self, input: Array4<f32>) -> VisionResult<Array4<f32>> {
        // 使用通用推理方法，消除代码重复
        self.base_model.inference_base(input, self).await
    }

    fn get_input_node_name(&self) -> &'static str {
        "x"
    }

    fn get_output_node_name(&self) -> &'static str {
        "fetch_name_0"
    }

    fn get_target_width(&self) -> u32 {
        self.base_model.input_width
    }

    fn get_target_height(&self) -> u32 {
        self.base_model.input_height
    }
}

#[async_trait]
impl TextDetector for PaddleDetDbNet {
    fn postprocess(
        &self,
        output: &Array4<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>> {
        // 实现DBNet后处理逻辑
        // 1. 二值化处理
        // 2. 轮廓提取
        // 3. 多边形近似
        // 4. 文本框扩展(unclip)
        let (_, _, h, w) = (
            output.shape()[0],
            output.shape()[1],
            output.shape()[2],
            output.shape()[3],
        );

        // 提取概率图
        let prob_map = output.slice(s![0, 0, .., ..]);

        // 二值化处理
        let mut binary_map = ImageBuffer::new(w as u32, h as u32);
        for y in 0..h {
            for x in 0..w {
                let val = if prob_map[[y, x]] > self.db_thresh {
                    255
                } else {
                    0
                };
                binary_map.put_pixel(x as u32, y as u32, image::Luma([val]));
            }
        }

        // 可选的膨胀操作
        if self.use_dilation {
            // 简单的膨胀操作
            let mut dilated = ImageBuffer::new(w as u32, h as u32);
            for y in 0..h as u32 {
                for x in 0..w as u32 {
                    let mut max_val = 0u8;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = (x as i32 + dx).clamp(0, w as i32 - 1) as u32;
                            let ny = (y as i32 + dy).clamp(0, h as i32 - 1) as u32;
                            max_val = max_val.max(binary_map.get_pixel(nx, ny).0[0]);
                        }
                    }
                    dilated.put_pixel(x, y, image::Luma([max_val]));
                }
            }
            binary_map = dilated;
        }

        // 查找轮廓
        let contours = find_contours::<i32>(&binary_map);
        // 处理每个轮廓
        let mut boxes: Vec<DetResult> = Vec::new();
        for contour in contours {
            // --- 步骤 1: 计算初始边界框 ---
            // [重要] 这里应该使用 `get_min_area_rect` (最小面积旋转矩形)
            // 但 imageproc 库标准版没有提供。我们暂时使用轴对齐框作为替代，
            // 但请注意这是导致检测倾斜文本失败的主要原因。
            let (min_box, min_side_len) = get_bounding_rect(&contour.points);
            if min_box.is_empty() {
                continue;
            }

            // 根据最小边长过滤
            if min_side_len < MIN_AREA {
                continue;
            }

            // --- 步骤 2: 计算得分并过滤 ---
            let score =
                box_score_fast(&prob_map.into_owned().into_dyn(), &contour.points, &min_box);

            if score < self.db_box_thresh {
                continue;
            }

            // --- 步骤 3: 扩展多边形 (unclip) ---
            let unclipped_poly = unclip_polygon(&min_box, self.unclip_ratio);
            if unclipped_poly.is_empty() {
                continue;
            }

            // --- 步骤 4: 对扩展后的多边形计算最终边界框 ---
            // 同样，这里也应该使用 `get_min_area_rect`

            // 检查扩展后的多边形是否满足最小面积要求
            let final_min_side = if unclipped_poly.len() == 4 {
                let width = (unclipped_poly[2].x - unclipped_poly[0].x) as f32;
                let height = (unclipped_poly[2].y - unclipped_poly[0].y) as f32;
                width.min(height)
            } else {
                0.0 // 不应该出现这种情况，因为输入是矩形
            };

            // 再次根据最小边长过滤
            if final_min_side < MIN_AREA_AFTER {
                continue;
            }

            // --- 步骤 5: 调整回原始图像坐标 ---
            //let scaled_points = scale_polygon(final_box, scale_factor, origin_shape);
            let scaled_points = scale_polygon(unclipped_poly, scale_factor, origin_shape);

            //boxes.push((scaled_points, score));
            boxes.push(DetResult {
                id: 0,
                pre_id: 0,
                next_id: 0,
                bounding_box: BoundingBox::new(
                    scaled_points[0].x,
                    scaled_points[0].y,
                    scaled_points[2].x,
                    scaled_points[2].y,
                ),
                index: 0,
                label: "txt".into(),
                score,
            });
        }

        // 按分数排序（可选）
        //boxes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(boxes)
    }

    fn get_detection_config(&self) -> DetectionConfig {
        DetectionConfig {
            confidence_thresh: None,
            iou_thresh: None,
            db_thresh: Some(self.db_thresh),
            db_box_thresh: Some(self.db_box_thresh),
            unclip_ratio: Some(self.unclip_ratio),
            use_dilation: Some(self.use_dilation),
        }
    }
}

// 来计算最小面积的旋转矩形，以正确处理倾斜的文本。
fn get_bounding_rect(points: &[Point<i32>]) -> (Vec<Point<i32>>, f32) {
    if points.is_empty() {
        return (Vec::new(), 0.0);
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    // 创建矩形
    let rect = vec![
        Point::new(min_x, min_y), // 左上
        Point::new(max_x, min_y), // 右上
        Point::new(max_x, max_y), // 右下
        Point::new(min_x, max_y), // 左下
    ];

    let width = (max_x - min_x) as f32;
    let height = (max_y - min_y) as f32;
    let min_side = width.min(height);

    (rect, min_side)
}

// 快速计算轮廓内概率图的平均分
fn box_score_fast(
    prob_map: &ndarray::ArrayD<f32>,
    points: &[Point<i32>],
    rect: &[Point<i32>],
) -> f32 {
    let h = prob_map.shape()[0] as i32;
    let w = prob_map.shape()[1] as i32;

    if rect.is_empty() {
        return 0.0;
    }

    let min_x = rect[0].x.max(0).min(w - 1);
    let min_y = rect[0].y.max(0).min(h - 1);
    let max_x = rect[2].x.max(0).min(w - 1);
    let max_y = rect[2].y.max(0).min(h - 1);

    let mut score_sum = 0.0;
    let mut count = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_polygon(Point::new(x, y), points) {
                // 使用 unsafe 访问以提高性能，因为我们已经确保了边界
                unsafe {
                    score_sum += *prob_map.uget([y as usize, x as usize]);
                }
                count += 1;
            }
        }
    }

    if count == 0 {
        return 0.0;
    }

    score_sum / count as f32
}

// 判断点是否在多边形内 (Ray-casting algorithm)
fn point_in_polygon(point: Point<i32>, polygon: &[Point<i32>]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        let intersect = ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x);

        if intersect {
            inside = !inside;
        }

        j = i;
    }

    inside
}

// 扩展多边形 (unclip)
// [注意] 这是一个简化的实现，它正确地计算了偏移距离，但仅适用于轴对齐的矩形。
// 一个鲁棒的实现需要一个像 Clipper 这样的多边形偏移库。
fn unclip_polygon(points: &[Point<i32>], unclip_ratio: f32) -> Vec<Point<i32>> {
    if points.len() != 4 {
        return points.to_vec(); // 只处理矩形
    }

    // 计算矩形的宽度和高度
    let width = (points[1].x - points[0].x).abs() as f32;
    let height = (points[3].y - points[0].y).abs() as f32;

    let perimeter = 2.0 * (width + height);
    if perimeter == 0.0 {
        return points.to_vec();
    }

    // 根据 DBNet 论文计算膨胀距离
    let distance = (width * height * unclip_ratio) / perimeter;

    // 扩展矩形
    let expand = distance.round() as i32;
    vec![
        Point::new(points[0].x - expand, points[0].y - expand), // 左上
        Point::new(points[1].x + expand, points[1].y - expand), // 右上
        Point::new(points[2].x + expand, points[2].y + expand), // 右下
        Point::new(points[3].x - expand, points[3].y + expand), // 左下
    ]
}

// 将多边形坐标调整回原始图像尺寸
fn scale_polygon(
    points: Vec<Point<i32>>,
    scale_factor: [f32; 2],
    origin_shape: [u32; 2],
) -> Vec<Point<i32>> {
    // let scale = self.get_target_height() as f32 / origin_h as f32;
    let [h_scale, w_scale] = scale_factor;
    points
        .into_iter()
        .map(|p| {
            let sx = ((p.x as f32 / w_scale).round() as i32).clamp(0, origin_shape[1] as i32 - 1);
            let sy = ((p.y as f32 / h_scale).round() as i32).clamp(0, origin_shape[0] as i32 - 1);
            Point::new(sx, sy)
        })
        .collect()
}
