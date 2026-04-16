use crate::app::config::email_conf::{
    get_email_config_app, send_test_email_app, set_email_config_app,
};
use crate::domain::config::notice_conf::EmailConfig;
use tauri::{command, AppHandle};

#[command]
pub async fn get_email_config_cmd(app_handle: AppHandle) -> Result<EmailConfig, String> {
    get_email_config_app(&app_handle)
        .await
        .map_err(|e| format!("读取邮件配置失败: {}", e))
}

#[command]
pub async fn set_email_config_cmd(
    app_handle: AppHandle,
    config: EmailConfig,
) -> Result<String, String> {
    set_email_config_app(&app_handle, &config)
        .await
        .map_err(|e| format!("保存邮件配置失败: {}", e))?;
    Ok("邮件配置已保存".to_string())
}

#[command]
pub async fn send_test_email_cmd(config: EmailConfig) -> Result<String, String> {
    send_test_email_app(&config)
        .await
        .map_err(|e| format!("测试邮件发送失败: {}", e))?;
    Ok("测试邮件已发送，请检查收件箱".to_string())
}
