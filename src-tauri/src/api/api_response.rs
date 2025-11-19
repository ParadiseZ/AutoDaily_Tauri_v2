use crate::app::app_error::AppResult;
use crate::infrastructure::core::{Deserialize, Serialize};

// 响应结构
#[derive(Debug, Serialize, Deserialize)]
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
    pub fn ok<T: Serialize>(msg: Option<String>) -> Self {
        Self::success(None,msg)
    }
    pub fn error<T: Serialize>(msg: Option<String>) -> Self {
        Self::failed(None,msg)
    }

    pub fn success<T: Serialize>(data: T,message: Option<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
        }
    }

    pub fn failed<T: Serialize>(data: T, message: Option<String>) -> Self {
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
            Ok(data) => ApiResponse::ok(data),
            Err(error) => ApiResponse::error(Some(error.to_string())),
        }
    }
}