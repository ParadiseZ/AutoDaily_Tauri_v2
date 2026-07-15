use crate::infra::image::SCREENSHOT_DIR;
use crate::infra::logging::log_trait::Log;
use base64::{Engine, engine::general_purpose};
use std::path::Path;
use tokio::fs;

pub async fn save_screenshot(
    image_data: &str,
    device_name: &str,
    image_type: &str,
) -> Result<String, String> {
    if !matches!(image_type, "window" | "adb") {
        return Err(format!("未知的截图方式: {}", image_type));
    }
    let result = save_base64_image(image_data, device_name).await;
    match &result {
        Ok(path) => Log::info(&format!("截图已保存到: {}", path)),
        Err(error) => Log::error(&format!("保存截图失败: {}", error)),
    }
    result
}

pub async fn save_base64_image(base64_image: &str, title: &str) -> Result<String, String> {
    fs::create_dir_all(SCREENSHOT_DIR)
        .await
        .map_err(|error| format!("创建目录失败: {}", error))?;
    let image = general_purpose::STANDARD
        .decode(base64_image)
        .map_err(|error| format!("base64解码失败: {}", error))?;
    let file_name = format!(
        "{}_{}.png",
        chrono::Local::now().format("%Y%m%d%H%M%S"),
        title
    );
    let path = Path::new(SCREENSHOT_DIR).join(file_name);
    fs::write(&path, image)
        .await
        .map_err(|error| format!("保存图像失败: {}", error))?;
    Ok(path.to_string_lossy().into_owned())
}
