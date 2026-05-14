use crate::infrastructure::db::get_pool;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tauri::{AppHandle, Emitter};

const SCRIPT_TRANSFER_EVENT: &str = "script-transfer";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTransferRecord {
    pub id: String,
    pub direction: String,
    pub local_script_id: Option<String>,
    pub cloud_script_id: Option<String>,
    pub script_name: Option<String>,
    pub status: String,
    pub model_file_count: i64,
    pub completed_model_file_count: i64,
    pub latest_file_name: Option<String>,
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub latest_message: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTransferProgressEvent {
    pub id: String,
    pub direction: String,
    pub local_script_id: Option<String>,
    pub cloud_script_id: Option<String>,
    pub script_name: Option<String>,
    pub status: String,
    pub model_file_count: i64,
    pub completed_model_file_count: i64,
    pub current_file_name: Option<String>,
    pub latest_file_name: Option<String>,
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub latest_message: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct CreateScriptTransferRecordInput {
    pub id: String,
    pub direction: String,
    pub local_script_id: Option<String>,
    pub cloud_script_id: Option<String>,
    pub script_name: Option<String>,
    pub status: String,
    pub model_file_count: i64,
    pub completed_model_file_count: i64,
    pub latest_file_name: Option<String>,
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub latest_message: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FinishScriptTransferRecordInput {
    pub id: String,
    pub status: String,
    pub completed_model_file_count: i64,
    pub latest_file_name: Option<String>,
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub latest_message: Option<String>,
    pub error_message: Option<String>,
    pub finished_at: Option<String>,
}

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn emit_script_transfer_event(app_handle: &AppHandle, payload: &ScriptTransferProgressEvent) {
    let _ = app_handle.emit(SCRIPT_TRANSFER_EVENT, payload);
}

pub async fn insert_script_transfer_record(input: CreateScriptTransferRecordInput) -> Result<(), String> {
    let pool = get_pool();
    let updated_at = input.finished_at.clone().unwrap_or_else(now_rfc3339);

    sqlx::query(
        "INSERT INTO script_transfer_records (
            id,
            direction,
            local_script_id,
            cloud_script_id,
            script_name,
            status,
            model_file_count,
            completed_model_file_count,
            latest_file_name,
            bytes_transferred,
            total_bytes,
            latest_message,
            error_message,
            started_at,
            finished_at,
            created_at,
            updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(input.id)
    .bind(input.direction)
    .bind(input.local_script_id)
    .bind(input.cloud_script_id)
    .bind(input.script_name)
    .bind(input.status)
    .bind(input.model_file_count)
    .bind(input.completed_model_file_count)
    .bind(input.latest_file_name)
    .bind(input.bytes_transferred)
    .bind(input.total_bytes)
    .bind(input.latest_message)
    .bind(input.error_message)
    .bind(input.started_at.clone())
    .bind(input.finished_at)
    .bind(input.started_at)
    .bind(updated_at)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

pub async fn finish_script_transfer_record(input: FinishScriptTransferRecordInput) -> Result<(), String> {
    let pool = get_pool();
    let updated_at = input.finished_at.clone().unwrap_or_else(now_rfc3339);

    sqlx::query(
        "UPDATE script_transfer_records
         SET status = ?,
             completed_model_file_count = ?,
             latest_file_name = ?,
             bytes_transferred = ?,
             total_bytes = ?,
             latest_message = ?,
             error_message = ?,
             finished_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(input.status)
    .bind(input.completed_model_file_count)
    .bind(input.latest_file_name)
    .bind(input.bytes_transferred)
    .bind(input.total_bytes)
    .bind(input.latest_message)
    .bind(input.error_message)
    .bind(input.finished_at)
    .bind(updated_at)
    .bind(input.id)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_script_transfer_records_cmd(
    direction: Option<String>,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ScriptTransferRecord>, String> {
    let pool = get_pool();
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT
            id,
            direction,
            local_script_id,
            cloud_script_id,
            script_name,
            status,
            model_file_count,
            completed_model_file_count,
            latest_file_name,
            bytes_transferred,
            total_bytes,
            latest_message,
            error_message,
            started_at,
            finished_at,
            created_at,
            updated_at
         FROM script_transfer_records
         WHERE 1 = 1",
    );

    if let Some(direction) = direction.filter(|value| !value.trim().is_empty()) {
        query.push(" AND direction = ").push_bind(direction);
    }
    if let Some(local_script_id) = local_script_id.filter(|value| !value.trim().is_empty()) {
        query
            .push(" AND local_script_id = ")
            .push_bind(local_script_id);
    }
    if let Some(cloud_script_id) = cloud_script_id.filter(|value| !value.trim().is_empty()) {
        query
            .push(" AND cloud_script_id = ")
            .push_bind(cloud_script_id);
    }

    query.push(" ORDER BY datetime(COALESCE(updated_at, started_at)) DESC, id DESC");

    if let Some(limit) = limit.filter(|value| *value > 0) {
        query.push(" LIMIT ").push_bind(limit);
    }

    query
        .build_query_as::<ScriptTransferRecord>()
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_script_transfer_record_cmd(record_id: String) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query("DELETE FROM script_transfer_records WHERE id = ?")
        .bind(record_id)
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_script_transfer_records_cmd(
    direction: Option<String>,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
) -> Result<(), String> {
    let pool = get_pool();
    let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM script_transfer_records WHERE 1 = 1");

    if let Some(direction) = direction.filter(|value| !value.trim().is_empty()) {
        query.push(" AND direction = ").push_bind(direction);
    }
    if let Some(local_script_id) = local_script_id.filter(|value| !value.trim().is_empty()) {
        query
            .push(" AND local_script_id = ")
            .push_bind(local_script_id);
    }
    if let Some(cloud_script_id) = cloud_script_id.filter(|value| !value.trim().is_empty()) {
        query
            .push(" AND cloud_script_id = ")
            .push_bind(cloud_script_id);
    }

    query
        .build()
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
