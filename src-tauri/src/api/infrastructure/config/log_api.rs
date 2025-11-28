use crate::app::config::log_conf::{update_log_level_app};
use crate::infrastructure::logging::LogLevel;
use tauri::{command};

/// 设置日志配置
#[command]
pub async fn update_log_level_cmd(
    log_level: LogLevel,
) -> Result<String, String> {
    match update_log_level_app(
        &log_level
    ).await {
        Ok(_) => Ok("设置成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err.to_string())),
    }
}
