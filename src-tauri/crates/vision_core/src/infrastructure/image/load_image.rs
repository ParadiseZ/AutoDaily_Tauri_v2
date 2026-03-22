use std::io::Cursor;
use crate::infrastructure::image::img_error::{ImageError, ImageResult};
use image::DynamicImage;
use std::path::Path;
use base64::Engine;
use base64::engine::general_purpose;
use crate::infrastructure::logging::log_trait::Log;

pub fn load_img_from_path(img_path: &str) -> ImageResult<DynamicImage> {
    Ok(
        image::open(Path::new(img_path)).map_err(|e| ImageError::LoadFromLocalFailed {
            path: img_path.to_string(),
            e: e.to_string(),
        })?,
    )
}

pub fn dynamic_image_to_base64(img: &DynamicImage) -> Result<String, String> {
    let mut cursor = Cursor::new(Vec::new());
    match DynamicImage::ImageRgba8(img.to_rgba8())
        .write_to(&mut cursor, image::ImageFormat::Png)
    {
        Ok(_) => {
            let buffer = cursor.into_inner();
            let base64_string = general_purpose::STANDARD.encode(&buffer);
            let msg = format!("转换base64编码截图成功：{}KB", base64_string.len() / 1024);
            Log::info(&msg);
            Ok(base64_string)
        }
        Err(e) => {
            Log::error(&format!("图像转换为base64失败: {:?}", e));
            Err("base64编码失败！".to_string())
        }
    }
}
