use crate::app::app_error::AppResult;
use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::image::load_image::load_img_from_path;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::ocr_service::OcrService;
use crate::infrastructure::vision::rec::RecognizerType;
use base64::engine::general_purpose;
use base64::Engine;
use image::DynamicImage;

fn load_img_from_base64(image_data: &str) -> Result<DynamicImage, String> {
    let base64 = image_data
        .split_once("base64,")
        .map(|(_, value)| value)
        .unwrap_or(image_data);
    let bytes = general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| format!("base64解码失败: {}", e))?;
    image::load_from_memory(&bytes).map_err(|e| format!("内存图像解析失败: {}", e))
}

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
    for (i, ocr) in ocr_results.iter().enumerate() {
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
            )
            .as_str(),
        );
    }
    Ok(ocr_results)
}

pub async fn yolo_infer_base64_test(
    image_data: &str,
    detector_config: DetectorType,
) -> Result<Vec<DetResult>, String> {
    let mut ocr_service = OcrService::new();
    ocr_service
        .init_detector(detector_config)
        .await
        .map_err(|e| e.to_string())?;
    let image = load_img_from_base64(image_data)?;
    ocr_service.detect(&image).map_err(|e| e.to_string())
}

pub async fn paddle_ocr_base64_infer(
    detector_config: DetectorType,
    recognizer_config: RecognizerType,
    image_data: &str,
) -> Result<Vec<OcrResult>, String> {
    let mut ocr_service = OcrService::new();
    ocr_service
        .init_detector(detector_config)
        .await
        .map_err(|e| e.to_string())?;
    let image = load_img_from_base64(image_data)?;
    let mut det_results = ocr_service.detect(&image).map_err(|e| e.to_string())?;
    ocr_service
        .init_recognizer(recognizer_config)
        .await
        .map_err(|e| e.to_string())?;
    let ocr_results = ocr_service
        .recognize(&image, &mut det_results)
        .map_err(|e| e.to_string())?;
    for (i, ocr) in ocr_results.iter().enumerate() {
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
            )
            .as_str(),
        );
    }
    Ok(ocr_results)
}
