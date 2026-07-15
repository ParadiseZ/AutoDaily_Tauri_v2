use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EmailProviderPreset {
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "163")]
    NetEase163,
    #[serde(rename = "qq")]
    Qq,
    #[serde(rename = "gmail")]
    Gmail,
    #[serde(rename = "outlook")]
    Outlook,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EmailSecurity {
    TlsWrapper,
    StartTls,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSmtpServer {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub security: EmailSecurity,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailConfig {
    pub desktop_notice: bool,
    pub email_notification: bool,
    pub provider: EmailProviderPreset,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub security: EmailSecurity,
    pub username: String,
    pub password: String,
    pub sender_name: String,
    pub sender_email: String,
    pub recipient: String,
    pub timeout_seconds: u64,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            desktop_notice: true,
            email_notification: false,
            provider: EmailProviderPreset::Custom,
            smtp_server: String::new(),
            smtp_port: 465,
            security: EmailSecurity::TlsWrapper,
            username: String::new(),
            password: String::new(),
            sender_name: "AutoDaily".to_string(),
            sender_email: String::new(),
            recipient: String::new(),
            timeout_seconds: 60,
        }
    }
}

impl Default for EmailProviderPreset {
    fn default() -> Self {
        Self::Custom
    }
}

impl Default for EmailSecurity {
    fn default() -> Self {
        Self::TlsWrapper
    }
}

impl EmailProviderPreset {
    pub(crate) fn preset_server(&self) -> Option<ResolvedSmtpServer> {
        let (smtp_server, smtp_port, security) = match self {
            Self::Custom => return None,
            Self::NetEase163 => ("smtp.163.com", 465, EmailSecurity::TlsWrapper),
            Self::Qq => ("smtp.qq.com", 465, EmailSecurity::TlsWrapper),
            Self::Gmail => ("smtp.gmail.com", 465, EmailSecurity::TlsWrapper),
            Self::Outlook => ("smtp-mail.outlook.com", 587, EmailSecurity::StartTls),
        };
        Some(ResolvedSmtpServer {
            smtp_server: smtp_server.to_string(),
            smtp_port,
            security,
        })
    }
}

impl EmailConfig {
    pub fn resolved_server(&self) -> ResolvedSmtpServer {
        self.provider
            .preset_server()
            .unwrap_or_else(|| ResolvedSmtpServer {
                smtp_server: self.smtp_server.trim().to_string(),
                smtp_port: self.smtp_port,
                security: self.security.clone(),
            })
    }

    pub fn sender_email_value(&self) -> String {
        let sender = self.sender_email.trim();
        if sender.is_empty() {
            self.username.trim().to_string()
        } else {
            sender.to_string()
        }
    }

    pub fn sender_name_value(&self) -> Option<String> {
        let sender_name = self.sender_name.trim();
        (!sender_name.is_empty()).then(|| sender_name.to_string())
    }

    pub fn recipient_list(&self) -> Vec<String> {
        self.recipient
            .split([',', ';', '\n'])
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub fn timeout_seconds_value(&self) -> u64 {
        self.timeout_seconds.max(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_builtin_provider_profile() {
        let config = EmailConfig {
            provider: EmailProviderPreset::Outlook,
            ..EmailConfig::default()
        };
        let resolved = config.resolved_server();
        assert_eq!(resolved.smtp_server, "smtp-mail.outlook.com");
        assert_eq!(resolved.smtp_port, 587);
        assert_eq!(resolved.security, EmailSecurity::StartTls);
    }

    #[test]
    fn splits_multiple_recipients() {
        let config = EmailConfig {
            recipient: "a@example.com; b@example.com,\n c@example.com ".to_string(),
            ..EmailConfig::default()
        };
        assert_eq!(
            config.recipient_list(),
            vec!["a@example.com", "b@example.com", "c@example.com"]
        );
    }
}
