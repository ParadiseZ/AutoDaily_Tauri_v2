// ===== 空闲监控相关命令 =====

use crate::domain::entities::app_result::AppResult;
use crate::domain::manager::conf_mgr::ConfigManager;
use crate::domain::services::idle_monitor::IdleMonitorTrait;
use crate::infrastructure::entities::config::idle_monitor::IdleMonitor;
use std::sync::Arc;
use tauri::{command, State};

#[command]
pub async fn start_idle_monitoring_cmd(
    manager: State<'_, ConfigManager>,
    idle_monitor: State<'_, Arc<IdleMonitor>>
) -> AppResult<()> {
    idle_monitor.start_monitoring(manager.inner().clone()).await?;
    Ok(())
}

#[command]
pub async fn stop_idle_monitoring_cmd(
    idle_monitor: State<'_, Arc<IdleMonitor>>
) -> AppResult<()> {
    idle_monitor.stop_monitoring().await;
    Ok(())
}

#[command]
pub async fn update_activity_cmd(
    idle_monitor: State<'_, Arc<IdleMonitor>>
) -> AppResult<()> {
    idle_monitor.update_activity().await;
    Ok(())
}

#[command]
pub async fn cancel_shutdown_cmd(
    idle_monitor: State<'_, Arc<IdleMonitor>>
) -> AppResult<()> {
    idle_monitor.cancel_shutdown().await?;
    Ok(())
}
