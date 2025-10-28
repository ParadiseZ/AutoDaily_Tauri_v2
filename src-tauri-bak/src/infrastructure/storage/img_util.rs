use crate::domain::entities::app_result::{AppError, AppResult};
use image::{DynamicImage, RgbaImage};
use std::path::Path;

pub fn load_img_from_path(img_path :&str) ->AppResult<DynamicImage>{
    let img = image::open(Path::new(img_path)).map_err(|e| AppError::IoError(format!("从{}加载图片失败：{}", img_path,e)))?;
    Ok(img)
}