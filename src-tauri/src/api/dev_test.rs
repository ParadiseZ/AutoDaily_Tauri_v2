use crate::app::dev_test::{
    paddle_ocr_base64_infer, paddle_ocr_infer, yolo_infer_base64_test, yolo_infer_test,
};
use crate::constant::sys_conf_path::{APP_STORE, SCRIPTS_CONFIG_KEY};
use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;

use crate::domain::devices::device_conf::{CapMethod, DeviceConfig};
use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::devices::device_ctx::DeviceCtx;
use crate::infrastructure::image::load_image::dynamic_image_to_base64;
use crate::infrastructure::image::save_image::save_screenshot;
use crate::infrastructure::store_local::config_store::get_or_init_config;
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::rec::RecognizerType;
use image::DynamicImage;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{command, AppHandle, Manager};
use tauri_plugin_store::StoreExt;
use vision_core::infrastructure::vision::base_model::ModelSource;
use tokio::sync::RwLock;

fn resolve_scripts_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .store(APP_STORE)
        .map(|store| get_or_init_config::<ScriptsConfig>(store, SCRIPTS_CONFIG_KEY).dir)
        .unwrap_or_else(|_| {
            app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("scripts")
        })
}

fn resolve_custom_model_path(path: &Path, scripts_dir: &Path) -> PathBuf {
    if path.as_os_str().is_empty() || path.is_absolute() {
        path.to_path_buf()
    } else {
        scripts_dir.join(path)
    }
}

fn resolve_optional_relative_path(path: &mut Option<PathBuf>, scripts_dir: &Path) {
    if let Some(current) = path.as_mut() {
        if !current.is_absolute() {
            *current = scripts_dir.join(current.clone());
        }
    }
}

fn normalize_detector_conf(app_handle: &AppHandle, detector_conf: DetectorType) -> DetectorType {
    let scripts_dir = resolve_scripts_dir(app_handle);
    match detector_conf {
        DetectorType::Yolo11(mut yolo) => {
            if yolo.base_model.model_source == ModelSource::Custom {
                yolo.base_model.model_path =
                    resolve_custom_model_path(&yolo.base_model.model_path, &scripts_dir);
            }
            resolve_optional_relative_path(&mut yolo.label_path, &scripts_dir);
            DetectorType::Yolo11(yolo)
        }
        DetectorType::Yolo26(mut yolo) => {
            if yolo.base_model.model_source == ModelSource::Custom {
                yolo.base_model.model_path =
                    resolve_custom_model_path(&yolo.base_model.model_path, &scripts_dir);
            }
            resolve_optional_relative_path(&mut yolo.label_path, &scripts_dir);
            DetectorType::Yolo26(yolo)
        }
        DetectorType::PaddleDbNet(mut dbnet) => {
            if dbnet.base_model.model_source == ModelSource::Custom {
                dbnet.base_model.model_path =
                    resolve_custom_model_path(&dbnet.base_model.model_path, &scripts_dir);
            }
            DetectorType::PaddleDbNet(dbnet)
        }
    }
}

fn normalize_recognizer_conf(
    app_handle: &AppHandle,
    recognizer_conf: RecognizerType,
) -> RecognizerType {
    let scripts_dir = resolve_scripts_dir(app_handle);
    match recognizer_conf {
        RecognizerType::PaddleCrnn(mut crnn) => {
            if crnn.base_model.model_source == ModelSource::Custom {
                crnn.base_model.model_path =
                    resolve_custom_model_path(&crnn.base_model.model_path, &scripts_dir);
            }
            resolve_optional_relative_path(&mut crnn.dict_path, &scripts_dir);
            RecognizerType::PaddleCrnn(crnn)
        }
    }
}

#[command]
pub async fn dev_capture_test(
    method: u8,
    device_conf: DeviceConfig,
    adb_conf: ADBConnectConfig,
) -> Result<String, String> {
    ADBCtx::new(adb_conf).await;
    let title = match device_conf.cap_method.clone() {
        CapMethod::Window(title) => Some(title),
        CapMethod::Adb => None,
    };
    let device_ctx = DeviceCtx::new(
        Arc::new(RwLock::new(device_conf)),
        CaptureMethod::from(method),
        title, //Arc::new(RwLock::new(adb_ctx)),
    );

    if !device_ctx.valid_capture().await {
        return Err("验证截图功能失败！".to_string());
    }
    match device_ctx.get_screenshot().await {
        Some(image_data) => Ok(dynamic_image_to_base64(&DynamicImage::ImageRgba8(
            image_data,
        ))?),
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
    app_handle: AppHandle,
    detector_conf: DetectorType,
    image_path: &str,
) -> Result<Vec<DetResult>, String> {
    let detector_conf = normalize_detector_conf(&app_handle, detector_conf);
    match yolo_infer_test(image_path, detector_conf).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub async fn yolo_inference_image_data_test(
    app_handle: AppHandle,
    detector_conf: DetectorType,
    image_data: &str,
) -> Result<Vec<DetResult>, String> {
    let detector_conf = normalize_detector_conf(&app_handle, detector_conf);
    yolo_infer_base64_test(image_data, detector_conf).await
}

#[command]
pub async fn paddle_ocr_inference_test(
    app_handle: AppHandle,
    det_model: DetectorType,
    rec_model: RecognizerType,
    image_path: &str,
) -> Result<Vec<OcrResult>, String> {
    let det_model = normalize_detector_conf(&app_handle, det_model);
    let rec_model = normalize_recognizer_conf(&app_handle, rec_model);
    match paddle_ocr_infer(det_model, rec_model, image_path).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[command]
pub async fn paddle_ocr_inference_image_data_test(
    app_handle: AppHandle,
    det_model: DetectorType,
    rec_model: RecognizerType,
    image_data: &str,
) -> Result<Vec<OcrResult>, String> {
    let det_model = normalize_detector_conf(&app_handle, det_model);
    let rec_model = normalize_recognizer_conf(&app_handle, rec_model);
    paddle_ocr_base64_infer(det_model, rec_model, image_data).await
}
