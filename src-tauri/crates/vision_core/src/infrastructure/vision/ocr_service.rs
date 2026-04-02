use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::base_model::ModelType;
use crate::infrastructure::vision::base_traits::{ModelHandler, TextDetector, TextRecognizer};
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::rec::RecognizerType;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::DynamicImage;
use std::sync::Arc;

/// 新的OCR模型管理器 - 使用trait对象来处理不同的模型
#[derive(Debug, Clone)]
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
    pub async fn init_detector(&mut self, config: DetectorType) -> VisionResult<()> {
        Log::info("初始化检测模型...");
        let detector: Arc<dyn TextDetector + Send + Sync> = match config {
            DetectorType::Yolo11(mut yolo) => {
                yolo.base_model.model_type = ModelType::Yolo11;
                yolo.refresh_runtime_config();
                //加载标签
                Log::debug("加载yolo标签文件...");
                yolo.load_labels().await?;
                Log::debug("加载yolo检测模型...");
                yolo.load_model()?;
                Arc::new(yolo)
            }
            DetectorType::Yolo26(mut yolo) => {
                yolo.base_model.model_type = ModelType::Yolo26;
                yolo.refresh_runtime_config();
                //加载标签
                Log::debug("加载yolo26标签文件...");
                yolo.load_labels().await?;
                Log::debug("加载yolo26检测模型...");
                yolo.load_model()?;
                Arc::new(yolo)
            }
            DetectorType::PaddleDbNet(mut db_net) => {
                Log::info("加载DBNet检测模型...");
                db_net.load_model()?;
                Arc::new(db_net)
            }
        };
        self.detector = Some(detector);
        Ok(())
    }

    /// 使用配置初始化识别器。
    ///
    /// 初始化顺序固定为：加载字典 -> 加载 ONNX 模型 -> 缓存 trait 对象实例。
    pub async fn init_recognizer(&mut self, config: RecognizerType) -> VisionResult<()> {
        Log::info("初始化文字识别模型...");
        let recognizer: Arc<dyn TextRecognizer + Send + Sync> = match config {
            RecognizerType::PaddleCrnn(mut crnn) => {
                // 加载字典
                Log::debug("加载字典文件...");
                crnn.load_dict().await?;
                Log::debug("加载CRNN识别模型...");
                crnn.load_model()?;
                Arc::new(crnn)
            }
        };
        self.recognizer = Some(recognizer);
        Ok(())
    }

    /// 设置已有的检测器实例
    pub fn set_detector_instance(&mut self, detector: Arc<dyn TextDetector + Send + Sync>) {
        self.detector = Some(detector);
    }

    /// 设置已有的识别器实例
    pub fn set_recognizer_instance(&mut self, recognizer: Arc<dyn TextRecognizer + Send + Sync>) {
        self.recognizer = Some(recognizer);
    }

    /// 只执行文本检测，不做识别。
    pub fn detect(&mut self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        if let Some(ref mut detector) = self.detector {
            Ok(detector.detect(image)?)
        } else {
            Err(VisionError::DetectorNotInit)
        }
    }

    /// 对给定检测框执行逐框识别。
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

    /// 对给定检测框执行批量识别。
    ///
    /// 是否真正走 micro-batch 由具体识别器配置决定。
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

    /// 执行完整 OCR 流程：检测后走逐框识别链路。
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
    /// 执行完整 OCR 流程：检测后走批量识别链路。
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::image::load_image::load_img_from_path;
    use serde::Deserialize;
    use std::time::Instant;

    const OCR_TEST_CONFIG_PATH_ENV: &str = "AUTODAILY_OCR_TEST_CONFIG_PATH";

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RealOcrTestConfig {
        detector: DetectorType,
        recognizer: RecognizerType,
        image_path: String,
        print_res: bool,
    }

    fn required_env(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| panic!("missing required env var: {}", name))
    }

    fn real_ocr_test_config() -> RealOcrTestConfig {
        let config_path = required_env(OCR_TEST_CONFIG_PATH_ENV);
        let config_text = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|e| panic!("failed to read test config '{}': {}", config_path, e));

        serde_json::from_str(&config_text)
            .unwrap_or_else(|e| panic!("failed to parse test config '{}': {}", config_path, e))
    }

    #[tokio::test]
    #[ignore = "requires AUTODAILY_OCR_TEST_CONFIG_PATH pointing to a real-path JSON config"]
    async fn real_paths_detector_init_and_detect() {
        let config = real_ocr_test_config();

        let mut service = OcrService::new();

        service
            .init_detector(config.detector)
            .await
            .expect("detector should initialize with real model path");

        let image = load_img_from_path(&config.image_path).expect("failed to load test image");
        let detect_start = Instant::now();
        let det_results = service
            .detect(&image)
            .expect("detect should run with real paths");
        let detect_elapsed = detect_start.elapsed();

        println!("detect elapsed: {:.3?}", detect_elapsed);
        println!("detector result count: {}", det_results.len());
        for (idx, item) in det_results.iter().enumerate() {
            println!(
                "#{idx}: label='{}' class={} score={:.4} box=({}, {}, {}, {})",
                item.label,
                item.index,
                item.score,
                item.bounding_box.x1,
                item.bounding_box.y1,
                item.bounding_box.x2,
                item.bounding_box.y2
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires AUTODAILY_OCR_TEST_CONFIG_PATH pointing to a real-path JSON config"]
    async fn real_paths_ocr_init_and_recognize() {
        let config = real_ocr_test_config();
        let config_print = real_ocr_test_config();

        let mut service = OcrService::new();

        service
            .init_detector(config.detector)
            .await
            .expect("detector should initialize with real model path");
        service
            .init_recognizer(config.recognizer)
            .await
            .expect("recognizer should initialize with real model path");

        let image = load_img_from_path(&config.image_path).expect("failed to load test image");
        let detect_start = Instant::now();
        let mut det_results = service
            .detect(&image)
            .expect("detect should run with real paths");
        let detect_elapsed = detect_start.elapsed();

        let recognize_batch_start = Instant::now();
        let ocr_results_batch = service
            .recognize_batch(&image, &mut det_results)
            .expect("recognize_batch should run with real paths");
        let recognize_batch_elapsed = recognize_batch_start.elapsed();

        let recognize_start = Instant::now();
        let ocr_results = service
            .recognize(&image, &mut det_results)
            .expect("recognize_batch should run with real paths");
        let recognize_elapsed = recognize_start.elapsed();
        let total_elapsed = detect_elapsed + recognize_elapsed;

        let det_conf = match config_print.detector {
            DetectorType::Yolo11(ref cfg) => (
                cfg.base_model.intra_thread_num,
                cfg.base_model.intra_spinning,
                cfg.base_model.inter_thread_num,
                cfg.base_model.inter_spinning,
                cfg.base_model.execution_provider.name(),
                cfg.get_target_width(),
                cfg.get_target_height(),
            ),
            DetectorType::PaddleDbNet(ref cfg) => (
                cfg.base_model.intra_thread_num,
                cfg.base_model.intra_spinning,
                cfg.base_model.inter_thread_num,
                cfg.base_model.inter_spinning,
                cfg.base_model.execution_provider.name(),
                cfg.get_target_width(),
                cfg.get_target_height(),
            ),
            _ => {
                panic!("unsupported detector type")
            }
        };
        let rec_conf = match config_print.recognizer {
            RecognizerType::PaddleCrnn(ref cfg) => (
                cfg.base_model.intra_thread_num,
                cfg.base_model.intra_spinning,
                cfg.base_model.inter_thread_num,
                cfg.base_model.inter_spinning,
                cfg.base_model.execution_provider.name(),
                cfg.get_target_width(),
                cfg.get_target_height(),
            ),
        };

        println!("det: intraThreadNum: {}, intraSpinning:{}, interThreadNum: {}, interSpinning:{}, provider: {}, inputWidth: {}, inputHeight: {}",
                 det_conf.0,
                 det_conf.1,
                 det_conf.2,
                 det_conf.3,
                 det_conf.4,
                 det_conf.5,
                 det_conf.6
        );
        println!("rec: intraThreadNum: {}, intraSpinning:{}, interThreadNum: {}, interSpinning:{}, provider: {}, inputWidth: {}, inputHeight: {}",
                 rec_conf.0,
                 rec_conf.1,
                 rec_conf.2,
                 rec_conf.3,
                 rec_conf.4,
                 rec_conf.5,
                 rec_conf.6
        );
        println!("service: {},  detect:{:.3?},  rec_batch:{:.3?},  rec:{:.3?},  ocr:{:.3?},  det_count:{},  ocr_count:{}",
                 service.is_ready(),
                 detect_elapsed,
                 recognize_batch_elapsed,
                 recognize_elapsed,
                 total_elapsed,
                 det_results.len(),
                 ocr_results.len()
        );
        if config.print_res {
            for (idx, item) in det_results.iter().enumerate() {
                println!("#{idx},{}", item);
            }
            println!("================批处理===============");
            for (idx, item) in ocr_results_batch.iter().enumerate() {
                println!(
                    "#{idx}: txt='{}' chars={:?} score={:?} box=({}, {}, {}, {})",
                    item.txt,
                    item.txt_char,
                    item.score,
                    item.bounding_box.x1,
                    item.bounding_box.y1,
                    item.bounding_box.x2,
                    item.bounding_box.y2
                )
            }
            println!("================单处理===============");
            for (idx, item) in ocr_results.iter().enumerate() {
                println!(
                    "#{idx}: txt='{}' chars={:?} score={:?} box=({}, {}, {}, {})",
                    item.txt,
                    item.txt_char,
                    item.score,
                    item.bounding_box.x1,
                    item.bounding_box.y1,
                    item.bounding_box.x2,
                    item.bounding_box.y2
                );
            }
        }
    }
}
