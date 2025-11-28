#[derive(Error, Debug, Serialize, Deserialize, Decode, Encode)]
pub enum CoreError {
    #[error("获取可用核心失败: {e}")]
    AffinityMaskErr { e: String },

    #[error("锁已中毒（数据可能不一致）: {e}")]
    LockPoisoned { e: String },
}

pub type CoreResult<T> = Result<T, CoreError>;

use crate::infrastructure::core::{Deserialize, Error, Serialize};
use bincode::{Decode, Encode};