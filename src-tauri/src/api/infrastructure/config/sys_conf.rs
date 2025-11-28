use crate::app::config::sys_conf::set_system_settings_app;
use crate::domain::config::sys_conf::SystemConfig;
use tauri::{command, AppHandle};

/// 设置系统设置配置
#[command]
pub async fn set_system_settings_cmd(
    app_handle: AppHandle,
    system_config: SystemConfig
) -> Result<String, String> {
    match set_system_settings_app(  &app_handle,system_config).await {
        Ok(_) => Ok("设置成功！".to_string()),
        Err(e) => Err(format!("设置失败:{}", e.to_string())),
    }
}
