use crate::app::config::sys_conf::{get_system_settings, save_window_state_if_enabled, set_system_settings};
use crate::domain::entities::app_result::AppResult;
use crate::domain::entities::config::sys_conf::SystemConfig;
use crate::domain::manager::conf_mgr::ConfigManager;
use tauri::command;

/// 获取系统设置配置
#[command]
pub async fn get_system_settings_cmd(
    manager: tauri::State<'_, ConfigManager>
) -> AppResult<String> {
    get_system_settings(manager).await
}

/// 设置系统设置配置
#[command]
pub async fn set_system_settings_cmd(
    manager: tauri::State<'_, ConfigManager>,
    system_config: SystemConfig
) -> AppResult<()> {
    Ok(set_system_settings(manager, system_config).await?)
}

/// 保存窗口状态
#[command]
pub async fn save_window_state_cmd(
    manager: tauri::State<'_, ConfigManager>
) -> AppResult<()> {
    Ok(save_window_state_if_enabled(manager).await?)
}

