use std::fs;
use std::path::PathBuf;
use chrono::Local;
use once_cell::sync::Lazy;
use ort::execution_providers::set_gpu_device;
use tauri::{Manager, State};
use tauri::path::BaseDirectory;
use tokio::sync::Mutex;
use tracing::subscriber::set_global_default;
use tracing::trace;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{fmt, reload, Registry};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::layer::SubscriberExt;
use crate::app::app_error::{AppError, AppResult};
use crate::constant::sys_conf_path::LOG_CONFIG_PATH;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::core::time_format::LocalTimer;
use crate::infrastructure::logging::config::{LogMain};
use crate::infrastructure::logging::log_error::{LogError, LogResult};
use crate::infrastructure::logging::log_trait::{Log, LogTrait};
use crate::infrastructure::path_resolve::model_path::PathUtil;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}
impl From<u8> for LogLevel {
    fn from(v: u8) -> Self {
        match v {
            1 => LogLevel::Debug,
            2 => LogLevel::Info,
            3 => LogLevel::Warn,
            4 => LogLevel::Error,
            _ => LogLevel::Off,
        }
    }
}

static LOG_LEVEL_HANDLE: Lazy<Mutex<Option<reload::Handle<LevelFilter, Registry>>>> =
    Lazy::new(|| Mutex::new(None));
// Convert string log level to LevelFilter
pub fn parse_log_level(level: &LogLevel) -> LevelFilter {
    match level {
        //Logl => LevelFilter::TRACE,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Off => LevelFilter::OFF
    }
}

impl LogMain{
    pub fn update_level(level: &LogLevel) -> LogResult<()> {
        let level_filter = parse_log_level(level);

        // Safely handle the mutex and update the log level
        let handle_opt = match LOG_LEVEL_HANDLE.lock() {
            Ok(guard) => guard,
            Err(poison_err) => {
                // 尝试从有毒的互斥锁中恢复数据
                return Err(LogError::LockLevelFilterErr{e : poison_err});
            }
        };

        if let Some(handle) = handle_opt.as_ref() {
            handle
                .reload(level_filter)
                .map_err(|e| LogError::ReloadFilterErr{e})?;
            Log::info(format!("主线程日志级别变更为: {:?}", level));
        } else {
            return Err(LogError::ReloadDataNotInit);
        }

        Ok(())
    }
}

impl LogMain{
    pub async fn init(mgr: &State<ConfigManager>, app_name: &str) -> LogResult<()> {
        // 只初始化日志配置，其他配置可以异步加载
        if let Err(e) = mgr.init_category::<LogMain>(LOG_CONFIG_PATH,BaseDirectory::AppConfig).await{
            tracing::error!("Failed to init logging config: {}", e);
        }
        // 获取日志配置并立即初始化日志系统
        let conf = mgr.get_conf::<LogMain>(LOG_CONFIG_PATH).await
            .map_err(|e| LogError::GetLogConfErr{e: e.to_string()})?;

        let log_level_filter = parse_log_level(&conf.log_level);

        // Resolve and ensure log directory
        let log_dir_path: PathBuf = PathUtil::get_absolute_path(&conf.log_dir, BaseDirectory::AppLog)
            .map_err(|e| LogError::CreateOrGet{e: e.to_string()})?;

        let date_str = Local::now().format("%y%m%d").to_string();
        let log_file = format!("AutoDaily_{}.log", date_str);

        // 添加调试信息
        /*        println!("主程序日志目录: {}", log_dir_path.display());
                println!("日志文件: {}", log_file);
                println!("日志级别: {:?}", log_level_filter);*/

        // Create a rolling file appender that creates a new file each day
        let file_appender = RollingFileAppender::new(
            Rotation::NEVER, // Create new file daily
            &log_dir_path,
            log_file,
        );

        // Create a reloadable filter
        let (filter, reload_handle) = reload::Layer::new(log_level_filter);

        // Store the reload handle for later use - safely handle the mutex
        match LOG_LEVEL_HANDLE.lock() {
            Ok(mut handle) => *handle = Some(reload_handle),
            Err(e) => return Err(LogError::LockLevelFilterErr{e}),
        }

        // Set up a single layer for file logging
        let file_layer = fmt::Layer::new()
            .with_writer(file_appender)
            .with_timer(LocalTimer::DayStamp)
            .with_ansi(false)
            .with_target(true);

        // Set up a layer for console output
        let stdout_layer = fmt::Layer::new()
            .with_writer(std::io::stdout)
            .with_timer(LocalTimer::DayStamp)
            .with_ansi(true)
            .with_target(true);

        // Create the subscriber with both layers and the reloadable filter
        let subscriber = Registry::default()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer);

        set_global_default(subscriber)
            .map_err(|e| LogError::SetRegistryErr)?;

        // Record startup log
        tracing::info!("===== {} 启动 =====", app_name);
        tracing::info!("Log level set to: {:?}", log_level_filter);

        Ok(())
    }
    

    /// 记录带有额外字段的信息日志
    pub fn info_with_fields(message: &str, fields: Vec<(&str, String)>) {
        let span = tracing::info_span!("", message = message);
        let _enter = span.enter();

        for (key, value) in fields {
            tracing::info!(key = key, value = value);
        }
    }

    /// 记录函数开始执行
    pub fn fn_begin(function_name: &str) {
        tracing::debug!("开始执行: {}", function_name);
    }

    /// 记录函数执行结束
    pub fn fn_end(function_name: &str) {
        tracing::debug!("执行结束: {}", function_name);
    }

    /// 记录带有标签的信息日志
    pub fn info_with_tag(tag: &str, message: &str) {
        tracing::info!(tag = tag, "{}", message);
    }

}

impl LogTrait for LogMain{
    fn debug(&self, msg: &str) {
        tracing::debug!("{}", msg);
    }

    fn info(&self, msg: &str) {
        tracing::info!("{}", msg);
    }

    fn warn(&self, msg: &str) {
        tracing::warn!("{}", msg);
    }

    fn error(&self, msg: &str) {
        tracing::error!("{}", msg);
    }
}