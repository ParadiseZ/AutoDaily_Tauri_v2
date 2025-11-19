use crate::api::api_response::ApiResponse;
use crate::app::dev_test::{paddle_ocr_infer, yolo_infer_test};
use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::devices::device_conf::DeviceConfig;
use crate::infrastructure::devices::device_ctx::DeviceCtx;
use crate::infrastructure::image::save_image::save_screenshot;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::path_resolve::model_path::PathUtil;
use crate::infrastructure::vision::ocr_factory::{DetectorConfig, DetectorType, RecognizerConfig, RecognizerType};
use base64::engine::general_purpose;
use base64::Engine;
use core_affinity::get_core_ids;
use image::DynamicImage;
use std::io::Cursor;
use std::sync::{Arc, RwLock};
use tauri::{command, AppHandle};

#[command]
pub async fn dev_capture_test(method: u8, device_conf: DeviceConfig, adb_conf:ADBConnectConfig) -> ApiResponse<String> {

    let adb_ctx = ADBCtx::new(adb_conf, get_core_ids()[0]);
    let device_ctx = DeviceCtx::new(Arc::new(RwLock::new(device_conf)), CaptureMethod::from(method) ,Arc::new(RwLock::new(adb_ctx)));

    if !device_ctx.valid_capture().await {
        return ApiResponse::error("验证截图功能失败！".into());
    }
    match device_ctx.get_screenshot().await{
        Some(image_data) => {
            let mut cursor = Cursor::new(Vec::new());
            match DynamicImage::ImageRgba8(image_data).write_to(&mut cursor, image::ImageFormat::Png) {
                Ok(_) => {
                    let buffer = cursor.into_inner();
                    let base64_string = general_purpose::STANDARD.encode(&buffer);
                    let msg = format!("转换base64编码截图成功：{}KB", base64_string.len() / 1024);
                    ApiResponse::success(Some(base64_string),Some(msg))
                }
                Err(e) => {
                    Log::error(&format!("图像转换为base64失败: {:?}", e));
                    ApiResponse::error(Some("base64编码失败！".to_string()))
                }
            }
        },
        _ => ApiResponse::error(Some("截图失败！".to_string()))
    }
}

/// 保存截图到文件
#[command]
pub fn save_captured_image(
    image_data: &str,
    device_name: &str,
    image_type: &str,
)-> ApiResponse<String>{
    match save_screenshot(image_data, device_name, image_type){
        Ok(msg) => {
            ApiResponse::success(None,Some(msg))
        },
        Err(e) => {
            ApiResponse::error(Some(e))
        }
    }
}

#[command]
pub async fn yolo_inference_test(
    model_path: &str,
    execution_provider: &str,
    class_file_path: &str,
    image_path: &str,
    target_size: u32,
    intra_thread_num: usize,
    intra_spinning: bool,
    inter_thread_num: usize,
    inter_spinning: bool,
    confidence_threshold: f32,
    iou_threshold: f32
) -> Result<ApiResponse<Vec<DetResult>>, String> {
    let detector_conf = DetectorConfig{
        detector_type: DetectorType::Yolo11,
        model_path: model_path.into(),
        execution_provider: execution_provider.into(),
        input_width: target_size,
        input_height: target_size,
        intra_thread_num,
        intra_spinning,
        inter_thread_num,
        inter_spinning,
        confidence_thresh: Some(confidence_threshold),
        iou_thresh: Some(iou_threshold),
        class_count: None,
        class_labels: None,
        class_file_path: Some(class_file_path.into()),
        db_thresh: None,
        db_box_thresh: None,
        unclip_ratio: None,
        use_dilation: None,
    };
    let result = yolo_infer_test(image_path, detector_conf).await;
    Ok(ApiResponse::from(result))
}

#[command]
pub async fn paddle_ocr_inference_test(
    model_path_type : &str,
    intra_thread_num: usize,
    intra_spinning: bool,
    inter_thread_num: usize,
    inter_spinning: bool,
    det_model_path: &str,
    rec_model_path: &str,
    class_file_path : &str,
    dict_path: &str,
    det_input_size: u32,
    rec_input_size: u32,
    det_db_thresh: f32,
    det_db_box_thresh: f32,
    det_confidence_thresh: f32,
    det_nms_iou_thresh: f32,
    unclip_ratio: f32,
    use_dilation: bool,
    det_model_type: u8,
    det_execution_provider: &str,
    rec_execution_provider: &str,
    image_path: &str,
    app_handle: AppHandle,
) -> Result<ApiResponse<Vec<OcrResult>>, String> {
    let det_type = match det_model_type {
        1 => DetectorType::PaddleDbNet,
        2 => DetectorType::Yolo11,
        _ => DetectorType::Yolo11,
    };
    let det_model_path = PathUtil::resolve_path(&app_handle,model_path_type,det_model_path)?;
    let rec_path_type = PathUtil::resolve_path(&app_handle,model_path_type,rec_model_path)?;
    let dict_path_type = PathUtil::resolve_path(&app_handle,model_path_type,dict_path)?;
    let detector_conf = match det_type {
        DetectorType::Yolo11 => DetectorConfig::new_yolo(
            det_type,
            det_model_path,
            det_execution_provider.into(),
            det_input_size,
            det_input_size,
            intra_thread_num,
            intra_spinning,
            inter_thread_num,
            inter_spinning,
            Some(det_confidence_thresh),
            Some(det_nms_iou_thresh),
            None,
            None,
            Some(class_file_path.into()),
        ),
        DetectorType::PaddleDbNet => DetectorConfig::new_paddle_det(
            det_type,
            det_model_path,
            det_execution_provider.into(),
            det_input_size,
            det_input_size,
            intra_thread_num,
            intra_spinning,
            inter_thread_num,
            inter_spinning,
            Some(det_db_thresh),
            Some(det_db_box_thresh),
            Some(unclip_ratio),
            Some(use_dilation),
        )
    };

    let rec_conf = RecognizerConfig{
        recognizer_type: RecognizerType::PaddleCrnn,
        model_path: rec_path_type,
        execution_provider: rec_execution_provider.into(),
        input_width: rec_input_size,
        input_height: rec_input_size,
        dict_path: Some(dict_path_type),

        intra_thread_num,
        intra_spinning,
        inter_thread_num,
        inter_spinning
    };
    let result = paddle_ocr_infer(
        detector_conf,
        rec_conf,
        image_path
    ).await;
    Ok(ApiResponse::from(result))
}