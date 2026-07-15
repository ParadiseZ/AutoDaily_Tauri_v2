use domain_notification::{EmailConfig, EmailSecurity};
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct EmailMessagePayload {
    pub subject: String,
    pub body: String,
}

pub async fn send_email(config: &EmailConfig, payload: &EmailMessagePayload) -> Result<(), String> {
    let server = config.resolved_server();
    let host = server.smtp_server.trim();
    if host.is_empty() {
        return Err("SMTP 服务器不能为空".to_string());
    }
    if server.smtp_port == 0 {
        return Err("SMTP 端口必须大于 0".to_string());
    }
    validate_smtp_host(host, &server.security)?;
    let username = config.username.trim();
    if username.is_empty() {
        return Err("SMTP 用户名不能为空".to_string());
    }
    let password = config.password.trim();
    if password.is_empty() {
        return Err("SMTP 密码不能为空".to_string());
    }
    let sender = config.sender_email_value();
    if sender.is_empty() {
        return Err("发件人邮箱不能为空".to_string());
    }
    let recipients = config.recipient_list();
    if recipients.is_empty() {
        return Err("收件人不能为空".to_string());
    }
    if payload.subject.trim().is_empty() {
        return Err("邮件主题不能为空".to_string());
    }

    let mut builder = Message::builder()
        .from(mailbox(config.sender_name_value(), &sender, "发件人")?)
        .subject(payload.subject.trim());
    for recipient in &recipients {
        builder = builder.to(mailbox(None, recipient, "收件人")?);
    }
    let message = builder
        .header(ContentType::TEXT_PLAIN)
        .body(payload.body.clone())
        .map_err(|error| format!("构建邮件内容失败: {}", error))?;
    transport(
        host,
        server.smtp_port,
        &server.security,
        Credentials::new(username.to_string(), password.to_string()),
        Some(Duration::from_secs(config.timeout_seconds_value())),
    )?
    .send(message)
    .await
    .map_err(|error| format!("SMTP 发送失败: {}", error))?;
    Ok(())
}

fn transport(
    host: &str,
    port: u16,
    security: &EmailSecurity,
    credentials: Credentials,
    timeout: Option<Duration>,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    use lettre::transport::smtp::client::{Tls, TlsParameters};
    let tls = match security {
        EmailSecurity::TlsWrapper => Tls::Wrapper(
            TlsParameters::new(host.to_string())
                .map_err(|error| format!("SMTP TLS 参数无效: {}", error))?,
        ),
        EmailSecurity::StartTls => Tls::Required(
            TlsParameters::new(host.to_string())
                .map_err(|error| format!("SMTP STARTTLS 参数无效: {}", error))?,
        ),
        EmailSecurity::None => Tls::None,
    };
    Ok(
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
            .port(port)
            .tls(tls)
            .credentials(credentials)
            .timeout(timeout)
            .build(),
    )
}

fn validate_smtp_host(host: &str, security: &EmailSecurity) -> Result<(), String> {
    if matches!(security, EmailSecurity::None) {
        return Ok(());
    }
    lettre::transport::smtp::client::TlsParameters::new(host.to_string())
        .map(|_| ())
        .map_err(|_| format!("SMTP 服务器[{host}]不是可用于 TLS 证书校验的有效域名"))
}

fn mailbox(name: Option<String>, email: &str, label: &str) -> Result<Mailbox, String> {
    match name {
        Some(name) => format!("{} <{}>", name, email),
        None => email.to_string(),
    }
    .parse::<Mailbox>()
    .map_err(|error| format!("{}格式无效: {}", label, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_missing_smtp_host_before_connecting() {
        let result = send_email(
            &EmailConfig::default(),
            &EmailMessagePayload {
                subject: "test".into(),
                body: String::new(),
            },
        )
        .await;
        assert!(matches!(result, Err(message) if message == "SMTP 服务器不能为空"));
    }
}
