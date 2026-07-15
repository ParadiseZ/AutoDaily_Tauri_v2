use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptProfile, ScriptTaskProfile,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRes {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    pub username: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendApiRes<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenReq {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}
use domain_script::{SCRIPT_RUNTIME_SCHEMA, ScriptInfo, supported_script_features};

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

pub fn apply_current_client_capability(script: &mut ScriptInfo) {
    let client = current_client_capability();
    script.min_app_version = Some(client.app_version);
    script.min_runtime_schema = Some(client.runtime_schema);
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
    pub version_num: Option<u32>,
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
    pub script: ScriptProfile,
    pub policies: Vec<PolicyProfile>,
    pub tasks: Vec<ScriptTaskProfile>,
    pub policy_groups: Vec<PolicyGroupProfile>,
    pub policy_sets: Vec<PolicySetProfile>,
    pub group_policies: Vec<PolicyGroupPolicyLink>,
    pub set_groups: Vec<PolicySetGroupLink>,
    #[serde(default)]
    pub model_files: Vec<ScriptModelFileDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameReq {
    pub new_username: String,
}
