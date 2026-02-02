use tauri::command;

/// 将本地图片转换为 base64 字符串
#[command]
pub async fn convert_img_to_base64_cmd(img_path: String) -> Result<String, String> {
    use crate::infrastructure::image::load_image::{load_img_from_path, dynamic_image_to_base64};
    let img = load_img_from_path(&img_path).map_err(|e| e.to_string())?;
    dynamic_image_to_base64(&img)
}