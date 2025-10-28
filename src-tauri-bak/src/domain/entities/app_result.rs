use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AppError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("IO错误: {0}")]
    IoError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("数据处理错误: {0}")]
    DataProcessingError(String),

    #[error("认证错误: {0}")]
    AuthError(String),

    #[error("未知错误: {0}")]
    UnknownError(String),

    #[error("截图失败: {0}")]
    CaptureError(String),

    #[error("加载模型失败: {0}")]
    LoadModelError(String),

    #[error("输入内容无效: {0}")]
    InvalidInputError(String),

    #[error("推理失败: {0}")]
    InferenceError(String),

    #[error("裁剪图像失败: {0}")]
    CropCaptureError(String),

    #[error("日志系统错误: {0}")]
    LoggingError(String),

    #[error("设备错误: {0}")]
    DeviceError(String),

    #[error("系统错误: {0}")]
    SystemError(String),

    #[error("线程池错误: {0}")]
    ThreadPoolError(String),

    #[error("内部错误: {0}")]
    InternalError(String)
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        AppError::ConfigError(error.to_string())
    }
}

// 可以为其他常用错误类型实现类似的 From trait

pub type AppResult<T> = Result<T, AppError>;

// 添加辅助函数
impl AppError {
    pub fn to_json_response(&self) -> String {
        serde_json::json!({
            "success": false,
            "error": {
                "type": match self {
                    AppError::ConfigError(_) => "config_error",
                    AppError::IoError(_) => "io_error",
                    AppError::NetworkError(_) => "network_error",
                    AppError::DataProcessingError(_) => "data_processing_error",
                    AppError::AuthError(_) => "auth_error",
                    AppError::UnknownError(_) => "unknown_error",
                    AppError::CaptureError(_) => "capture_error",
                    AppError::LoadModelError(_) => "load_model_error",
                    AppError::InvalidInputError(_) => "invalid_input_error",
                    AppError::InferenceError(_) => "inference_error",
                    AppError::CropCaptureError(_) => "crop_capture_error",
                    AppError::LoggingError(_) => "logging_error",
                    AppError::DeviceError(_) => "device_error",
                    AppError::SystemError(_) => "system_error",
                    AppError::ThreadPoolError(_) => "thread_pool_error",
                    AppError::InternalError(_) => "internal_error"
                },
                "message": self.to_string(),
            }
        })
            .to_string()
    }
}
