use crate::constant::sys_conf_path::APP_STORE;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::logging::log_trait::Log;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[allow(dead_code)]
pub async fn before_exit(app_handle: &AppHandle) {
    if let Some(manager) = get_process_manager() {
        manager.stop_all().await;
    }

    match app_handle.store(APP_STORE) {
        Ok(store) => {
            if let Err(e) = store.save() {
                Log::error(&format!("保存配置文件失败：{}", e));
            }
        }
        Err(e) => {
            Log::error(&format!("获取配置文件失败：{}", e));
        }
    }
}
