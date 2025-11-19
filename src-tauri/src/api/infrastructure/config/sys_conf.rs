use crate::app::config::sys_conf::{get_system_settings, save_window_state_if_enabled, set_system_settings};
use crate::domain::config::sys_conf::SystemConfig;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use tauri::{command, AppHandle, Manager};

/// 获取系统设置配置
#[command]
pub async fn get_system_settings_cmd(
    manager: tauri::State<'_, ConfigManager>
) -> Result<SystemConfig,String> {
    match get_system_settings(manager).await{
        Ok(config) => Ok( config),
        Err(e) => Err(format!("获取系统设置失败:{}", e.to_string()))
    }
}

/// 设置系统设置配置
#[command]
pub async fn set_system_settings_cmd(
    manager: tauri::State<'_, ConfigManager>,
    system_config: SystemConfig
) -> Result<String,String> {
    match set_system_settings(manager, system_config).await {
        Ok(_) => Ok("设置成功！".to_string()),
        Err(e) => Err(format!("设置失败:{}",e.to_string()))
    }
}

/// 保存窗口状态
#[command]
pub async fn save_window_state_cmd(
    app_handle: &AppHandle
)->Result<(),String>{
    if let Err(e) = save_window_state_if_enabled(app_handle).await{
        return Err(format!("保存窗口状态失败:{}",e.to_string()))
    }
    Ok(())
}