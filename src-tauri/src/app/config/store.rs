use crate::infra::logging::log_trait::Log;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tauri::Wry;
use tauri_plugin_store::Store;

pub fn get_or_init_config<T: Serialize + DeserializeOwned + Clone + Default>(
    store: Arc<Store<Wry>>,
    key: &str,
) -> T {
    if let Some(value) = store.get(key) {
        if let Ok(config) = serde_json::from_value(value.clone()) {
            return config;
        }
        Log::error(&format!("配置 {key} 解析失败，使用默认值"));
    } else {
        Log::info(&format!("配置 {key} 不存在，初始化默认值"));
    }
    let default = T::default();
    store.set(key, serde_json::to_value(&default).unwrap_or_default());
    default
}
