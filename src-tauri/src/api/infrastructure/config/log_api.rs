use tauri::command;
use crate::api::api_response::ApiResponse;
use crate::app::config::log_conf::{get_log_config, set_log_config};
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::logging::config::LogMain;

/// 获取日志配置
#[command]
pub async fn get_log_cmd(
    config_manager: tauri::State<'_, ConfigManager>
) -> ApiResponse<String> {
    match get_log_config(config_manager).await{
        Ok(config) => ApiResponse::success(Some(config), None),
        Err(err) => ApiResponse::error(Some(err.to_string()))
    }
}

/// 设置日志配置
#[command]
pub async fn set_log_cmd(
    config_manager: tauri::State<'_, ConfigManager>,
    log_config: LogMain,
) -> ApiResponse<()> {
    match set_log_config(
        config_manager,
        &log_config.log_level,
        &*log_config.log_dir,
        log_config.max_file_size,
        log_config.retention_days
    ).await{
        Ok(_) => ApiResponse::success(None,Some("设置成功！".to_string())),
        Err(err) => ApiResponse::error(Some(err.to_string()))
    }
}