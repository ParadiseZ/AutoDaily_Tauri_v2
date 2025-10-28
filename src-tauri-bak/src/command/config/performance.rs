use tauri::command;
use crate::app::config::performance::{get_performance_config, set_performance_config};
use crate::domain::entities::app_result::AppResult;
use crate::domain::entities::config::performance::Performance;
use crate::domain::manager::conf_mgr::ConfigManager;
use crate::infrastructure::services::capture::window_cap::get_cpu_cores;

// ===== 性能配置相关命令 =====
/// 获取系统CPU核心数
#[command]
pub fn get_cpu_cores_cmd() -> AppResult<u32> {
    get_cpu_cores()
}

/// 获取性能配置
#[command]
pub async fn get_performance_cmd(config_manager: tauri::State<'_, ConfigManager>) -> AppResult<String> {
    get_performance_config(config_manager).await
}

/// 设置性能配置
#[command]
pub async fn set_performance_cmd(config_manager: tauri::State<'_, ConfigManager>, performance: Performance) -> AppResult<()> {
    set_performance_config(config_manager, performance.max_devices, performance.cores_per_device).await
}