#[derive(Error, Debug,Serialize,Deserialize)]
pub enum LogError {
    #[error("获取日志设置失败: {e}")]
    GetLogConfErr {e: String },

    #[error("创建/获取日志目录失败: {e}")]
    CreateOrGet {e: String },
    
    #[error("获取日志级别数据锁失败: {e}")]
    LockLevelFilterErr {e: String },

    #[error("重载日志级别失败: {e}")]
    ReloadFilterErr { e:String },

    #[error("日志级别数据LOG_LEVEL_HANDLE未初始化")]
    ReloadDataNotInit,

    #[error("设置日志订阅者失败{path}失败: {e}")]
    SetRegistryErr
}

pub type LogResult<T> = Result<T, LogError>;

pub use LogError::*;
use crate::infrastructure::core::{Deserialize, Error, Serialize};