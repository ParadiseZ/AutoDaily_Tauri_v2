use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub(crate) enum OrtError {
    #[error("{method} 模型加载错误: {e}")]
    LoadModelErr { method: String, e: String },
}
