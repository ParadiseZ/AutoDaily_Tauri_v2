use crate::app::app_error::AppResult;
use crate::infrastructure::core::{Deserialize, Serialize};

// 响应结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}


impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success<T: Serialize>(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error<T: Serialize>(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

impl<T> From<AppResult<T>> for ApiResponse<T>
where
    T: Serialize,
{
    fn from(result: AppResult<T>) -> Self {
        match result {
            Ok(data) => ApiResponse::success(data),
            Err(error) => ApiResponse::error(error.to_string()),
        }
    }
}