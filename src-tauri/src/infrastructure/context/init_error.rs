#[derive(Error, Debug, Serialize, Deserialize)]
pub enum InitError {
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

    #[error("main-初始化{category}配置失败: {e}")]
    InitMainConfigErr { category: String, e: String },

    #[error("main-初始化脚本管理器失败: {e}")]
    InitMainScriptMgrErr { e: String },

    #[error("main-初始化设备{name}信息失败: {e}")]
    InitMainDevicesErr { device_name: String, e: String },

    #[error("main-初始化设备{device_name}日志级别失败: {e}")]
    InitMainLogLevels { device_name: String, e: String },

    #[error("main-初始化IPC通道{name}失败: {e}")]
    InitMainIpcServerErr { name: String, e: String },

    #[error("main-初始化消息处理器失败: {e}")]
    InitMainMsgHandlerErr { e: String },
}

pub type InitResult<T> = Result<T, InitError>;

use crate::infrastructure::core::{Deserialize, Error, Serialize};
pub use InitError::*;
