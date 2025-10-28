use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::vision::det_result::DetResult;
use crate::domain::entities::vision::ocr::OcrResult;
use crate::infrastructure::factory::ocr_factory::{DetectorConfig, OcrModelFactory, RecognizerConfig};
use crate::infrastructure::entities::vision::base_traits::{TextDetector, TextRecognizer};
use image::DynamicImage;
use std::sync::Arc;

pub mod det;
pub mod rec;
pub mod base_traits;
pub mod examples;
pub mod performance;
pub mod base_model;
mod model_memory;

/// 检测器配置
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    // YOLO特有
    pub confidence_thresh: Option<f32>,
    pub iou_thresh: Option<f32>,

    // DBNet特有
    pub db_thresh: Option<f32>,
    pub db_box_thresh: Option<f32>,
    pub unclip_ratio: Option<f32>,
    pub use_dilation: Option<bool>,
}

/// 识别器配置
#[derive(Debug, Clone)]
pub struct RecognitionConfig {
    pub beam_width: Option<i32>,
    pub temperature: Option<f32>,
}

/// 新的OCR模型管理器 - 使用trait对象来处理不同的模型
#[derive(Clone)]
pub struct OcrService {
    detector: Option<Arc<dyn TextDetector + Send + Sync>>,
    recognizer: Option<Arc<dyn TextRecognizer + Send + Sync>>,
}

impl OcrService {
    /// 创建新的OCR服务实例
    pub fn new() -> Self {
        Self {
            detector: None,
            recognizer: None,
        }
    }
    
    /// 使用配置初始化检测器
    pub async fn init_detector(&mut self, config: DetectorConfig) -> AppResult<()> {
        let detector = OcrModelFactory::create_detector(config).await?;
        self.detector = Some(detector);
        Ok(())
    }
    
    /// 使用配置初始化识别器
    pub async fn init_recognizer(&mut self, config: RecognizerConfig) -> AppResult<()> {
        let recognizer = OcrModelFactory::create_recognizer(config).await?;
        self.recognizer = Some(recognizer);
        Ok(())
    }
    
    /// 执行文本检测
    pub async fn detect(&self, image: &DynamicImage) -> AppResult<Vec<DetResult>> {
        if let Some(detector) = &self.detector {
            detector.detect(image).await
        } else {
            Err(AppError::InternalError("检测器未初始化".into()))
        }
    }
    
    /// 执行文本识别
    pub async fn recognize(&self,image: &DynamicImage, det_result: &mut[DetResult]) -> AppResult<Vec<OcrResult>> {
        if let Some(recognizer) = &self.recognizer {
            recognizer.recognize(image,det_result).await
        } else {
            Err(AppError::InternalError("识别器未初始化".into()))
        }
    }

    /// 执行批量文本识别
    pub async fn recognize_batch(&self,image: &DynamicImage, det_result: &mut[DetResult]) -> AppResult<Vec<OcrResult>> {
        if let Some(recognizer) = &self.recognizer {
            recognizer.recognize_batch(image,det_result).await
        } else {
            Err(AppError::InternalError("识别器未初始化(批量)".into()))
        }
    }
    
    /// 执行完整的OCR流程 (检测 + 识别)
    pub async fn ocr(&self, image: &DynamicImage) -> AppResult<Vec<OcrResult>> {
        // 1. 首先进行文本检测
        let mut det_result = self.detect(image).await?;
        
        // 2. 对每个检测到的文本区域进行识别
        //let mut results = Vec::new();
        
        // 这里需要根据DetResult的具体结构来实现
        // 假设DetResult包含了检测到的文本框坐标
        // for text_box in det_result.boxes {
        //     let cropped_image = crop_image(image, &text_box)?;
        //     let text = self.recognize(&cropped_image).await?;
        //     results.push((text_box, text));
        // }
        
        // 暂时返回整体结果
        let ocr_results = self.recognize(image, &mut *det_result).await?;
        //results.push((det_result, text));
        
        Ok(ocr_results)
    }
    pub async fn ocr_batch(&self, image: &DynamicImage) -> AppResult<Vec<OcrResult>> {
        // 1. 首先进行文本检测
        let mut det_result = self.detect(image).await?;
        // 暂时返回整体结果
        let ocr_results = self.recognize_batch(image, &mut *det_result).await?;

        Ok(ocr_results)
    }
    
    /// 检查服务状态
    pub fn is_ready(&self) -> bool {
        self.detector.is_some() && self.recognizer.is_some()
    }
    
    /// 获取检测器信息
    pub fn detector_info(&self) -> Option<String> {
        if let Some(detector) = &self.detector {
            let config = detector.get_detection_config();
            Some(format!("检测器配置: 置信度阈值={:?}", config.confidence_thresh))
        } else {
            None
        }
    }
    
    /// 获取识别器信息
    pub fn recognizer_info(&self) -> Option<String> {
        if let Some(recognizer) = &self.recognizer {
            let config = recognizer.get_recognition_config();
            Some(format!("识别器配置: 束搜索宽度={:?}", config.beam_width))
        } else {
            None
        }
    }
}

impl Default for OcrService {
    fn default() -> Self {
        Self::new()
    }
}