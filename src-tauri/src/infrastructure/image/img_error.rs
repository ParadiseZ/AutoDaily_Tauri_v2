#[derive(Error, Debug,Serialize,Deserialize)]
pub enum ImageError {
    #[error("裁剪图像失败，源：{detail}, 错误：{e}")]
    CropErr { detail:DetResult,e: String },

    #[error("从{path}加载图像失败：{e}")]
    LoadFromLocalFailed { path:String, e: String },
}

pub type ImageResult<T> = Result<T, ImageError>;

pub use ImageError::*;
use crate::domain::vision::result::DetResult;
use crate::infrastructure::core::{Deserialize, Error, Serialize};