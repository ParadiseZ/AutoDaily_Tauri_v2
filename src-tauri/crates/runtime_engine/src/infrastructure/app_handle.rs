use tauri::AppHandle;

pub(crate) static GLOBAL_APP_HANDLE: tokio::sync::OnceCell<AppHandle> =
    tokio::sync::OnceCell::const_new();

pub fn get_app_handle() -> &'static AppHandle {
    GLOBAL_APP_HANDLE.get().unwrap()
}

pub fn init_app_handle(app_handle: &AppHandle) {
    GLOBAL_APP_HANDLE.set(app_handle.clone()).unwrap();
}
