use crate::app::app_error::{AppError, AppResult};
use crate::constant::sys_conf_path::{APP_STORE, EMAIL_CONFIG_KEY};
use crate::domain::config::notice_conf::EmailConfig;
use runtime_engine::infrastructure::mail::{send_email, EmailMessagePayload};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub async fn get_email_config_app(app_handle: &AppHandle) -> AppResult<EmailConfig> {
    let store = app_handle.store(APP_STORE).map_err(|e| AppError::SetConfigFailed {
        detail: "读取邮件配置失败".to_string(),
        e: e.to_string(),
    })?;

    Ok(store
        .get(EMAIL_CONFIG_KEY)
        .and_then(|value| serde_json::from_value::<EmailConfig>(value.clone()).ok())
        .unwrap_or_default())
}

pub async fn set_email_config_app(
    app_handle: &AppHandle,
    config: &EmailConfig,
) -> AppResult<()> {
    let store = app_handle.store(APP_STORE).map_err(|e| AppError::SetConfigFailed {
        detail: "写入邮件配置失败".to_string(),
        e: e.to_string(),
    })?;

    let normalized = normalize_email_config(config.clone());
    let value = serde_json::to_value(&normalized).map_err(|e| AppError::SerializeConfErr {
        detail: "邮件配置".to_string(),
        e: e.to_string(),
    })?;

    store.set(EMAIL_CONFIG_KEY, value);
    store.save().map_err(|e| AppError::SetConfigFailed {
        detail: "持久化邮件配置失败".to_string(),
        e: e.to_string(),
    })?;

    Ok(())
}

pub async fn send_test_email_app(config: &EmailConfig) -> AppResult<()> {
    let normalized = normalize_email_config(config.clone());
    let resolved = normalized.resolved_server();
    let payload = EmailMessagePayload {
        subject: "AutoDaily SMTP 测试邮件".to_string(),
        body: format!(
            "这是一封来自 AutoDaily 的 SMTP 测试邮件。\n\nSMTP 服务: {}\n端口: {}\n加密方式: {:?}\n发送时间: {}",
            resolved.smtp_server,
            resolved.smtp_port,
            resolved.security,
            chrono::Local::now().to_rfc3339(),
        ),
    };

    send_email(&normalized, &payload)
        .await
        .map_err(|e| AppError::SetConfigFailed {
            detail: "发送测试邮件失败".to_string(),
            e,
        })
}

fn normalize_email_config(mut config: EmailConfig) -> EmailConfig {
    let resolved = config.resolved_server();
    config.smtp_server = resolved.smtp_server;
    config.smtp_port = resolved.smtp_port;
    config.security = resolved.security;
    config
}
