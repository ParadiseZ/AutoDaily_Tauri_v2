use crate::infra::image::crop_image::get_crop_images;
use crate::infra::vision::vision_error::VisionResult;
use domain_vision::{DetResult, OcrResult};
use image::{DynamicImage, RgbaImage};
use ndarray::{ArrayD, ArrayViewD};

/// 模型处理器的核心trait - 定义了所有模型的通用操作
pub(crate) trait ModelHandler: Send + Sync + std::fmt::Debug {
    fn load_model(&mut self) -> VisionResult<()>;

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])>;
    /// 执行模型推理
    fn inference(&self, input: ArrayViewD<f32>) -> VisionResult<ArrayD<f32>>;

    /// 获取模型输入节点名称
    fn get_input_node_name(&self) -> &'static str;

    /// 获取模型输出节点名称  
    fn get_output_node_name(&self) -> &'static str;

    fn get_target_height(&self) -> u32;
}

/// 文本检测器trait - 继承ModelHandler并添加检测特有的方法
pub(crate) trait TextDetector: ModelHandler {
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

pub(crate) trait TextRecognizer: ModelHandler {
    /// 识别文本内容
    fn recognize(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let cropped_images = get_crop_images(image, det_results)?;
        self.recognize_crops(cropped_images, det_results)
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
