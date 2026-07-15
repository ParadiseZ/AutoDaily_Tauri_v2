use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub(crate) enum ChildRuntimeInitError {
    #[error("child-初始化上下文失败: {e}")]
    InitChildAppCtxFailed { e: String },
    #[error("child-初始化adb上下文失败: {e}")]
    InitChildAdbCtxFailed { e: String },
    #[error("child初始化IPC客户端失败: {e}")]
    InitChildIpcClientFailed { e: String },
    #[error("child-初始化rayon线程池失败: {e}")]
    InitChildRayonPoolFailed { e: String },
    #[error("-初始化日志管理器失败: {e}")]
    InitLoggerFailed { e: String },
    #[error("child-初始化cpu亲和性失败: {e}")]
    InitChildCoreAffinity { e: String },
    #[error("child-初始化ort环境失败： {e}")]
    InitChildOrtEnvFailed { e: String },
    #[error("child-初始化数据库连接失败： {e}")]
    InitChildDatabaseEnvFailed { e: String },
}

pub(crate) type ChildRuntimeInitResult<T> = Result<T, ChildRuntimeInitError>;
