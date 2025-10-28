use tauri::AppHandle;

pub(crate) static GLOBAL_APP_HANDLE: tokio::sync::OnceCell<AppHandle> = tokio::sync::OnceCell::const_new();

pub fn get_app_handle() -> &'static AppHandle{
    GLOBAL_APP_HANDLE.get().unwrap()
}
