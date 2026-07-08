use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::image::crop_image::{get_crop_images, get_crop_images_rgba};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::vision_error::VisionResult;
use image::{DynamicImage, RgbaImage};
use ndarray::{ArrayD, ArrayViewD};
use rayon::prelude::*;

/// 模型处理器的核心trait - 定义了所有模型的通用操作
pub trait ModelHandler: Send + Sync + std::fmt::Debug {
    fn load_model(&mut self) -> VisionResult<()>;

    /// 获取模型输入尺寸
    fn get_input_size(&self) -> (u32, u32);

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])>;
    /// 执行模型推理
    fn inference(&self, input: ArrayViewD<f32>) -> VisionResult<ArrayD<f32>>;

    /// 获取模型输入节点名称
    fn get_input_node_name(&self) -> &'static str;

    /// 获取模型输出节点名称  
    fn get_output_node_name(&self) -> &'static str;

    fn get_target_width(&self) -> u32;
    fn get_target_height(&self) -> u32;
}

/// 文本检测器trait - 继承ModelHandler并添加检测特有的方法
pub trait TextDetector: ModelHandler {
    /// 检测文本区域
    fn detect(&self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        // 通用的检测流程
        let (preprocessed, scale_factor, origin_shape) = self.preprocess(image)?;
        let raw_output = self.inference(preprocessed.view())?;
        let det_res = self.postprocess(raw_output.view(), scale_factor, origin_shape)?;
        //self.parse_detection_result(processed_output)
        Ok(det_res)
    }

    fn detect_rgba(&self, image: &RgbaImage) -> VisionResult<Vec<DetResult>> {
        self.detect(&DynamicImage::ImageRgba8(image.clone()))
    }
    /// 后处理推理结果
    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>>;

    // 获取检测特有的配置参数
}

pub trait TextRecognizer: ModelHandler {
    fn recognize_preprocessed_inputs(
        &self,
        preprocessed_inputs: Vec<(usize, ArrayD<f32>)>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        if preprocessed_inputs.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-预处理：部分图像预处理失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    preprocessed_inputs.len()
                )
                .as_str(),
            );
        }

        let mut inference_outputs: Vec<(usize, ArrayD<f32>)> =
            Vec::with_capacity(preprocessed_inputs.len());

        for (idx, input) in preprocessed_inputs {
            match self.inference(input.view()) {
                Ok(output) => inference_outputs.push((idx, output)),
                Err(e) => {
                    Log::warn(format!("文字识别-推理：第 {} 项推理失败: {:?}", idx, e).as_str());
                }
            }
        }

        if inference_outputs.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-推理：部分图像推理失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    inference_outputs.len()
                )
                .as_str(),
            );
        }

        let mut ocr_res: Vec<(usize, OcrResult)> = inference_outputs
            .par_iter()
            .filter_map(|(idx, output)| {
                if let Some(det_res) = det_results.get(*idx) {
                    self.postprocess(output.view(), det_res, 0)
                        .ok()
                        .map(|ocr| (*idx, ocr))
                } else {
                    Log::warn(format!("文字识别-后处理：索引 {} 越界", *idx).as_str());
                    None
                }
            })
            .collect();

        if ocr_res.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-后处理：部分结果处理失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    ocr_res.len()
                )
                .as_str(),
            );
        }

        ocr_res.sort_by_key(|(idx, _)| *idx);
        Ok(ocr_res.into_iter().map(|(_, item)| item).collect())
    }

    /// 识别文本内容
    fn recognize(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let cropped_images = get_crop_images(image, det_results)?;
        self.recognize_crops(cropped_images, det_results)
    }

    fn recognize_rgba(
        &self,
        image: &RgbaImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let cropped_images = get_crop_images_rgba(image, det_results)?;
        self.recognize_crops_rgba(cropped_images, det_results)
    }

    fn recognize_crops(
        &self,
        cropped_images: Vec<DynamicImage>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>>;

    fn recognize_crops_rgba(
        &self,
        cropped_images: Vec<RgbaImage>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        self.recognize_crops(
            cropped_images
                .into_iter()
                .map(DynamicImage::ImageRgba8)
                .collect(),
            det_results,
        )
    }

    /// 后处理推理结果
    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        det_result: &DetResult,
        batch_size: usize,
    ) -> VisionResult<OcrResult>;
}
