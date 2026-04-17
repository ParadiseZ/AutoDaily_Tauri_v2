use crate::app::config::vision_cache_conf::{
    get_vision_text_cache_config_app, set_vision_text_cache_config_app,
};
use crate::domain::config::vision_cache_conf::VisionTextCacheConfig;
use tauri::{command, AppHandle};

#[command]
pub async fn get_vision_text_cache_config_cmd(
    app_handle: AppHandle,
) -> Result<VisionTextCacheConfig, String> {
    get_vision_text_cache_config_app(&app_handle)
        .await
        .map_err(|e| format!("读取 OCR 文字缓存设置失败: {}", e))
}

#[command]
pub async fn set_vision_text_cache_config_cmd(
    app_handle: AppHandle,
    config: VisionTextCacheConfig,
) -> Result<String, String> {
    set_vision_text_cache_config_app(&app_handle, &config)
        .await
        .map_err(|e| format!("保存 OCR 文字缓存设置失败: {}", e))?;
    Ok("OCR 文字缓存设置已保存".to_string())
}
