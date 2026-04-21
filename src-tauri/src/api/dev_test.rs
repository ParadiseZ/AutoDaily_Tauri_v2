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

fn describe_adb_config_issue(adb_conf: &ADBConnectConfig) -> Option<String> {
    match adb_conf {
        ADBConnectConfig::DirectTcp(addr) => {
            if addr.is_none() {
                Some("未设置直连地址".to_string())
            } else {
                None
            }
        }
        ADBConnectConfig::DirectUsb(_) => Some("当前暂不支持 DirectUsb 截图".to_string()),
        ADBConnectConfig::ServerConnectByIp(config) => {
            if config.adb_config.adb_path.as_deref().is_none_or(|value| value.trim().is_empty()) {
                return Some("未设置 adb 程序路径".to_string());
            }
            if !config.adb_config.valid() {
                return Some("ADB 服务配置无效，请检查 adb 路径和服务地址".to_string());
            }
            if config.client_connect.is_none() {
                return Some("未设置设备连接地址".to_string());
            }
            None
        }
        ADBConnectConfig::ServerConnectByName(config) => {
            if config.adb_config.adb_path.as_deref().is_none_or(|value| value.trim().is_empty()) {
                return Some("未设置 adb 程序路径".to_string());
            }
            if !config.adb_config.valid() {
                return Some("ADB 服务配置无效，请检查 adb 路径和服务地址".to_string());
            }
            if config
                .device_name
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Some("未设置设备名称".to_string());
            }
            None
        }
    }
}

fn validate_capture_request(
    capture_method: &CaptureMethod,
    device_conf: &DeviceConfig,
    adb_conf: &ADBConnectConfig,
) -> Result<(), String> {
    match capture_method {
        CaptureMethod::Window => match &device_conf.cap_method {
            CapMethod::Window(title) => {
                let title = title.trim();
                if title.is_empty() {
                    Err("窗口截图未配置窗口标题".to_string())
                } else {
                    Ok(())
                }
            }
            CapMethod::Adb => Err("当前设备保存的截图方式是 ADB，但本次请求走了窗口截图".to_string()),
        },
        CaptureMethod::Adb => describe_adb_config_issue(adb_conf)
            .map(|issue| format!("ADB 截图配置无效：{}", issue))
            .map_or(Ok(()), Err),
    }
}

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
    let capture_method = CaptureMethod::from(method);
    validate_capture_request(&capture_method, &device_conf, &adb_conf)?;
    let capture_method_for_error = capture_method.clone();
    let device_conf_for_error = device_conf.clone();
    let adb_conf_for_error = adb_conf.clone();
    if matches!(capture_method, CaptureMethod::Adb) {
        ADBCtx::new(adb_conf).await;
    }
    let title = match device_conf.cap_method.clone() {
        CapMethod::Window(title) => Some(title),
        CapMethod::Adb => None,
    };
    let device_ctx = DeviceCtx::new(
        Arc::new(RwLock::new(device_conf)),
        capture_method,
        title, //Arc::new(RwLock::new(adb_ctx)),
    )
    .await;

    if !device_ctx.valid_capture().await {
        let reason = match capture_method_for_error {
            CaptureMethod::Window => match &device_conf_for_error.cap_method {
                CapMethod::Window(title) => format!(
                    "窗口截图校验失败：未找到标题包含“{}”的可截图窗口，或目标窗口已最小化",
                    title.trim()
                ),
                CapMethod::Adb => "窗口截图校验失败：当前设备截图方式配置不一致".to_string(),
            },
            CaptureMethod::Adb => match describe_adb_config_issue(&adb_conf_for_error) {
                Some(issue) => format!("ADB 截图校验失败：{}", issue),
                None => "ADB 截图校验失败：请检查设备连接状态、ADB 服务和截图通道".to_string(),
            },
        };
        return Err(reason);
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
