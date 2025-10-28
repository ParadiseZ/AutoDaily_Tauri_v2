
// ===== 日志配置相关命令 =====

use tauri::command;
use crate::app::config::log_conf::{get_log_config, set_log_config};
use crate::domain::entities::app_result::AppResult;
use crate::domain::entities::config::log_conf::Log;
use crate::domain::manager::conf_mgr::ConfigManager;

/// 获取日志配置
#[command]
pub async fn get_log_cmd(
    config_manager: tauri::State<'_, ConfigManager>
) -> AppResult<String> {
    get_log_config(config_manager).await
}

/// 设置日志配置
#[command]
pub async fn set_log_cmd(
    config_manager: tauri::State<'_, ConfigManager>,
    log_config: Log,
) -> AppResult<()> {
    let _ = set_log_config(
        config_manager,
        &log_config.log_level,
        &*log_config.log_dir,
        log_config.max_file_size,
        log_config.retention_days
    ).await;
    Ok(())
}