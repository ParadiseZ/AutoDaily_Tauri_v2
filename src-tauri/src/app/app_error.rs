// Root application error boundary.
use crate::infra::logging::log_error::LogError;
use domain_script::ScriptError;
use infra_vision::{ImageError, VisionError};
use runner_protocol::channel_error::ChannelError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppError {
    #[error("序列化配置{detail}失败{e}")]
    SerializeConfErr { detail: String, e: String },

    /*#[error(transparent)]
    CapErr(#[from] CapError),*/
    #[error(transparent)]
    ImageErr(#[from] ImageError),

    #[error(transparent)]
    ChannelErr(#[from] ChannelError),

    #[error(transparent)]
    LogErr(#[from] LogError),

    #[error(transparent)]
    ScriptErr(#[from] ScriptError),

    #[error(transparent)]
    VisionErr(#[from] VisionError),

    #[error("快捷键设置失败：{detail}, {e}")]
    ShortCutSetFailed { detail: String, e: String },

    #[error("设置系统设置失败：{detail}, {e}")]
    SetConfigFailed { detail: String, e: String },

    #[error("HTTP请求出错：{detail}, {e}")]
    HttpErr { detail: String, e: String },
}

pub type AppResult<T> = Result<T, AppError>;
