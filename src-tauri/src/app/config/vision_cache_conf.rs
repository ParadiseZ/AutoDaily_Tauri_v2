use crate::app::app_error::{AppError, AppResult};
use crate::constant::sys_conf_path::{APP_STORE, VISION_TEXT_CACHE_CONFIG_KEY};
use crate::domain::config::vision_cache_conf::{
    VisionTextCacheConfig, VisionTextCacheRuntimeConfig,
};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

pub async fn get_vision_text_cache_config_app(
    app_handle: &AppHandle,
) -> AppResult<VisionTextCacheConfig> {
    let store = app_handle
        .store(APP_STORE)
        .map_err(|e| AppError::SetConfigFailed {
            detail: "读取 OCR 文字缓存设置失败".to_string(),
            e: e.to_string(),
        })?;

    Ok(store
        .get(VISION_TEXT_CACHE_CONFIG_KEY)
        .and_then(|value| serde_json::from_value::<VisionTextCacheConfig>(value.clone()).ok())
        .unwrap_or_default())
}

pub async fn set_vision_text_cache_config_app(
    app_handle: &AppHandle,
    config: &VisionTextCacheConfig,
) -> AppResult<()> {
    let store = app_handle
        .store(APP_STORE)
        .map_err(|e| AppError::SetConfigFailed {
            detail: "写入 OCR 文字缓存设置失败".to_string(),
            e: e.to_string(),
        })?;

    let value = serde_json::to_value(config).map_err(|e| AppError::SerializeConfErr {
        detail: "OCR 文字缓存设置".to_string(),
        e: e.to_string(),
    })?;

    store.set(VISION_TEXT_CACHE_CONFIG_KEY, value);
    store.save().map_err(|e| AppError::SetConfigFailed {
        detail: "持久化 OCR 文字缓存设置失败".to_string(),
        e: e.to_string(),
    })?;
    Ok(())
}

pub async fn get_vision_text_cache_runtime_config_app(
    app_handle: &AppHandle,
) -> AppResult<VisionTextCacheRuntimeConfig> {
    let persisted = get_vision_text_cache_config_app(app_handle).await?;
    let fallback_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::SetConfigFailed {
            detail: "获取 OCR 缓存默认目录失败".to_string(),
            e: e.to_string(),
        })?
        .join("ocr-text-cache");

    Ok(persisted.to_runtime_config(fallback_dir))
}
