use std::fmt::format;
use crate::constant::sys_conf_path::{DEVICES_CONFIG_PATH, EMAIL_CONFIG_PATH, LOG_CONFIG_PATH, SCRIPTS_CONFIG_PATH, SYSTEM_SETTINGS_PATH};
use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::domain::config::sys_conf::SystemConfig;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::devices::device_conf::DeviceConfMap;
use crate::infrastructure::logging::config::LogMain;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use tracing::trace;
use crate::domain::config::notice_conf::EmailConfig;
use crate::infrastructure::logging::log_trait::Log;

pub async fn init_conf_sync(conf_mgr : &State<ConfigManager>) -> InitResult<()>{
    //系统设置
    conf_mgr.init_category::<SystemConfig>(SYSTEM_SETTINGS_PATH,BaseDirectory::AppConfig).await
        .map_err(|e| InitError::InitMainConfigErr{ category:"系统-系统设置".into(), e: e.to_string()  })?;
    //日志设置
    if let Err(err) = LogMain::init(conf_mgr, "AutoDaily").await {
        tracing::error!("初始化日志系统失败: {}", err);
        //std::process::exit(1);
    }
    Log::init_logger(Box::new(LogMain));
    Ok(())
}
pub fn init_conf_async(conf_mgr : &State<ConfigManager>){
    tokio::spawn(async move {
        // 设备设置
        if let Err(e) = conf_mgr.init_category::<DeviceConfMap>(DEVICES_CONFIG_PATH,BaseDirectory::AppConfig).await {
            Log::error(&format!("系统-设备设置初始化失败:{}",e));
        }
        // 脚本设置
        if let Err(e) = conf_mgr.init_category::<ScriptsConfig>(SCRIPTS_CONFIG_PATH,BaseDirectory::AppConfig).await {
            Log::error(&format!("系统-脚本设置初始化失败:{}",e));
        }
        // 通知设置
        if let Err(e) = conf_mgr.init_category::<EmailConfig>(EMAIL_CONFIG_PATH,BaseDirectory::AppConfig).await {
            Log::error(&format!("系统-邮件设置初始化失败:{}",e));
        }
    });
}
