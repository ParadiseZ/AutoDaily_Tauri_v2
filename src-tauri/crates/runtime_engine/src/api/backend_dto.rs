use crate::infrastructure::core::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenReq {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}
