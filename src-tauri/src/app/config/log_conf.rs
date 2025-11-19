use crate::app::app_error::{AppError, AppResult};
use crate::constant::sys_conf_path::LOG_CONFIG_PATH;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::LogLevel;

pub async fn get_log_config(
    config_manager: tauri::State<'_, ConfigManager>
) -> AppResult<LogMain>{
    let log_config= config_manager.get_conf::<LogMain>(LOG_CONFIG_PATH).await?;
    //let res = serde_json::to_string_pretty(&log_config)
        //.map_err(|e| AppError::SerializeConfErr{detail: log_config.to_string(), e: e.to_string()})?;
    Ok(log_config)
}
pub async fn set_log_config(
    config_manager: tauri::State<'_, ConfigManager>,
    log_level: &LogLevel,
    log_dir : &str,
    max_file_size : usize,
    retention_days : u32
)-> AppResult<()>{
    let mut log_config= config_manager.get_conf_mut::<LogMain>(LOG_CONFIG_PATH).await?;
    //更新日志级别
    LogMain::update_level(&log_level)?;
    log_config.log_level = log_level.clone();
    log_config.log_dir = log_dir.parse().unwrap();
    log_config.max_file_size = max_file_size;
    log_config.retention_days = retention_days;
    Ok(())
}