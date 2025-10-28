use crate::constant::sys_conf_path::LOG_CONFIG_PATH;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, LogLevel, Logger};
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager};

pub async fn get_log_config(
    config_manager: tauri::State<'_, ConfigManager>
) -> AppResult<String>{
    let log_config= config_manager.get_conf::<Log>(LOG_CONFIG_PATH).await?;
    let res = serde_json::to_string(&log_config)
        .map_err(|e| AppError::ConfigError(format!("序列化配置失败：{}",e)))?;
    Ok(res)
}
pub async fn set_log_config(
    config_manager: tauri::State<'_, ConfigManager>,
    log_level: &LogLevel,
    log_dir : &str,
    max_file_size : usize,
    retention_days : u32
)-> AppResult<()>{
    let mut log_config= config_manager.get_conf_mut::<Log>(LOG_CONFIG_PATH).await?;
    //更新日志级别
    Log::update_level(&log_level)?;
    log_config.log_level = log_level.clone();
    log_config.log_dir = log_dir.parse().unwrap();
    log_config.max_file_size = max_file_size;
    log_config.retention_days = retention_days;
    Ok(())
}