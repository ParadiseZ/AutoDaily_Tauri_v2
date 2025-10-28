use tauri::command;
use crate::app::dev_test::{paddle_ocr_infer, yolo_infer_test};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::infrastructure::factory::ocr_factory::{DetectorConfig, DetectorType, ModelPathType, RecognizerConfig, RecognizerType};
use crate::infrastructure::services::capture::window_cap::window_cap_test;
use crate::infrastructure::storage::capture::save_screenshot;
use crate::infrastructure::performance::{ProcessManager, ProcessConfig, create_process_config, create_process_config_with_core};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::Lazy;

// 全局进程管理器实例
static PROCESS_MANAGER: Lazy<Arc<ProcessManager>> = Lazy::new(|| {
    Arc::new(ProcessManager::new())
});

#[command]
pub fn window_capture_test(method: &str, device: &str, win_name: &str) -> String {
    window_cap_test(method, device, win_name)
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
) -> String {
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
    match yolo_infer_test(image_path, detector_conf).await{
        Ok(resp) => {
            // 后端修改
            let response = serde_json::json!({
                "status": "ok",
                "detections": resp
            });
            serde_json::to_string(&response).unwrap_or_default()
        } ,
        Err(e) =>{
            Log::error(e.to_json_response().as_str());
            let response = serde_json::json!({
                "status": "error",
                "detections": "详情请查看日志"
            });
            serde_json::to_string(&response).unwrap_or_default()
        }
    }
}

// ===================== 进程管理相关命令 =====================

/// 获取系统性能信息
#[command]
pub fn get_system_performance_info() -> String {
    let manager = &*PROCESS_MANAGER;
    let info = serde_json::json!({
        "cpuCount": manager.get_cpu_count(),
        "availableCoreIds": manager.get_available_core_ids(),
        "activeProcessCount": manager.get_active_process_count()
    });
    serde_json::to_string(&info).unwrap_or_default()
}

/// 启动测试子进程
#[command]
pub fn start_test_process(
    process_name: String,
    program: String,
    args: Vec<String>,
    core_id: Option<usize>,
    working_dir: Option<String>,
) -> Result<String, String> {
    let manager = &*PROCESS_MANAGER;
    
    let mut config = if let Some(core) = core_id {
        create_process_config_with_core(&process_name, &program, args, core)
    } else {
        create_process_config(&process_name, &program, args)
    };
    
    // 设置工作目录
    config.working_dir = working_dir;
    
    manager
        .spawn_process_with_affinity(config)
        .map_err(|e| e.to_string())
}

/// 终止指定进程
#[command]
pub fn terminate_process(process_id: String) -> Result<(), String> {
    let manager = &*PROCESS_MANAGER;
    manager.terminate_process(&process_id).map_err(|e| e.to_string())
}

/// 获取活跃进程信息
#[command]
pub fn get_active_processes_info() -> String {
    let manager = &*PROCESS_MANAGER;
    let processes_info = manager.get_active_process_status();
    serde_json::to_string(&processes_info).unwrap_or_default()
}

/// 清理已完成的进程
#[command]
pub fn cleanup_finished_processes() {
    let manager = &*PROCESS_MANAGER;
    manager.cleanup_finished_processes();
}

/// 获取进程输出
#[command]
pub fn get_process_output(process_id: String) -> Result<(String, String), String> {
    let manager = &*PROCESS_MANAGER;
    manager.get_process_output(&process_id).map_err(|e| e.to_string())
}

/// 启动简单的测试进程（Windows cmd/Linux echo）
#[command]
pub fn start_simple_test_process(core_id: Option<usize>) -> Result<String, String> {
    let manager = &*PROCESS_MANAGER;
    
    // 根据系统选择适合的命令
    #[cfg(target_os = "windows")]
    let (program, args) = ("cmd".to_string(), vec!["/C".to_string(), "echo".to_string(), "Hello from Windows!".to_string()]);
    
    #[cfg(not(target_os = "windows"))]
    let (program, args) = ("echo".to_string(), vec!["Hello from Unix!".to_string()]);
    
    let config = if let Some(core) = core_id {
        create_process_config_with_core("simple_test", &program, args, core)
    } else {
        create_process_config("simple_test", &program, args)
    };
    
    manager
        .spawn_process_with_affinity(config)
        .map_err(|e| e.to_string())
}

