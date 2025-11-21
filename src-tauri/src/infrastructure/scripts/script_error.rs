#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ScriptError {
    #[error("从{path}加载脚本信息失败： {e}")]
    LoadFromFileErr { path: String, e: String },

    #[error("从缓存加载脚本(id:{script_id})失败")]
    LoadFromCacheErr { script_id: ScriptId },
}

pub type ScriptResult<T> = Result<T, ScriptError>;

use crate::infrastructure::core::{Deserialize, Error, ScriptId, Serialize};
pub use ScriptError::*;
