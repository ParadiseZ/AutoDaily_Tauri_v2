use crate::infrastructure::config::conf_error::ConfigError;
use crate::infrastructure::context::init_error::InitError;
use crate::infrastructure::core::{Deserialize, Error, Serialize};
use crate::infrastructure::image::img_error::ImageError;
use crate::infrastructure::ipc::channel_error::ChannelError;
use crate::infrastructure::logging::log_error::LogError;
use crate::infrastructure::path_resolve::path_error::PathError;
use crate::infrastructure::scripts::script_error::ScriptError;
use crate::infrastructure::vision::vision_error::VisionError;

#[derive(Error, Debug, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppError {
    #[error("序列化配置{detail}失败{e}")]
    SerializeConfErr{detail: String, e: String},

    #[error(transparent)]
    ConfigErr(#[from] ConfigError),
    
    /*#[error(transparent)]
    CapErr(#[from] CapError),*/

    #[error(transparent)]
    ImageErr(#[from] ImageError),
    
    #[error(transparent)]
    InitErr(#[from] InitError),

    #[error(transparent)]
    ChannelErr(#[from] ChannelError),

    #[error(transparent)]
    LogErr(#[from] LogError),

    #[error(transparent)]
    PathErr(#[from] PathError),

    #[error(transparent)]
    ScriptErr(#[from] ScriptError),

    #[error(transparent)]
    VisionErr(#[from] VisionError),

    #[error("快捷键设置失败：{detail}, {e}")]
    ShortCutSetFailed{detail: String, e: String},
    
    #[error("设置系统设置失败：{detail}, {e}")]
    SetConfigFailed{detail: String, e: String},
}

pub type AppResult<T> = Result<T, AppError>;