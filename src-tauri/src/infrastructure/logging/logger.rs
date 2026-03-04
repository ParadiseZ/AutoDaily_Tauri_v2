use crate::infrastructure::core::time_format::LocalTimer;
use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::log_error::{LogError, LogResult};
use crate::infrastructure::logging::log_trait::{Log, LogTrait};
use crate::infrastructure::path_resolve::model_path::PathUtil;
use chrono::Local;
use std::path::PathBuf;
use std::sync::Arc;
use bincode::{Decode, Encode};
use lazy_static::lazy_static;
use tauri::path::BaseDirectory;
use tokio::sync::{Mutex, RwLock};
use tracing::subscriber::set_global_default;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, reload, Registry};

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, ts_rs::TS)]
#[ts(export)]
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

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warn",
            LogLevel::Error => "Error",
            LogLevel::Off => "Off",
        };
        write!(f, "{}", s)
    }
}

lazy_static! {
    /// 日志级别 reload handle（用于动态调整主进程日志级别）
    pub static ref LOG_LEVEL_HANDLE: Mutex<Option<reload::Handle<LevelFilter, Registry>>> = Mutex::new(None);
    /// 当前日志目录（可动态修改）
    pub static ref LOG_DIR: RwLock<PathBuf> = RwLock::new(PathBuf::new());
}

/// 将 LogLevel 转换为 tracing 的 LevelFilter
pub fn parse_log_level(level: &LogLevel) -> LevelFilter {
    match level {
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Off => LevelFilter::OFF,
    }
}

impl LogMain {
    /// 动态更新主进程日志级别
    pub async fn update_level(level: &LogLevel) -> LogResult<()> {
        let level_filter = parse_log_level(level);

        if let Some(handle) = LOG_LEVEL_HANDLE.lock().await.as_ref() {
            handle
                .reload(level_filter)
                .map_err(|e| LogError::ReloadFilterErr { e: e.to_string() })?;
            Log::info(format!("主线程日志级别变更为: {:?}", level).as_ref());
        } else {
            return Err(LogError::ReloadDataNotInit);
        }

        Ok(())
    }

    /// 获取当前日志目录
    pub async fn get_log_dir() -> PathBuf {
        LOG_DIR.read().await.clone()
    }

    /// 更新日志目录（运行时动态切换）
    pub async fn update_log_dir(new_dir: &str) -> LogResult<()> {
        let new_path = PathUtil::get_absolute_path(new_dir, BaseDirectory::AppLog)
            .map_err(|e| LogError::CreateOrGet { e: e.to_string() })?;

        // 确保目录存在
        if !new_path.exists() {
            std::fs::create_dir_all(&new_path)
                .map_err(|e| LogError::CreateOrGet { e: e.to_string() })?;
        }

        let mut log_dir = LOG_DIR.write().await;
        *log_dir = new_path;

        Log::info(&format!("日志目录变更为: {}", new_dir));
        Ok(())
    }
}

impl LogMain {
    /// 初始化主进程日志系统
    pub async fn init(conf: LogMain, app_name: &str) -> LogResult<Self> {
        let log_level_filter = parse_log_level(&conf.log_level);

        // 解析并确保日志目录存在
        let log_dir_path: PathBuf =
            PathUtil::get_absolute_path(&conf.log_dir, BaseDirectory::AppLog)
                .map_err(|e| LogError::CreateOrGet { e: e.to_string() })?;

        if !log_dir_path.exists() {
            std::fs::create_dir_all(&log_dir_path)
                .map_err(|e| LogError::CreateOrGet { e: e.to_string() })?;
        }

        // 存储日志目录
        {
            let mut log_dir = LOG_DIR.write().await;
            *log_dir = log_dir_path.clone();
        }

        let date_str = Local::now().format("%y%m%d").to_string();
        let log_file = format!("{}_{}.log",app_name, date_str);

        // 创建文件日志 appender（按天滚动）
        let file_appender = RollingFileAppender::new(
            Rotation::NEVER,
            &log_dir_path,
            log_file,
        );

        // 创建可 reload 的级别过滤器
        let (filter, reload_handle) = reload::Layer::new(log_level_filter);

        // 存储 reload handle
        let mut guard = LOG_LEVEL_HANDLE.lock().await;
        *guard = Some(reload_handle);

        // 文件日志 layer
        let file_layer = fmt::Layer::new()
            .with_writer(file_appender)
            .with_timer(LocalTimer::DayStamp)
            .with_ansi(false)
            .with_target(true);

        // 控制台日志 layer
        let stdout_layer = fmt::Layer::new()
            .with_writer(std::io::stdout)
            .with_timer(LocalTimer::DayStamp)
            .with_ansi(true)
            .with_target(true);

        // 组合 subscriber
        let subscriber = Registry::default()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer);

        set_global_default(subscriber).map_err(|e| LogError::SetRegistryErr { e: e.to_string() })?;

        // 记录启动日志
        //tracing::info!("===== {} 启动 =====", app_name);
        tracing::info!("level: {:?}, Dir: {}", log_level_filter, log_dir_path.display());

        // 启动日志清理器
        if conf.retention_days > 0 {
            let cleaner_dir = log_dir_path.clone();
            let days = conf.retention_days;
            tokio::spawn(async move {
                crate::infrastructure::logging::log_cleaner::LogCleaner::start(cleaner_dir, days).await;
            });
        }

        Ok(conf)
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

impl LogTrait for LogMain {
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
