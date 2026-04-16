use crate::domain::config::notice_conf::{EmailConfig, EmailSecurity};
use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::constant::sys_conf_path::{APP_STORE, EMAIL_CONFIG_KEY};
use crate::infrastructure::logging::log_trait::Log;

#[derive(Clone, Debug)]
pub struct EmailMessagePayload {
    pub subject: String,
    pub body: String,
}

pub fn load_email_config(app_handle: &AppHandle) -> Result<EmailConfig, String> {
    let store = app_handle
        .store(APP_STORE)
        .map_err(|e| format!("读取邮件配置失败: {}", e))?;

    Ok(store
        .get(EMAIL_CONFIG_KEY)
        .and_then(|value| serde_json::from_value::<EmailConfig>(value.clone()).ok())
        .unwrap_or_default())
}

pub async fn send_email(
    config: &EmailConfig,
    payload: &EmailMessagePayload,
) -> Result<(), String> {
    let resolved_server = config.resolved_server();
    let smtp_server = resolved_server.smtp_server.trim().to_string();
    if smtp_server.is_empty() {
        return Err("SMTP 服务器不能为空".to_string());
    }

    if resolved_server.smtp_port == 0 {
        return Err("SMTP 端口必须大于 0".to_string());
    }

    validate_smtp_host_for_security(&smtp_server, &resolved_server.security)?;

    let username = config.username.trim().to_string();
    if username.is_empty() {
        return Err("SMTP 用户名不能为空".to_string());
    }

    let password = config.password.trim().to_string();
    if password.is_empty() {
        return Err("SMTP 密码不能为空".to_string());
    }

    let sender_email = config.sender_email_value();
    if sender_email.is_empty() {
        return Err("发件人邮箱不能为空".to_string());
    }

    let recipients = config.recipient_list();
    if recipients.is_empty() {
        return Err("收件人不能为空".to_string());
    }

    if payload.subject.trim().is_empty() {
        return Err("邮件主题不能为空".to_string());
    }

    let from_mailbox = build_mailbox(config.sender_name_value(), &sender_email, "发件人")?;
    let mut builder = Message::builder().from(from_mailbox).subject(payload.subject.trim());
    for recipient in &recipients {
        builder = builder.to(build_mailbox(None, recipient, "收件人")?);
    }

    let email = builder
        .header(ContentType::TEXT_PLAIN)
        .body(payload.body.clone())
        .map_err(|e| format!("构建邮件内容失败: {}", e))?;

    let credentials = Credentials::new(username, password);
    let timeout = Some(Duration::from_secs(config.timeout_seconds_value()));

    let transport = build_transport(
        &smtp_server,
        resolved_server.smtp_port,
        &resolved_server.security,
        credentials,
        timeout,
    )?;

    transport
        .send(email)
        .await
        .map_err(|e| format!("SMTP 发送失败: {}", e))?;

    Ok(())
}

pub async fn send_stored_timeout_email(
    app_handle: &AppHandle,
    payload: &EmailMessagePayload,
) -> Result<(), String> {
    let config = load_email_config(app_handle)?;
    if !config.email_notification {
        return Ok(());
    }

    send_email(&config, payload).await
}

fn build_transport(
    smtp_server: &str,
    smtp_port: u16,
    security: &EmailSecurity,
    credentials: Credentials,
    timeout: Option<Duration>,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    use lettre::transport::smtp::client::{Tls, TlsParameters};

    let tls = match security {
        EmailSecurity::TlsWrapper => Tls::Wrapper(
            TlsParameters::new(smtp_server.to_string())
                .map_err(|e| format!("SMTP TLS 参数无效: {}", e))?,
        ),
        EmailSecurity::StartTls => Tls::Required(
            TlsParameters::new(smtp_server.to_string())
                .map_err(|e| format!("SMTP STARTTLS 参数无效: {}", e))?,
        ),
        EmailSecurity::None => Tls::None,
    };

    Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(smtp_server)
        .port(smtp_port)
        .tls(tls)
        .credentials(credentials)
        .timeout(timeout)
        .build())
}

fn validate_smtp_host_for_security(
    smtp_server: &str,
    security: &EmailSecurity,
) -> Result<(), String> {
    let host = smtp_server.trim();
    if host.is_empty() {
        return Err("SMTP 服务器不能为空".to_string());
    }

    if matches!(security, EmailSecurity::None) {
        return Ok(());
    }

    lettre::transport::smtp::client::TlsParameters::new(host.to_string())
        .map(|_| ())
        .map_err(|_| format!("SMTP 服务器[{host}]不是可用于 TLS 证书校验的有效域名"))
}

fn build_mailbox(
    display_name: Option<String>,
    email: &str,
    field_label: &str,
) -> Result<Mailbox, String> {
    let mailbox = match display_name {
        Some(name) => format!("{} <{}>", name, email),
        None => email.to_string(),
    };

    mailbox
        .parse::<Mailbox>()
        .map_err(|e| format!("{}格式无效: {}", field_label, e))
}

pub fn send_timeout_email_in_background(
    app_handle: AppHandle,
    payload: EmailMessagePayload,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = send_stored_timeout_email(&app_handle, &payload).await {
            Log::warn(&format!("[ email ] 超时邮件发送失败: {}", error));
        }
    });
}
