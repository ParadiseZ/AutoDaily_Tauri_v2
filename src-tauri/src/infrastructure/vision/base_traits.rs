use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::image::crop_image::{get_crop_image, get_crop_images};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::ocr_service::DetectionConfig;
use crate::infrastructure::vision::vision_error::VisionResult;
use async_trait::async_trait;
use image::DynamicImage;
use ndarray::Array4;
use rayon::prelude::IntoParallelRefIterator;

/// 模型处理器的核心trait - 定义了所有模型的通用操作
#[async_trait]
pub trait ModelHandler: Send + Sync {
    fn load_model(&mut self);

    /// 获取模型输入尺寸
    fn get_input_size(&self) -> (u32, u32);

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(Array4<f32>, [f32; 2], [u32; 2])>;
    /// 执行模型推理
    async fn inference(&self, input: Array4<f32>) -> VisionResult<Array4<f32>>;

    /// 获取模型输入节点名称
    fn get_input_node_name(&self) -> &'static str;

    /// 获取模型输出节点名称  
    fn get_output_node_name(&self) -> &'static str;

    fn get_target_width(&self) -> u32;
    fn get_target_height(&self) -> u32;
}

/// 文本检测器trait - 继承ModelHandler并添加检测特有的方法
#[async_trait]
pub trait TextDetector: ModelHandler {
    /// 检测文本区域
    async fn detect(&self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        // 通用的检测流程
        let (preprocessed, scale_factor, origin_shape) = self.preprocess(image)?;
        let raw_output = self.inference(preprocessed).await?;
        let det_res = self.postprocess(&raw_output, scale_factor, origin_shape)?;
        //self.parse_detection_result(processed_output)
        Ok(det_res)
    }
    /// 后处理推理结果
    fn postprocess(
        &self,
        output: &Array4<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>>;

    /// 获取检测特有的配置参数
    fn get_detection_config(&self) -> DetectionConfig;
}

/// 文本识别器trait - 继承ModelHandler并添加识别特有的方法  
#[async_trait]
pub trait TextRecognizer: ModelHandler {
    /// 识别文本内容
    async fn recognize(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        // 预处理
        let rgba_img = &image.to_rgba8();
        let inputs: Vec<Array4<f32>> = det_results
            .par_iter()
            .map(|&det_res| {
                let img = get_crop_image(rgba_img, det_res)?;
                let (input, _, _) = self.preprocess(&img)?;
                input
            })
            .collect();
        // 推理
        let outputs: Vec<Array4<f32>> = inputs
            .into_iter()
            .map(|input| async {
                self.inference(input).await?;
            })
            .collect();

        // 后处理
        let ocr_res: Vec<OcrResult> = outputs
            .par_iter()
            .filter_map(|output| {
                self.postprocess(output, det_results, 0).ok()
            })
            .collect();
        if ocr_res.len() != det_results.len() {
            Log::error("识别部分行的文字错误！");
        }
        /*let imgs = self.get_rotate_crop_image(image, det_results)?;
        let mut ocr_res = Vec::with_capacity(imgs.len());
        let _ = imgs.par_iter()
            .map(|img| async {
                let (input, _, _) = self.preprocess(img)?;
                let raw_output = self.inference(input).await?;
                let processed_output = self.postprocess(raw_output, &det_results[idx])?;
                Ok(processed_output)
            });
        Log::warn("rayon 线程池获取锁失败！将使用单线程处理");
        for (idx , img) in imgs.par_iter_mut().enumerate() {
            let (input, _, _) = self.preprocess(&img)?;
            let raw_output = self.inference(input).await?;
            let processed_output = self.postprocess(raw_output,&det_results[idx])?;
            ocr_res.push(processed_output);
        }*/
        Ok(ocr_res)
        //self.parse_recognition_result(processed_output)
    }
    async fn recognize_batch(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let imgs = get_crop_images(image, det_results)?;
        let input = self.preprocess_batch(&imgs)?;
        let raw_output = self.inference(input).await?;
        let ocr_res = self.postprocess_batch(&raw_output, det_results)?;
        Ok(ocr_res)
        //self.parse_recognition_result(processed_output)
    }

    /// 后处理推理结果
    fn postprocess(
        &self,
        output: &Array4<f32>,
        det_result: &DetResult,
        batch_size: usize,
    ) -> VisionResult<OcrResult>;

    /// 批量处理
    fn preprocess_batch(&self, images: &[DynamicImage]) -> VisionResult<Array4<f32>>;

    fn postprocess_batch(
        &self,
        output: &Array4<f32>,
        det_result: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>>;
}
