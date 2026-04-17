use crate::infrastructure::logging::log_trait::Log;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::Store;

#[allow(dead_code)]
pub async fn before_exit(app_handle: &AppHandle) {
    if let Err(e) = app_handle.state::<Store<Wry>>().save() {
        Log::error(&format!("保存配置文件失败：{}", e.to_string()))
    }
}
