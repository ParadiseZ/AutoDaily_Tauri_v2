#[derive(Error, Debug,Serialize,Deserialize, Decode, Encode)]
pub enum CoreError {
    #[error("获取可用核心失败: {e}")]
    AffinityMaskErr { e: String },

    #[error("锁已中毒（数据可能不一致）: {0}")]
    LockPoisoned{ e: String },
}


// 统一处理读锁错误
pub fn read_lock<T>(lock: &Arc<RwLock<T>>) -> RwLockReadGuard<T> {
    lock.read()
        .unwrap_or_else(|e: PoisonError<_>| e.into_inner())
}

// 统一处理写锁错误
pub fn write_lock<T>(lock: &Arc<RwLock<T>>) -> RwLockWriteGuard<T> {
    lock.write()
        .unwrap_or_else(|e: PoisonError<_>| e.into_inner())
}

pub type CoreResult<T> = Result<T, CoreError>;

use std::sync::{Arc, PoisonError};
use bincode::{Decode, Encode};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use CoreError::*;
use crate::infrastructure::core::{Deserialize, Error, Serialize};