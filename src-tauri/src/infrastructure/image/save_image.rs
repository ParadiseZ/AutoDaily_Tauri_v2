use crate::constant::project::SCREENSHOT_DIR;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::path_resolve::model_path::PathUtil;
use base64::engine::general_purpose;
use base64::Engine;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 成功返回文件路径，失败返回错误信息
pub async fn save_screenshot(
    image_data: &str,
    device_name: &str,
    image_type: &str,
) -> Result<String, String> {
    // 根据图像类型选择处理方式
    let result = match image_type {
        "window" => {
            // 处理窗口截图（base64格式）
            save_base64_image(image_data, device_name).await
        }
        "adb" => {
            // 未来用于处理ADB截图
            save_base64_image(image_data, device_name).await
        }
        _ => {
            let error_msg = format!("未知的截图方式: {}", image_type);
            Log::error(&error_msg);
            Err(error_msg)
        }
    };

    match &result {
        Ok(path) => Log::info(&format!("截图已保存到: {}", path)),
        Err(e) => Log::error(&format!("保存截图失败: {}", e)),
    }

    result
}

/// 保存base64编码的图像到文件
///
/// 返回保存的文件路径或错误信息
pub async fn save_base64_image(base64_image: &str, title: &str) -> Result<String, String> {
    // 确保目录存在
    match PathUtil::sure_parent_exists(&PathBuf::from(SCREENSHOT_DIR)) {
        Ok(_) => Log::info(&format!("目录 {} 已确认可用", SCREENSHOT_DIR)),
        Err(e) => {
            let error_msg = format!("创建目录失败: {:?}", e);
            Log::error(&error_msg);
            return Err(error_msg);
        }
    }

    // 解码base64数据
    let image_data = match general_purpose::STANDARD.decode(base64_image) {
        Ok(data) => data,
        Err(e) => {
            let error_msg = format!("base64解码失败: {:?}", e);
            Log::error(&error_msg);
            return Err(error_msg);
        }
    };

    // 生成文件名和路径
    let filename = format!(
        "{}_{}.png",
        chrono::Local::now().format("%Y%m%d%H%M%S").to_string(),
        title
    );
    let file_path = Path::new(SCREENSHOT_DIR).join(filename);
    let path_str = file_path.to_string_lossy().to_string();

    // 保存图像
    match fs::write(&file_path, image_data).await {
        Ok(_) => {
            Log::info(&format!("图像已保存到: {}", path_str));
            Ok(path_str)
        }
        Err(e) => {
            Log::error(&format!("保存图像失败: {:?}", e));
            Err(e.to_string())
        }
    }
}
