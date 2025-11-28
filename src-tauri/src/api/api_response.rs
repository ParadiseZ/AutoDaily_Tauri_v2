use crate::app::app_error::AppResult;
use crate::infrastructure::core::{Deserialize, Serialize};

// 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: Option<T>) -> Self {
        Self::success(data, None)
    }

    pub fn error(msg: Option<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: msg,
        }
    }

    pub fn success(data: Option<T>, message: Option<String>) -> Self {
        Self {
            success: true,
            data,
            message,
        }
    }

    pub fn failed(data: Option<T>, message: Option<String>) -> Self {
        Self {
            success: false,
            data,
            message,
        }
    }
}

impl<T> From<AppResult<T>> for ApiResponse<T>
where
    T: Serialize,
{
    fn from(result: AppResult<T>) -> Self {
        match result {
            Ok(data) => ApiResponse::ok(Some(data)),
            Err(error) => ApiResponse::error(Some(error.to_string())),
        }
    }
}
