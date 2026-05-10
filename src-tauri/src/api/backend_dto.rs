use crate::infrastructure::core::{Deserialize, Serialize};
pub use runtime_engine::api::backend_dto::{AuthRes, BackendApiRes};
use runtime_engine::domain::scripts::script_info::{
    supported_script_features, SCRIPT_RUNTIME_SCHEMA,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapability {
    pub app_version: String,
    pub runtime_schema: u32,
    pub supported_features: Vec<String>,
}

pub fn current_client_capability() -> ClientCapability {
    ClientCapability {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        runtime_schema: SCRIPT_RUNTIME_SCHEMA,
        supported_features: supported_script_features(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterReq {
    pub username: String,
    pub password: String,
    pub email: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordReq {
    pub email: String,
    pub new_password: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptSearchReq {
    pub page: i64,
    pub size: i64,
    pub keyword: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "runtimeType")]
    pub runtime_type: Option<String>,
    pub client: Option<ClientCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDownloadReq {
    pub client: ClientCapability,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptModelFileDto {
    pub script_id: Option<String>,
    pub version_num: Option<u64>,
    pub runtime_type: String,
    pub r#type: String,
    pub file_name: String,
    pub download_path: String,
    pub size_bytes: Option<u64>,
    pub hash_algorithm: Option<String>,
    pub hash_value: Option<String>,
    pub etag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SponsorRedeemReq {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRes<T> {
    pub records: Vec<T>,
    pub total: i64,
    pub size: i64,
    pub current: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptUploadRequest {
    pub script: crate::domain::scripts::script_info::ScriptTable,
    pub policies: Vec<crate::domain::scripts::policy::PolicyTable>,
    pub tasks: Vec<crate::domain::scripts::script_task::ScriptTaskTable>,
    pub policy_groups: Vec<crate::domain::scripts::policy::PolicyGroupTable>,
    pub policy_sets: Vec<crate::domain::scripts::policy::PolicySetTable>,
    pub group_policies: Vec<crate::domain::scripts::policy::GroupPolicyRelation>,
    pub set_groups: Vec<crate::domain::scripts::policy::SetGroupRelation>,
    #[serde(default)]
    pub model_files: Vec<ScriptModelFileDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameReq {
    pub new_username: String,
}
