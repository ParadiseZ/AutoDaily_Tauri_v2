use crate::constant::table_name::DEVICE_TABLE;
use crate::api::infrastructure::process_api::{
    cmd_device_shutdown, cmd_prepare_device_checkpoint, cmd_spawn_device,
    cmd_sync_device_runtime_session,
};
use crate::constant::table_name::RECOVERY_CHECKPOINT_TABLE;
use crate::domain::schedule::recovery_checkpoint::RecoveryCheckpointRow;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::ipc::message::SessionCheckpointReason;
use tauri::command;
use crate::domain::devices::device_conf::DeviceTable;

async fn latest_checkpoint_updated_at(device_id: DeviceId) -> Result<Option<String>, String> {
    let query = format!(
        "SELECT execution_id, source_session_id, device_id, run_target_json, assignment_id, script_id, time_template_id, account_id, task_id, step_id, resume_mode, definition_fingerprint, updated_at
         FROM {}
         WHERE device_id = ?
         ORDER BY updated_at DESC
         LIMIT 1",
        RECOVERY_CHECKPOINT_TABLE
    );
    sqlx::query_as::<_, RecoveryCheckpointRow>(&query)
        .bind(device_id.to_string())
        .fetch_optional(get_pool())
        .await
        .map(|row| row.map(|checkpoint| checkpoint.updated_at))
        .map_err(|e| e.to_string())
}

async fn wait_for_checkpoint_refresh(
    device_id: DeviceId,
    previous_updated_at: Option<String>,
    timeout: std::time::Duration,
) -> Result<bool, String> {
    let started_at = tokio::time::Instant::now();
    loop {
        let latest = latest_checkpoint_updated_at(device_id).await?;
        if latest.is_some() && latest != previous_updated_at {
            return Ok(true);
        }

        if started_at.elapsed() >= timeout {
            return Ok(false);
        }

        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }
}

async fn reconcile_runtime_after_device_save(
    app_handle: &tauri::AppHandle,
    previous: Option<&DeviceTable>,
    device: &DeviceTable,
) -> Result<(), String> {
    let Some(previous) = previous else {
        return Ok(());
    };
    let Some(manager) = get_process_manager() else {
        return Ok(());
    };

    if !manager.is_running(&device.id).await {
        return Ok(());
    }

    if previous.data.0.cores != device.data.0.cores {
        let previous_checkpoint = latest_checkpoint_updated_at(device.id).await?;
        let _ = cmd_prepare_device_checkpoint(device.id, SessionCheckpointReason::Restart).await?;
        let _ = wait_for_checkpoint_refresh(
            device.id,
            previous_checkpoint,
            std::time::Duration::from_secs(3),
        )
        .await?;
        cmd_device_shutdown(device.id).await?;
        cmd_spawn_device(app_handle.clone(), device.id).await?;
        cmd_sync_device_runtime_session(app_handle.clone(), device.id).await?;
        return Ok(());
    }

    if previous.data.0.execution_policy != device.data.0.execution_policy {
        cmd_sync_device_runtime_session(app_handle.clone(), device.id).await?;
    }

    Ok(())
}

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
    let previous = DbRepo::get_by_id::<DeviceTable>(DEVICE_TABLE, &device.id.to_string()).await?;
    DbRepo::upsert_id_data(DEVICE_TABLE, &device.id.to_string(), &device.data).await?;
    reconcile_runtime_after_device_save(&app_handle, previous.as_ref(), &device).await
}

/// 删除设备配置
#[command]
pub async fn delete_device_cmd(device_id: DeviceId) -> Result<(), String> {
    if let Some(manager) = get_process_manager() {
        if manager.is_running(&device_id).await {
            cmd_device_shutdown(device_id).await?;
        }
    }
    DbRepo::delete(DEVICE_TABLE, &device_id.to_string()).await
}

/// 获取当前 CPU 核心数
#[command]
pub fn get_cpu_count_cmd() -> usize {
    num_cpus::get()
}
