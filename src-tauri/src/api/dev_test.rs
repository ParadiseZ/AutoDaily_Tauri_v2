use crate::app::dev_test::{paddle_ocr_infer, yolo_infer_test};
use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;

use crate::domain::devices::device_conf::DeviceConfig;
use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::devices::device_ctx::DeviceCtx;
use crate::infrastructure::image::load_image::dynamic_image_to_base64;
use crate::infrastructure::image::save_image::save_screenshot;
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::rec::RecognizerType;
use image::DynamicImage;
use std::sync::{Arc, RwLock};
use tauri::command;

#[command]
pub async fn dev_capture_test(
    method: u8,
    device_conf: DeviceConfig,
    adb_conf: ADBConnectConfig,
) -> Result<String, String> {
    ADBCtx::new(adb_conf).await;
    let device_ctx = DeviceCtx::new(
        Arc::new(RwLock::new(device_conf)),
        CaptureMethod::from(method),
        //Arc::new(RwLock::new(adb_ctx)),
    );

    if !device_ctx.valid_capture().await {
        return Err("验证截图功能失败！".to_string());
    }
    match device_ctx.get_screenshot().await {
        Some(image_data) => {
            Ok(dynamic_image_to_base64(&DynamicImage::ImageRgba8(image_data))?)
        }
        _ => Err("截图失败！".to_string()),
    }
}

/// 保存截图到文件
#[command]
pub async fn save_captured_image(
    image_data: &str,
    device_name: &str,
    image_type: &str,
) -> Result<String, String> {
    save_screenshot(image_data, device_name, image_type).await
}

#[command]
pub async fn yolo_inference_test(
    detector_conf: DetectorType,
    image_path: &str,
) -> Result<Vec<DetResult>, String> {
    match yolo_infer_test(image_path, detector_conf).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub async fn paddle_ocr_inference_test(
    det_model: DetectorType,
    rec_model: RecognizerType,
    image_path: &str,
) -> Result<Vec<OcrResult>, String> {
    match paddle_ocr_infer(det_model, rec_model, image_path).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}
