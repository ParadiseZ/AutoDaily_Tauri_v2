use domain_script::ScriptTransferRecord;
use sqlx::{FromRow, QueryBuilder, Sqlite};

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

#[derive(FromRow)]
struct ScriptTransferRow {
    id: String,
    direction: String,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
    script_name: Option<String>,
    status: String,
    model_file_count: i64,
    completed_model_file_count: i64,
    latest_file_name: Option<String>,
    bytes_transferred: i64,
    total_bytes: i64,
    latest_message: Option<String>,
    error_message: Option<String>,
    started_at: String,
    finished_at: Option<String>,
    created_at: String,
    updated_at: String,
}
impl From<ScriptTransferRow> for ScriptTransferRecord {
    fn from(row: ScriptTransferRow) -> Self {
        Self {
            id: row.id,
            direction: row.direction,
            local_script_id: row.local_script_id,
            cloud_script_id: row.cloud_script_id,
            script_name: row.script_name,
            status: row.status,
            model_file_count: row.model_file_count,
            completed_model_file_count: row.completed_model_file_count,
            latest_file_name: row.latest_file_name,
            bytes_transferred: row.bytes_transferred,
            total_bytes: row.total_bytes,
            latest_message: row.latest_message,
            error_message: row.error_message,
            started_at: row.started_at,
            finished_at: row.finished_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub async fn insert_script_transfer_record(
    input: CreateScriptTransferRecordInput,
    updated_at: String,
) -> Result<(), String> {
    sqlx::query("INSERT INTO script_transfer_records (id, direction, local_script_id, cloud_script_id, script_name, status, model_file_count, completed_model_file_count, latest_file_name, bytes_transferred, total_bytes, latest_message, error_message, started_at, finished_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(input.id).bind(input.direction).bind(input.local_script_id).bind(input.cloud_script_id).bind(input.script_name).bind(input.status).bind(input.model_file_count).bind(input.completed_model_file_count).bind(input.latest_file_name).bind(input.bytes_transferred).bind(input.total_bytes).bind(input.latest_message).bind(input.error_message).bind(input.started_at.clone()).bind(input.finished_at).bind(input.started_at).bind(updated_at)
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}
pub async fn finish_script_transfer_record(
    input: FinishScriptTransferRecordInput,
    updated_at: String,
) -> Result<(), String> {
    sqlx::query("UPDATE script_transfer_records SET status = ?, completed_model_file_count = ?, latest_file_name = ?, bytes_transferred = ?, total_bytes = ?, latest_message = ?, error_message = ?, finished_at = ?, updated_at = ? WHERE id = ?")
        .bind(input.status).bind(input.completed_model_file_count).bind(input.latest_file_name).bind(input.bytes_transferred).bind(input.total_bytes).bind(input.latest_message).bind(input.error_message).bind(input.finished_at).bind(updated_at).bind(input.id)
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}
pub async fn list_script_transfer_records(
    direction: Option<String>,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ScriptTransferRecord>, String> {
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id, direction, local_script_id, cloud_script_id, script_name, status, model_file_count, completed_model_file_count, latest_file_name, bytes_transferred, total_bytes, latest_message, error_message, started_at, finished_at, created_at, updated_at FROM script_transfer_records WHERE 1 = 1",
    );
    if let Some(value) = direction.filter(|value| !value.trim().is_empty()) {
        query.push(" AND direction = ").push_bind(value);
    }
    if let Some(value) = local_script_id.filter(|value| !value.trim().is_empty()) {
        query.push(" AND local_script_id = ").push_bind(value);
    }
    if let Some(value) = cloud_script_id.filter(|value| !value.trim().is_empty()) {
        query.push(" AND cloud_script_id = ").push_bind(value);
    }
    query.push(" ORDER BY datetime(COALESCE(updated_at, started_at)) DESC, id DESC");
    if let Some(value) = limit.filter(|value| *value > 0) {
        query.push(" LIMIT ").push_bind(value);
    }
    query
        .build_query_as::<ScriptTransferRow>()
        .fetch_all(crate::bootstrap::get_pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|error| error.to_string())
}
pub async fn delete_script_transfer_record(id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM script_transfer_records WHERE id = ?")
        .bind(id)
        .execute(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
pub async fn clear_script_transfer_records(
    direction: Option<String>,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
) -> Result<(), String> {
    let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM script_transfer_records WHERE 1 = 1");
    if let Some(value) = direction.filter(|value| !value.trim().is_empty()) {
        query.push(" AND direction = ").push_bind(value);
    }
    if let Some(value) = local_script_id.filter(|value| !value.trim().is_empty()) {
        query.push(" AND local_script_id = ").push_bind(value);
    }
    if let Some(value) = cloud_script_id.filter(|value| !value.trim().is_empty()) {
        query.push(" AND cloud_script_id = ").push_bind(value);
    }
    query
        .build()
        .execute(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
