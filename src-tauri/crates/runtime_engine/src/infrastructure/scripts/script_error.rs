#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ScriptError {
    #[error("从{path}加载脚本信息失败： {e}")]
    LoadFromFileErr { path: String, e: String },

    #[error("从缓存加载脚本(id:{script_id})失败")]
    LoadFromCacheErr { script_id: ScriptId },

    #[error("脚本执行失败,类型:{step_type},错误:{e}")]
    ExecuteErr { step_type: String, e: String },
}

pub type ScriptResult<T> = Result<T, ScriptError>;

pub type ExecuteResult<T> = Result<T, ScriptError>;

use crate::infrastructure::core::{Deserialize, Error, ScriptId, Serialize};