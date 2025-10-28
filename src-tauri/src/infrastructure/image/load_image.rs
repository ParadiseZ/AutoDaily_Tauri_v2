use std::path::Path;
use image::DynamicImage;
use crate::infrastructure::image::img_error::{ImageError, ImageResult};

pub fn load_img_from_path(img_path :&str) ->ImageResult<DynamicImage>{
    Ok(image::open(Path::new(img_path)).map_err(|e| ImageError::LoadFromLocalFailed{path: img_path.to_string(), e: e.to_string()})?)
}