pub(crate) use infra_mail::{EmailMessagePayload, send_email};

use domain_notification::EmailConfig;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::app::constants::{APP_STORE, EMAIL_CONFIG_KEY};
use crate::infra::logging::log_trait::Log;

pub fn load_email_config(app_handle: &AppHandle) -> Result<EmailConfig, String> {
    let store = app_handle
        .store(APP_STORE)
        .map_err(|error| format!("读取邮件配置失败: {}", error))?;
    Ok(store
        .get(EMAIL_CONFIG_KEY)
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default())
}

pub async fn send_stored_timeout_email(
    app_handle: &AppHandle,
    payload: &EmailMessagePayload,
) -> Result<(), String> {
    let config = load_email_config(app_handle)?;
    if config.email_notification {
        send_email(&config, payload).await?;
    }
    Ok(())
}

pub fn send_timeout_email_in_background(app_handle: AppHandle, payload: EmailMessagePayload) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = send_stored_timeout_email(&app_handle, &payload).await {
            Log::warn(&format!("[ email ] 超时邮件发送失败: {}", error));
        }
    });
}
