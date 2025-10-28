use crate::domain::entities::app_result::AppResult;
use crate::domain::manager::conf_mgr::ConfigManager;
use crate::domain::trait_ad::config_category::ConfigCategory;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use crate::domain::app_handle::get_app_handle;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    #[serde(rename = "warning")] // 特殊变体单独处理
    Warn,
    Error,
    Off
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub log_base: LogBase,
    pub log_level: LogLevel,
    pub retention_days: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogBase {
    pub log_dir: String,
    pub max_file_size: usize, // KB
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogC {
    pub log_base: LogBase, // KB
    pub log_level: LogLevel,
    pub write_file : bool,
    pub write_ui : bool,
    pub write_ui_mask : bool
}

impl LogBase{
    fn new() -> Self{
        Self{
            log_dir : get_app_handle().path().app_log_dir().unwrap().join("logs").to_string_lossy().into(),
            max_file_size : 10240
        }
    }
}

impl LogC{
    fn new() -> Self{
        Self{
            log_base : LogBase::new(),
            log_level : LogLevel::Off,
            write_file : false,
            write_ui : false,
            write_ui_mask : false
        }
    }
}

impl ConfigCategory for Log{
    fn default() -> Self {
        Self{
            log_base : LogBase::new(),
            log_level : LogLevel::Off,
            retention_days: 7
        }
    }
}
#[async_trait]
pub trait Logger {
    async fn init(mgr : ConfigManager, app_name: &str) -> AppResult<()>;
    fn info(msg: &str);
    fn warn(msg: &str);
    fn error(msg: &str);
    fn debug(msg: &str);
    /// 记录带有额外字段的信息日志
    fn info_with_fields(message: &str, fields: Vec<(&str, String)>);
    /// 记录函数开始执行
    fn fn_begin(function_name: &str);

    /// 记录函数执行结束
    fn fn_end(function_name: &str);

    /// 记录带有标签的信息日志
    fn info_with_tag(tag: &str, message: &str);

}