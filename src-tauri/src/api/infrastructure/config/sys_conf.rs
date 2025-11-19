use crate::api::api_response::ApiResponse;
use crate::app::config::sys_conf::{get_system_settings, save_window_state_if_enabled, set_system_settings};
use crate::constant::sys_conf_path::SYSTEM_SETTINGS_PATH;
use crate::domain::config::sys_conf::SystemConfig;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use tauri::{command, AppHandle, Manager};

/// 获取系统设置配置
#[command]
pub async fn get_system_settings_cmd(
    manager: tauri::State<'_, ConfigManager>
) -> ApiResponse<String> {
    match get_system_settings(manager).await{
        Ok(config) => ApiResponse::ok(Some(config)),
        Err(e) => ApiResponse::error(Some("获取系统设置失败！".to_string()))
    }
}

/// 设置系统设置配置
#[command]
pub async fn set_system_settings_cmd(
    manager: tauri::State<'_, ConfigManager>,
    system_config: SystemConfig
) -> ApiResponse<String> {
    match set_system_settings(manager, system_config).await {
        Ok(_) => ApiResponse::ok(Some("设置成功！".to_string())),
        Err(e) => ApiResponse::error(Some("设置快捷键失败！".to_string()))
    }
}

/// 保存窗口状态
#[command]
pub async fn save_window_state_cmd(
    app_handle: &AppHandle
) -> ApiResponse<()> {
    Ok(save_window_state_if_enabled(app_handle).await?)
}