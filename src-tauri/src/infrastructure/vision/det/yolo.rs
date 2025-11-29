use crate::domain::vision::result::{BoundingBox, DetResult};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::execution_provider_mgr::InferenceBackend;
use crate::infrastructure::vision::base_model::{BaseModel, ModelType};
use crate::infrastructure::vision::base_traits::{ModelHandler, TextDetector};
use crate::infrastructure::vision::ocr_service::DetectionConfig;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use memmap2::Mmap;
use ndarray::{s, Array, ArrayD, ArrayViewD, Axis};

#[derive(Debug)]
pub struct YoloDet<'a> {
    pub base_model: BaseModel<'a>,
    pub class_count: usize,
    pub class_labels: Vec<String>,
    pub confidence_thresh: f32,
    pub iou_thresh: f32,
}

impl YoloDet<'_> {
    pub fn new(
        input_width: u32,
        input_height: u32,
        intra_thread_num: usize,
        intra_spinning: bool,
        inter_thread_num: usize,
        inter_spinning: bool,
        model_bytes_map: Mmap,
        execution_provider: InferenceBackend,
        class_count: usize,
        class_labels: Vec<String>,
        confidence_thresh: f32,
        iou_thresh: f32,
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
                ModelType::Yolo11,
            ),
            class_count,
            class_labels,
            confidence_thresh,
            iou_thresh,
        }
    }
}

impl ModelHandler for YoloDet<'_> {
    fn load_model(&mut self) -> VisionResult<()> {
        self.base_model.load_model_base::<Self>("det_yolo")
    }

    fn get_input_size(&self) -> (u32, u32) {
        (self.base_model.input_width, self.base_model.input_height)
    }

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])> {
        // 实现YOLO特有的预处理逻辑
        // 1. 图像解码
        // 2. 尺寸调整为模型输入尺寸
        // 3. 归一化 (0-255 -> 0-1)
        // 4. 通道顺序调整 (HWC -> CHW)
        let (w, h) = self.get_input_size();
        let (origin_w, origin_h) = image.dimensions();
        let scale = origin_w.max(origin_h) as f32 / w as f32;

        let img = image.resize(w, h, FilterType::Triangle);
        let (width, height) = img.dimensions();
        let img_buffer = img.to_rgb8();
        let raw_pixels = img_buffer.into_raw();

        let img_array = Array::from_shape_vec((height as usize, width as usize, 3), raw_pixels)
            .map_err(|e| VisionError::DataProcessingErr {
                method: "preprocess".to_string(),
                e: e.to_string(),
            })?;

        // (H, W, C) -> (C, H, W)
        let img_array = img_array.permuted_axes([2, 0, 1]);

        // Normalize and add batch dimension
        let input = img_array.mapv(|x| x as f32 / 255.0).insert_axis(Axis(0));

        Ok((input.into_dyn(), [scale, scale], [origin_h, origin_w]))
    }

    fn inference(&mut self, input: ArrayViewD<f32>) -> VisionResult<ArrayD<f32>> {
        // 使用通用推理方法，消除代码重复
        self.base_model.inference_base(input, self.get_input_node_name(), self.get_output_node_name())
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

impl TextDetector for YoloDet<'_> {
    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        _origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>> {
        // 实现YOLO后处理逻辑
        // 1. NMS (非极大值抑制)
        // 2. 置信度过滤
        // 3. 坐标转换

        let mut boxes: Vec<DetResult> = Vec::new();
        let output = output.slice(s![0, .., .., 0]);
        // let scale = origin_w.max(origin_h) as f32 / target_width as f32;
        let scale = scale_factor[0];
        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, prob) = row
                .iter()
                .skip(4) // 跳过边界框坐标
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap_or((0, 0.0));

            if prob < self.confidence_thresh {
                continue;
            }

            // 确保类别ID在有效范围内
            if class_id >= self.class_count {
                Log::warn(&format!("[ yolo ]无效类别ID: {}", class_id));
                continue;
            }
            //let (w,h) = self.get_input_size();
            let label: &String = &self.class_labels[class_id];


            let xc = row[0] * scale;
            let yc = row[1] * scale;
            let w = row[2] * scale;
            let h = row[3] * scale;

            boxes.push(DetResult {
                id: 0,
                pre_id: 0,
                next_id: 0,
                bounding_box: BoundingBox::new(
                    (xc - w / 2.) as i32,
                    (yc - h / 2.) as i32,
                    (xc + w / 2.) as i32,
                    (yc + h / 2.) as i32,
                ),
                index: class_id as i32,
                label: label.clone(),
                score: prob,
            });
        }

        // 应用非极大值抑制(NMS)
        // 过滤掉IoU高于阈值的框

        // 应用非极大值抑制(NMS)
        let result = apply_nms(boxes, self.iou_thresh)?;

        Ok(result)
    }

    fn get_detection_config(&self) -> DetectionConfig {
        DetectionConfig {
            confidence_thresh: Some(self.confidence_thresh),
            iou_thresh: Some(self.iou_thresh),
            db_thresh: None,
            db_box_thresh: None,
            unclip_ratio: None,
            use_dilation: None,
        }
    }
}

// 计算IoU的辅助函数
fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> i32 {
    (box1.x2.min(box2.x2) - box1.x1.max(box2.x1)).max(0)
        * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1)).max(0)
}

fn union(box1: &BoundingBox, box2: &BoundingBox) -> i32 {
    ((box1.x2 - box1.x1) * (box1.y2 - box1.y1)) + ((box2.x2 - box2.x1) * (box2.y2 - box2.y1))
        - intersection(box1, box2)
}

/// 计算两个边界框的 IoU (Intersection over Union)
fn calculate_iou(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    let intersection_area = intersection(box1, box2) as f32;
    let union_area = union(box1, box2) as f32;

    if union_area <= 0.0 {
        return 0.0;
    }

    intersection_area / union_area
}

fn apply_nms(boxes: Vec<DetResult>, iou_thresh: f32) -> VisionResult<Vec<DetResult>> {
    if boxes.is_empty() {
        return Ok(boxes);
    }

    // 按类别分组
    use std::collections::HashMap;
    let mut boxes_by_class: HashMap<i32, Vec<DetResult>> = HashMap::new();

    for detection in boxes {
        boxes_by_class
            .entry(detection.index)
            .or_insert_with(Vec::new)
            .push(detection);
    }

    let mut final_results = Vec::new();

    // 对每个类别分别进行 NMS
    for (_class_id, mut class_boxes) in boxes_by_class {
        // 按置信度降序排序
        class_boxes.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

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

                let iou = calculate_iou(&class_boxes[i].bounding_box, &class_boxes[j].bounding_box);
                if iou > iou_thresh {
                    suppress[j] = true;
                }
            }
        }

        final_results.extend(keep);
    }

    Ok(final_results)
}
