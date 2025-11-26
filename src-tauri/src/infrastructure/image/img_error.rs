#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ImageError {
    #[error("裁剪图像失败，源：{detail}, 错误：{e}")]
    CropErr { detail: String, e: String },

    #[error("从{path}加载图像失败：{e}")]
    LoadFromLocalFailed { path: String, e: String },
}

pub type ImageResult<T> = Result<T, ImageError>;

use crate::infrastructure::core::{Deserialize, Error, Serialize};
pub use ImageError::*;