/// 启动CPU密集型测试进程
#[command]
pub fn start_cpu_intensive_process(core_id: Option<usize>, duration_seconds: u64) -> Result<String, String> {
    let manager = &*PROCESS_MANAGER;
    
    // 使用Python或PowerShell创建CPU密集型任务
    #[cfg(target_os = "windows")]
    let (program, args) = (
        "powershell".to_string(),
        vec![
            "-Command".to_string(),
            format!("$end = (Get-Date).AddSeconds({}); while ((Get-Date) -lt $end) {{ $sum = 0; for ($i = 0; $i -lt 100000; $i++) {{ $sum += $i * $i }} }}", duration_seconds)
        ]
    );
    
    #[cfg(not(target_os = "windows"))]
    let (program, args) = (
        "bash".to_string(),
        vec![
            "-c".to_string(),
            format!("end=$(($(date +%s) + {})); while [ $(date +%s) -lt $end ]; do sum=0; for i in {{1..100000}}; do sum=$((sum + i * i)); done; done", duration_seconds)
        ]
    );
    
    let config = if let Some(core) = core_id {
        create_process_config_with_core("cpu_test", &program, args, core)
    } else {
        create_process_config("cpu_test", &program, args)
    };
    
    manager
        .spawn_process_with_affinity(config)
        .map_err(|e| e.to_string())
}

/// 启动多个并行进程
#[command]
pub fn start_parallel_processes(process_count: usize, task_duration_seconds: u64) -> Result<Vec<String>, String> {
    let manager = &*PROCESS_MANAGER;
    let cpu_count = manager.get_cpu_count();
    let actual_process_count = std::cmp::min(process_count, cpu_count);
    
    let mut process_ids = Vec::new();
    
    for i in 0..actual_process_count {
        let core_id = i % cpu_count;
        
        #[cfg(target_os = "windows")]
        let (program, args) = (
            "powershell".to_string(),
            vec![
                "-Command".to_string(),
                format!(
                    "Write-Host 'Process {} starting on core {}'; $end = (Get-Date).AddSeconds({}); while ((Get-Date) -lt $end) {{ Start-Sleep -Milliseconds 100 }}; Write-Host 'Process {} finished'",
                    i, core_id, task_duration_seconds, i
                )
            ]
        );
        
        #[cfg(not(target_os = "windows"))]
        let (program, args) = (
            "bash".to_string(),
            vec![
                "-c".to_string(),
                format!(
                    "echo 'Process {} starting on core {}'; sleep {}; echo 'Process {} finished'",
                    i, core_id, task_duration_seconds, i
                )
            ]
        );
        
        let config = create_process_config_with_core(&format!("parallel_worker_{}", i), &program, args, core_id);
        
        let process_id = manager
            .spawn_process_with_affinity(config)
            .map_err(|e| e.to_string())?;
        
        process_ids.push(process_id);
    }
    
    Ok(process_ids)
}

#[command]
pub async fn paddle_ocr_inference_test(
    model_type : &str,
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
    image_path: &str
) -> String {
    let det_type = match det_model_type {
        1 => DetectorType::PaddleDbNet,
        2 => DetectorType::Yolo11,
        _ => DetectorType::Yolo11,
    };
    let (det_path_type, rec_path_type, dict_path_type) = match model_type {
        "build-in" => ( ModelPathType::Resource(det_model_path.into()) ,
                        ModelPathType::Resource(rec_model_path.into()),
                        ModelPathType::Resource(dict_path.into())),
        _ => ( ModelPathType::Absolute(det_model_path.into()) ,
               ModelPathType::Absolute(rec_model_path.into()),
               ModelPathType::Absolute(dict_path.into()))
    };
    let detector_conf = DetectorConfig::new(
        det_type,
        det_path_type,
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
        Some(det_db_thresh),
        Some(det_db_box_thresh),
        Some(unclip_ratio),
        Some(use_dilation)
    );

    let rec_conf = RecognizerConfig{
        recognizer_type: RecognizerType::PaddleCrnn,
        model_path: rec_path_type,
        execution_provider: rec_execution_provider.into(),
        input_width: rec_input_size,
        input_height: rec_input_size,
        dict_path: Some(dict_path_type),
        beam_width: None,

        intra_thread_num,
        intra_spinning,
        inter_thread_num,
        inter_spinning
    };
    match paddle_ocr_infer(
        detector_conf,
        rec_conf,
        image_path
    ).await{
        Ok(resp) => {
            // 后端修改
            let response = serde_json::json!({
                "status": "ok",
                "ocrResults": resp
            });
            serde_json::to_string(&response).unwrap_or_default()
        } ,
        Err(e) =>{
            Log::error(e);
            let response = serde_json::json!({
                "status": "error",
                "ocrResults": "详情请查看日志"
            });
            serde_json::to_string(&response).unwrap_or_default()
        }
    }
}