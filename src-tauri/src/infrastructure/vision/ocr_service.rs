use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::vision::base_traits::{TextDetector, TextRecognizer};
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::rec::RecognizerType;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::DynamicImage;
use std::sync::Arc;
use crate::infrastructure::logging::log_trait::Log;

/// 新的OCR模型管理器 - 使用trait对象来处理不同的模型
#[derive(Clone)]
pub struct OcrService {
    detector: Option<Arc<dyn TextDetector>>,
    recognizer: Option<Arc<dyn TextRecognizer>>,
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
    pub async fn init_detector(&mut self, config: DetectorType) -> VisionResult<()> {
        Log::info("初始化检测模型...");
        let detector : Arc<dyn TextDetector> = match config {
            DetectorType::Yolo11(mut yolo) => {
                //加载标签
                Log::info("加载yolo标签文件...");
                yolo.load_labels().await?;
                Arc::new(yolo)
            }
            DetectorType::PaddleDbNet(db_net) => {
                Arc::new(db_net)
            }
        };
        self.detector = Some(detector);
        Ok(())
    }

    /// 使用配置初始化识别器
    pub async fn init_recognizer(&mut self, config: RecognizerType) -> VisionResult<()> {
        Log::info("初始化文字识别模型...");
        let recognizer : Arc<dyn TextRecognizer> = match config {
            RecognizerType::PaddleCrnn(mut crnn) => {
                // 加载字典
                Log::info("加载字典文件...");
                crnn.load_dict().await?;
                Arc::new(crnn)
            }
        };
        self.recognizer = Some(recognizer);
        Ok(())
    }

    /// 设置已有的检测器实例
    pub fn set_detector_instance(&mut self, detector: Arc<dyn TextDetector>) {
        self.detector = Some(detector);
    }

    /// 设置已有的识别器实例
    pub fn set_recognizer_instance(&mut self, recognizer: Arc<dyn TextRecognizer>) {
        self.recognizer = Some(recognizer);
    }

    /// 执行文本检测
    pub fn detect(&mut self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        if let Some(ref mut detector) = self.detector {
            Ok(detector.detect(image)?)
        } else {
            Err(VisionError::DetectorNotInit)
        }
    }

    /// 执行文本识别
    pub fn recognize(
        &mut self,
        image: &DynamicImage,
        det_result: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        if let Some(ref mut recognizer) = self.recognizer {
            Ok(recognizer.recognize(image, det_result)?)
        } else {
            Err(VisionError::RecognizeNotInit)
        }
    }

    /// 执行批量文本识别
    pub fn recognize_batch(
        &mut self,
        image: &DynamicImage,
        det_result: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        if let Some(ref mut recognizer) = self.recognizer {
            Ok(recognizer.recognize_batch(image, det_result)?)
        } else {
            Err(VisionError::RecognizeNotInit)
        }
    }

    /// 执行完整的OCR流程 (检测 + 识别)
    pub fn ocr(&mut self, image: &DynamicImage) -> VisionResult<Vec<OcrResult>> {
        // 1. 首先进行文本检测
        let mut det_result = self.detect(image)?;
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
        let ocr_results = self.recognize(image, &mut *det_result)?;
        //results.push((det_result, text));

        Ok(ocr_results)
    }
    pub fn ocr_batch(&mut self, image: &DynamicImage) -> VisionResult<Vec<OcrResult>> {
        // 1. 首先进行文本检测
        let mut det_result = self.detect(image)?;
        // 暂时返回整体结果
        let ocr_results = self.recognize_batch(image, &mut *det_result)?;

        Ok(ocr_results)
    }

    /// 检查服务状态
    pub fn is_ready(&self) -> bool {
        self.detector.is_some() && self.recognizer.is_some()
    }

    /*pub fn detector_info(&self) -> Option<String> {
        if let Some(detector) = &self.detector {
            let config = detector.get_detection_config();
            Some(format!(
                "检测器配置: 置信度阈值={:?}",
                config.confidence_thresh
            ))
        } else {
            None
        }
    }*/


    /*pub fn recognizer_info(&self) -> Option<String> {
        if let Some(recognizer) = &self.recognizer {
            let config = recognizer.get_recognition_config();
            Some(format!("识别器配置: 束搜索宽度={:?}", config.beam_width))
        } else {
            None
        }
    }*/
}

impl Default for OcrService {
    fn default() -> Self {
        Self::new()
    }
}
