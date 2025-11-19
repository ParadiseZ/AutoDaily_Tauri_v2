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
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::logging::log_trait::Log;

pub async fn init_conf_sync(conf_mgr : &State<'_, ConfigManager>) -> InitResult<()>{
    //系统设置
    conf_mgr.init_category::<SystemConfig>(SYSTEM_SETTINGS_PATH,BaseDirectory::AppConfig).await
        .map_err(|e| InitError::InitMainConfigErr{ category:"系统-系统设置".into(), e: e.to_string()  })?;
    //日志设置
    match LogMain::init(conf_mgr, "AutoDaily").await  {
        Ok(conf) => {
            if let Err(e) = Log::init_logger(Box::new(conf)){
                tracing::error!("初始化日志系统失败: {}", e);
                //std::process::exit(1);
            };
        },
        Err(err) => {
            tracing::error!("初始化日志系统失败: {}", err);
            //std::process::exit(1);
        }
    }

    Ok(())
}
pub fn init_conf_async(){
    tokio::spawn(async move {
        let conf_mgr = get_app_handle().state::<ConfigManager>();
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
