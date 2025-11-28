use crate::infrastructure::core::Serialize;
use crate::infrastructure::logging::log_trait::Log;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tauri::Wry;
use tauri_plugin_store::Store;

/// 安全获取配置，失败时自动回填默认值
pub fn get_or_init_config<T: Serialize + DeserializeOwned + Clone + Default>(
    store: Arc<Store<Wry>>,
    key: &str
) -> T {
    // 尝试获取并解析
    if let Some(value) = store.get(key) {
        if let Ok(config) = serde_json::from_value(value.clone()) {
            return config;
        }
        Log::error("配置 {key} 解析失败，使用默认值");
    } else {
        Log::info("配置 {key} 不存在，初始化默认值");
    }
    let default = T::default();
    // 回填默认值（失败仅记录）
    store.set(key, serde_json::to_value(&default).unwrap_or_default());
    default
}