use crate::app::config::log_conf::{
    clean_logs_now_app, get_log_config_app, update_child_log_level_app, update_log_dir_app,
    update_log_level_app, update_retention_days_app,
};
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::LogLevel;
use tauri::command;

/// 更新主进程日志级别
#[command]
pub async fn update_log_level_cmd(log_level: LogLevel) -> Result<String, String> {
    match update_log_level_app(&log_level).await {
        Ok(_) => Ok("设置成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}

/// 更新日志目录
#[command]
pub async fn update_log_dir_cmd(log_dir: String) -> Result<String, String> {
    match update_log_dir_app(&log_dir).await {
        Ok(_) => Ok("日志目录更新成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}

/// 更新日志保留天数
#[command]
pub async fn update_retention_days_cmd(days: u32) -> Result<String, String> {
    match update_retention_days_app(days).await {
        Ok(_) => Ok("保留天数更新成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}

/// 获取当前日志配置
#[command]
pub async fn get_log_config_cmd() -> Result<LogMain, String> {
    match get_log_config_app().await {
        Ok(config) => Ok(config),
        Err(err) => Err(format!("获取失败：{}", err)),
    }
}

/// 手动触发日志清理
#[command]
pub async fn clean_logs_now_cmd() -> Result<String, String> {
    match clean_logs_now_app().await {
        Ok(_) => Ok("清理完成！".to_string()),
        Err(err) => Err(format!("清理失败：{}", err)),
    }
}

/// 更新子进程日志级别
#[command]
pub async fn update_child_log_level_cmd(
    device_id: DeviceId,
    log_level: LogLevel,
) -> Result<String, String> {
    match update_child_log_level_app(device_id, &log_level).await {
        Ok(_) => Ok("子进程日志级别更新成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}
