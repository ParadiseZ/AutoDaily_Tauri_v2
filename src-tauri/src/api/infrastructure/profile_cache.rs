use crate::infrastructure::http_client::HttpClient;
use crate::constant::sys_conf_path::APP_STORE;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub const USER_PROFILE_CACHE_KEY: &str = "user_profile_cache";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedUserProfile {
    username: String,
    profile: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct CurrentAuthenticatedUser {
    pub id: Option<String>,
    pub username: String,
}

pub fn load_cached_user_profile(app_handle: &AppHandle, username: &str) -> Option<serde_json::Value> {
    let store = app_handle.store(APP_STORE).ok()?;
    let cached = store
        .get(USER_PROFILE_CACHE_KEY)
        .and_then(|value| serde_json::from_value::<CachedUserProfile>(value.clone()).ok())?;
    if cached.username.trim() != username.trim() {
        return None;
    }
    Some(cached.profile)
}

pub fn load_cached_profile_for_current_session(app_handle: &AppHandle) -> Option<serde_json::Value> {
    let client = HttpClient::new(app_handle.clone());
    let session = client.get_auth_session()?;
    load_cached_user_profile(app_handle, &session.username)
}

pub fn persist_cached_user_profile(
    app_handle: &AppHandle,
    username: &str,
    profile: &serde_json::Value,
) {
    let Ok(store) = app_handle.store(APP_STORE) else {
        return;
    };

    let payload = CachedUserProfile {
        username: username.to_string(),
        profile: profile.clone(),
    };

    if let Ok(value) = serde_json::to_value(payload) {
        store.set(USER_PROFILE_CACHE_KEY, value);
    }
}

pub fn clear_cached_user_profile(app_handle: &AppHandle) {
    if let Ok(store) = app_handle.store(APP_STORE) {
        store.delete(USER_PROFILE_CACHE_KEY);
    }
}

pub fn should_use_cached_profile(code: i32, message: &str) -> bool {
    if code == 503 {
        return true;
    }

    !(code == 401
        || code == 403
        || message.contains("401")
        || message.contains("Unauthorized")
        || message.contains("未登录")
        || message.contains("认证失败"))
}

pub fn load_current_authenticated_user(app_handle: &AppHandle) -> Option<CurrentAuthenticatedUser> {
    let client = HttpClient::new(app_handle.clone());
    let session = client.get_auth_session()?;
    let username = session.username.trim();
    if username.is_empty() {
        return None;
    }

    let id = load_cached_user_profile(app_handle, username)
        .as_ref()
        .and_then(|profile| profile.get("id"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    Some(CurrentAuthenticatedUser {
        id,
        username: username.to_string(),
    })
}
