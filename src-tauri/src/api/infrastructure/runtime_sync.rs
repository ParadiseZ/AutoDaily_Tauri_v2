use crate::api::infrastructure::process_api::cmd_sync_device_runtime_session;
use crate::constant::table_name::ASSIGNMENT_TABLE;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::{DeviceId, ScriptId};
use crate::infrastructure::db::get_pool;

fn parse_device_id(value: String) -> Result<DeviceId, String> {
    uuid::Uuid::parse_str(&value)
        .map(DeviceId::from)
        .map_err(|error| error.to_string())
}

pub async fn load_assigned_device_ids_by_script(script_id: ScriptId) -> Result<Vec<DeviceId>, String> {
    let query = format!(
        "SELECT DISTINCT device_id FROM {} WHERE script_id = ? ORDER BY device_id ASC",
        ASSIGNMENT_TABLE
    );
    let rows = sqlx::query_scalar::<_, String>(&query)
        .bind(script_id.to_string())
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

    rows.into_iter().map(parse_device_id).collect()
}

pub async fn load_assigned_device_ids_by_time_template(
    template_id: &str,
) -> Result<Vec<DeviceId>, String> {
    let query = format!(
        "SELECT DISTINCT device_id FROM {} WHERE time_template_id = ? ORDER BY device_id ASC",
        ASSIGNMENT_TABLE
    );
    let rows = sqlx::query_scalar::<_, String>(&query)
        .bind(template_id)
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

    rows.into_iter().map(parse_device_id).collect()
}

pub async fn sync_device_session_if_online(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let Some(manager) = get_process_manager() else {
        return Ok(());
    };

    if manager.is_running(&device_id).await {
        cmd_sync_device_runtime_session(app_handle.clone(), device_id).await?;
    }

    Ok(())
}

pub async fn sync_device_sessions_if_online(
    app_handle: &tauri::AppHandle,
    device_ids: impl IntoIterator<Item = DeviceId>,
) -> Result<(), String> {
    for device_id in device_ids {
        sync_device_session_if_online(app_handle, device_id).await?;
    }

    Ok(())
}
