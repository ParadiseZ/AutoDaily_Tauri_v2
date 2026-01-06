use crate::app::app_error::AppResult;
use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::image::load_image::load_img_from_path;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::ocr_factory::{DetectorType, RecognizerType};
use crate::infrastructure::vision::ocr_service::OcrService;

pub async fn yolo_infer_test(
    image_path: &str,
    detector_config: DetectorType,
) -> AppResult<Vec<DetResult>> {
    // 1. 创建OCR服务实例
    let mut ocr_service = OcrService::new();
    ocr_service.init_detector(detector_config).await?;
    let image = load_img_from_path(image_path)?;
    Ok(ocr_service.detect(&image)?)
}

pub async fn paddle_ocr_infer(
    detector_config: DetectorType,
    recognizer_config: RecognizerType,
    image_path: &str,
) -> AppResult<Vec<OcrResult>> {
    // 1. 创建OCR服务实例
    let mut ocr_service = OcrService::new();
    // det
    ocr_service.init_detector(detector_config).await?;
    let image = load_img_from_path(image_path)?;
    let mut det_results = ocr_service.detect(&image)?;
    // rec
    ocr_service.init_recognizer(recognizer_config).await?;
    //let image = load_img_from_path(image_path)?;
    let ocr_results = ocr_service.recognize(&image, &mut det_results)?;
    // 4. 记录检测结果
    for (i, ocr) in ocr_results.iter().enumerate(){
        Log::info(
            format!(
            "#{}: 文本='{}' (分数={:?}, 位置=[{:.1}, {:.1}, {:.1}, {:.1}]",
            i + 1,
            ocr.txt,
            ocr.score,
            ocr.bounding_box.x1,
            ocr.bounding_box.y1,
            ocr.bounding_box.x2,
            ocr.bounding_box.y2,
        ).as_str());
    }
    Ok(ocr_results)
}
