use crate::infrastructure::core::{Deserialize, Serialize};

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
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordReq {
    pub email: String,
    pub password: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub struct ScriptSearchReq {
    pub page: i64,
    pub size: i64,
    pub keyword: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "runtimeType")]
    pub runtime_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendApiRes<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SponsorRedeemReq {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TauriUpdateRes {
    pub version: String,
    pub notes: String,
    #[serde(rename = "pub_date")]
    pub pub_date: String,
    pub platforms: std::collections::HashMap<String, PlatformUpdateInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformUpdateInfo {
    pub signature: String,
    pub url: String,
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameReq {
    pub new_username: String,
}
