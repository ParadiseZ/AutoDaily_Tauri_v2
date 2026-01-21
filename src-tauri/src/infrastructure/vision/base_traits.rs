use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::image::crop_image::{get_crop_image, get_crop_images};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::vision_error::VisionResult;
use image::DynamicImage;
use ndarray::{ArrayD, ArrayViewD};
use rayon::prelude::*;

/// 模型处理器的核心trait - 定义了所有模型的通用操作
pub trait ModelHandler: Send + Sync {
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
    /// 后处理推理结果
    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>>;

    // 获取检测特有的配置参数
}

/// 文本识别器trait - 继承ModelHandler并添加识别特有的方法  
pub trait TextRecognizer: ModelHandler {
    /// 识别文本内容
    fn recognize(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        // 1. 预处理阶段
        // 保留原始索引：(original_index, preprocessed_input)
        //let rgba_img = &image.to_rgba8();//移入预处理阶段
        let preprocessed_inputs: Vec<(usize, ArrayD<f32>)> = det_results
            .par_iter()
            .enumerate()
            .filter_map(|(idx, det_res)| {
                get_crop_image(image, det_res)
                    .ok()
                    .and_then(|img| {
                        self.preprocess(&img).ok()
                    })
                    .and_then(|input| Some((idx, input.0))) // input.0 is Array4<f32> based on preprocess signature
            })
            .collect();

        if preprocessed_inputs.len() != det_results.len() {
            Log::warn(format!(
                "文字识别-预处理：部分图像预处理失败！(总数: {}, 成功: {})",
                det_results.len(),
                preprocessed_inputs.len()
            )
            .as_str());
        }

        // 2. 推理阶段
        // 依次执行推理，保留原始索引：(original_index, inference_output)
        // 注意：这里为了配合ort调度，我们使用串行await，或者可以使用futures::stream::FuturesOrdered如果需要并发
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
            Log::warn(format!(
                "文字识别-推理：部分图像推理失败！(总数: {}, 成功: {})",
                det_results.len(),
                inference_outputs.len()
            )
            .as_str());
        }

        // 3. 后处理阶段
        // 使用原始索引找回对应的 det_result
        let ocr_res: Vec<OcrResult> = inference_outputs
            .par_iter()
            .filter_map(|(idx, output)| {
                // 使用原始索引获取对应的检测结果
                if let Some(det_res) = det_results.get(*idx) {
                    self.postprocess(output.view(), det_res, 0).ok()
                } else {
                    Log::warn(format!("文字识别-后处理：索引 {} 越界", *idx).as_str());
                    None
                }
            })
            .collect();

        if ocr_res.len() != det_results.len() {
            Log::warn(format!(
                "文字识别-后处理：部分结果处理失败！(总数: {}, 成功: {})",
                det_results.len(),
                ocr_res.len()
            )
            .as_str());
        }

        Ok(ocr_res)
    }
    fn recognize_batch(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let imgs = get_crop_images(image, det_results)?;
        let input = self.preprocess_batch(&imgs)?;
        let raw_output = self.inference(input.view())?;
        let ocr_res = self.postprocess_batch(raw_output.view(), det_results)?;
        Ok(ocr_res)
        //self.parse_recognition_result(processed_output)
    }

    /// 后处理推理结果
    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        det_result: &DetResult,
        batch_size: usize,
    ) -> VisionResult<OcrResult>;

    /// 批量处理
    fn preprocess_batch(&self, images: &[DynamicImage]) -> VisionResult<ArrayD<f32>>;

    fn postprocess_batch(
        &self,
        output: ArrayViewD<f32>,
        det_result: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>>;
}
