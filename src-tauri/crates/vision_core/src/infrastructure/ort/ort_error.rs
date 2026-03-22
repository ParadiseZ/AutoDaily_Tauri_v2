use thiserror::Error;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum OrtError {
    #[error("{method} 模型加载错误: {e}")]
    LoadModelErr { method: String, e: String },
}
