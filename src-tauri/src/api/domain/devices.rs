use crate::api::infrastructure::process_api::{
    enqueue_device_config_reconcile_job, notify_auto_dispatch_planner, DispatchPlanner,
};
use crate::constant::table_name::{ASSIGNMENT_SCHEDULE_TABLE, ASSIGNMENT_TABLE, DEVICE_TABLE};
use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::logging::log_trait::Log;
use sqlx::Acquire;
use tauri::command;

/// 获取所有设备配置
#[command]
pub async fn get_all_devices_cmd() -> Result<Vec<DeviceTable>, String> {
    DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await
}

/// 根据 ID 获取设备配置
#[command]
pub async fn get_device_by_id_cmd(device_id: DeviceId) -> Result<Option<DeviceTable>, String> {
    DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string()).await
}

/// 保存（新增或更新）设备配置
#[command]
pub async fn save_device_cmd(
    app_handle: tauri::AppHandle,
    device: DeviceTable,
) -> Result<(), String> {
    let device_id = device.id.to_string();
    let device_name = device.data.0.device_name.clone();
    let result = async {
        let previous =
            DbRepo::get_by_id::<DeviceTable>(DEVICE_TABLE, &device.id.to_string()).await?;
        DbRepo::upsert_id_data(DEVICE_TABLE, &device_id, &device.data).await?;
        notify_auto_dispatch_planner();
        enqueue_device_config_reconcile_job(&app_handle, previous, device)?;
        Ok(())
    }
    .await;

    if let Err(error) = &result {
        Log::error(&format!(
            "[ device ] 保存设备失败 device_id={} device_name={} error={}",
            device_id, device_name, error
        ));
    }

    result
}

async fn ensure_device_deletable(device_id: DeviceId) -> Result<(), String> {
    if let Some(manager) = get_process_manager() {
        if manager.is_running(&device_id).await {
            return Err("设备子进程仍在运行，请先关闭设备后再删除。".to_string());
        }
    }

    let planner_state = DispatchPlanner::init().snapshot_device_state(device_id)?;
    if planner_state.active_dispatch.is_some()
        || !planner_state.pending_dispatches.is_empty()
        || !planner_state.pending_debug_sessions.is_empty()
    {
        return Err("设备仍有活动或待派发的运行任务，请停止并清空运行状态后再删除。".to_string());
    }

    let active_schedule_count = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM {} WHERE device_id = ? AND status IN ('planned', 'dispatched', 'running')",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(device_id.to_string())
    .fetch_one(get_pool())
    .await
    .map_err(|error| error.to_string())?;
    if active_schedule_count > 0 {
        return Err("设备仍有未完成的调度记录，请先停止或清理调度后再删除。".to_string());
    }

    Ok(())
}

/// 删除设备配置
#[command]
pub async fn delete_device_cmd(device_id: DeviceId) -> Result<(), String> {
    let result = async {
        ensure_device_deletable(device_id).await?;

        let mut tx = get_pool()
            .begin()
            .await
            .map_err(|error| error.to_string())?;
        let conn = tx.acquire().await.map_err(|error| error.to_string())?;
        sqlx::query(&format!(
            "DELETE FROM {} WHERE device_id = ?",
            ASSIGNMENT_TABLE
        ))
        .bind(device_id.to_string())
        .execute(&mut *conn)
        .await
        .map_err(|error| error.to_string())?;
        sqlx::query(&format!("DELETE FROM {} WHERE id = ?", DEVICE_TABLE))
            .bind(device_id.to_string())
            .execute(&mut *conn)
            .await
            .map_err(|error| error.to_string())?;
        tx.commit().await.map_err(|error| error.to_string())?;

        let _ = DispatchPlanner::init().clear_device_state(device_id);
        notify_auto_dispatch_planner();
        Ok(())
    }
    .await;

    if let Err(error) = &result {
        Log::error(&format!(
            "[ device ] 删除设备失败 device_id={} error={}",
            device_id, error
        ));
    }

    result
}

/// 获取当前 CPU 核心数
#[command]
pub fn get_cpu_count_cmd() -> usize {
    num_cpus::get()
}
