use crate::app::app_error::AppResult;
use crate::infra::logging::log_trait::Log;
use base64::Engine;
use base64::engine::general_purpose;
use domain_vision::DetectorType;
use domain_vision::RecognizerType;
use domain_vision::{DetResult, OcrResult};
use image::RgbaImage;
use infra_vision::{OcrService, VisionResult, get_crop_image_rgba, load_img_from_path};

fn load_img_from_base64(image_data: &str) -> Result<RgbaImage, String> {
    let base64 = image_data
        .split_once("base64,")
        .map(|(_, value)| value)
        .unwrap_or(image_data);
    let bytes = general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| format!("base64解码失败: {}", e))?;
    image::load_from_memory(&bytes)
        .map(|image| image.to_rgba8())
        .map_err(|e| format!("内存图像解析失败: {}", e))
}

fn run_ocr_rgba(ocr_service: &mut OcrService, image: &RgbaImage) -> VisionResult<Vec<OcrResult>> {
    let det_results = ocr_service.detect_rgba(image)?;
    let (cropped_images, crop_det_results): (Vec<_>, Vec<_>) = det_results
        .iter()
        .filter_map(|det_result| {
            get_crop_image_rgba(image, det_result)
                .ok()
                .map(|crop| (crop, det_result.clone()))
        })
        .unzip();

    ocr_service.recognize_crops_rgba(cropped_images, &crop_det_results)
}

fn log_ocr_results(ocr_results: &[OcrResult]) {
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
}

pub async fn yolo_infer_test(
    image_path: &str,
    detector_config: DetectorType,
) -> AppResult<Vec<DetResult>> {
    // 1. 创建OCR服务实例
    let mut ocr_service = OcrService::new();
    ocr_service.init_detector(detector_config).await?;
    let image = load_img_from_path(image_path)?.to_rgba8();
    Ok(ocr_service.detect_rgba(&image)?)
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
    let image = load_img_from_path(image_path)?.to_rgba8();
    // rec
    ocr_service.init_recognizer(recognizer_config).await?;
    let ocr_results = run_ocr_rgba(&mut ocr_service, &image)?;
    // 4. 记录检测结果
    log_ocr_results(&ocr_results);
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
    ocr_service.detect_rgba(&image).map_err(|e| e.to_string())
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
    ocr_service
        .init_recognizer(recognizer_config)
        .await
        .map_err(|e| e.to_string())?;
    let ocr_results = run_ocr_rgba(&mut ocr_service, &image).map_err(|e| e.to_string())?;
    log_ocr_results(&ocr_results);
    Ok(ocr_results)
}
