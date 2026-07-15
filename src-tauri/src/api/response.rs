//! Root API response envelope.
use crate::app::app_error::AppResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub details: Option<Value>,
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
            details: None,
        }
    }

    pub fn success(data: Option<T>, message: Option<String>) -> Self {
        Self {
            success: true,
            data,
            message,
            details: None,
        }
    }

    pub fn failed(data: Option<T>, message: Option<String>) -> Self {
        Self {
            success: false,
            data,
            message,
            details: None,
        }
    }

    pub fn failed_with_details(
        data: Option<T>,
        message: Option<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            success: false,
            data,
            message,
            details,
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
