use std::fs;
use std::path::PathBuf;
use chrono::Local;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri::path::BaseDirectory;
use tauri::{Manager};
use tracing::subscriber::set_global_default;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{self, time::FormatTime},
    prelude::*,
    reload, Registry,
};
use crate::constant::sys_conf_path::LOG_CONFIG_PATH;
use crate::domain::app_handle::get_app_handle;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, LogLevel, Logger};
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager};
use crate::domain::utils::path_utils::{create_dirs, get_absolute_path};

// Global handle for reloading the log filter
static LOG_HANDLE: Lazy<Mutex<Option<reload::Handle<LevelFilter, Registry>>>> =
    Lazy::new(|| Mutex::new(None));

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}
// Convert string log level to LevelFilter
pub fn parse_log_level(level: &LogLevel) -> LevelFilter {
    match level {
        //Logl => LevelFilter::TRACE,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::OFF => LevelFilter::OFF
    }
}

impl Log{
    fn update_level(level: &LogLevel) -> AppResult<()> {
        let level_filter = parse_log_level(level);

        // Safely handle the mutex and update the log level
        let handle_opt = match LOG_HANDLE.lock() {
            Ok(guard) => guard,
            Err(poison_err) => {
                // 尝试从有毒的互斥锁中恢复数据
                return Err(AppError::LoggingError(format!(
                    "无法锁定日志级别句柄，互斥锁已被污染: {}",
                    poison_err
                )));
            }
        };

        if let Some(handle) = handle_opt.as_ref() {
            handle
                .reload(level_filter)
                .map_err(|e| AppError::LoggingError(format!("重载日志级别失败: {}", e)))?;
            tracing::info!("Log level updated to: {:?}", level);
        } else {
            return Err(AppError::LoggingError("日志系统未初始化".to_string()));
        }

        Ok(())
    }
}

impl Logger for Log{
    async fn init(mgr: ConfigManager, app_name: &str) -> AppResult<()> {
        // 只初始化日志配置，其他配置可以异步加载
        if let Err(e) = mgr.init_category::<Log>(LOG_CONFIG_PATH,BaseDirectory::AppConfig).await{
            eprintln!("Failed to init logging config: {}", e);
        }
        // 获取日志配置并立即初始化日志系统
        let conf = mgr.get_conf::<Log>(LOG_CONFIG_PATH).await?;

        let log_level_filter = parse_log_level(&conf.log_level);

        // Resolve and ensure log directory
        let log_dir_path: PathBuf = get_absolute_path(&conf.log_dir, BaseDirectory::AppConfig);
        
        create_dirs(&log_dir_path)?;

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
        match LOG_HANDLE.lock() {
            Ok(mut handle) => *handle = Some(reload_handle),
            Err(e) => return Err(AppError::LoggingError(format!("无法锁定日志句柄: {}", e))),
        }

        // Set up a single layer for file logging
        let file_layer = fmt::Layer::new()
            .with_writer(file_appender)
            .with_timer(LocalTimer)
            .with_ansi(false)
            .with_target(true);

        // Set up a layer for console output
        let stdout_layer = fmt::Layer::new()
            .with_writer(std::io::stdout)
            .with_timer(LocalTimer)
            .with_ansi(true)
            .with_target(true);

        // Create the subscriber with both layers and the reloadable filter
        let subscriber = Registry::default()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer);

        set_global_default(subscriber)
            .map_err(|e| AppError::LoggingError(format!("无法设置全局日志订阅者: {}", e)))?;

        // Record startup log
        tracing::info!("===== {} 启动 =====", app_name);
        tracing::info!("Log level set to: {:?}", log_level_filter);

        Ok(())
    }
    

    fn info(msg: &str) {
        tracing::info!("{}", msg);
    }

    fn warn(msg: &str) {
        tracing::warn!("{}", msg);
    }

    fn error(msg: &str) {
        tracing::error!("{}", msg);
    }

    fn debug(msg: &str) {
        tracing::debug!("{}", msg);
    }

    /// 记录带有额外字段的信息日志
    fn info_with_fields(message: &str, fields: Vec<(&str, String)>) {
        let span = tracing::info_span!("", message = message);
        let _enter = span.enter();

        for (key, value) in fields {
            tracing::info!(key = key, value = value);
        }
    }

    /// 记录函数开始执行
    fn fn_begin(function_name: &str) {
        tracing::debug!("开始执行: {}", function_name);
    }

    /// 记录函数执行结束
    fn fn_end(function_name: &str) {
        tracing::debug!("执行结束: {}", function_name);
    }

    /// 记录带有标签的信息日志
    fn info_with_tag(tag: &str, message: &str) {
        tracing::info!(tag = tag, "{}", message);
    }

}