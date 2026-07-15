#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
