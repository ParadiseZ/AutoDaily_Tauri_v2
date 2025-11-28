use crate::infrastructure::config::conf_write_guard::ConfigCategory;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfig {
    pub desktop_notice: bool,
    pub email_notification: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub recipient: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            desktop_notice: true,
            email_notification: false,
            smtp_server: String::new(),
            smtp_port: 465,
            username: String::new(),
            password: String::new(),
            recipient: String::new(),
        }
    }
}
