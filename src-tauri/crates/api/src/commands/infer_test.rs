use tauri::command;

#[command]
pub fn dev_capture_test(method: &str, device: &str, win_name: &str) -> ApiResponse<String> {
    match method {
        "adb" => {
            let device : DeviceConfig = DeviceConfig::default();
            //set_cap_way(device);
            //screen_cap_test(method, device, win_name)
        }
        _ =>{
            let window_init = WindowInfo::init(win_name);
            set_cap_way(window_init);
            if let Some(img) = get_base64_capture(){
                ApiResponse::success( img)
            }else {
                ApiResponse::error( "未获取到截图，详情请查看日志".to_string() )
            }
        }
    }
}

/// 保存截图到文件
#[command]
pub fn save_captured_image(
    image_data: &str,
    device_name: &str,
    image_type: &str,
) -> Result<String, String> {
    save_screenshot(image_data, device_name, image_type)
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
) -> ApiResponse<Vec<DetResult>> {
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
    Ok(yolo_infer_test(image_path, detector_conf).await?)
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
    app_handle: &AppHandle,
) -> ApiResponse<Vec<OcrResult>> {
    let det_type = match det_model_type {
        1 => DetectorType::PaddleDbNet,
        2 => DetectorType::Yolo11,
        _ => DetectorType::Yolo11,
    };
    let det_model_path = PathUtil::resolve_path(app_handle,model_path_type,det_model_path)?;
    let rec_path_type = PathUtil::resolve_path(app_handle,model_path_type,rec_model_path)?;
    let dict_path_type = PathUtil::resolve_path(app_handle,model_path_type,dict_path)?;
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
    Ok(paddle_ocr_infer(
        detector_conf,
        rec_conf,
        image_path
    ).await?)
}