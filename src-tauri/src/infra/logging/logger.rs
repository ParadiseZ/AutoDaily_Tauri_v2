//! Root-process tracing logger.
use crate::infra::logging::LogLevel;
use crate::infra::logging::config::LogMain;
use crate::infra::logging::local_timer::LocalTimer;
use crate::infra::logging::log_error::{LogError, LogResult};
use crate::infra::logging::log_trait::{Log, LogTrait};
use chrono::Local;
use lazy_static::lazy_static;
use std::path::PathBuf;
use tauri::Manager;
use tauri::path::BaseDirectory;
use tokio::sync::{Mutex, RwLock};
use tracing::subscriber::set_global_default;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Registry, fmt, reload};

lazy_static! {
    /// 日志级别 reload handle（用于动态调整主进程日志级别）
    pub(crate) static ref LOG_LEVEL_HANDLE: Mutex<Option<reload::Handle<LevelFilter, Registry>>> = Mutex::new(None);
    /// 当前日志目录（可动态修改）
    pub(crate) static ref LOG_DIR: RwLock<PathBuf> = RwLock::new(PathBuf::new());
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

fn resolve_log_dir(target_dir: &str) -> LogResult<PathBuf> {
    let path = if target_dir.contains('/') || target_dir.contains('\\') {
        PathBuf::from(target_dir)
    } else {
        crate::infra::app_handle::get_app_handle()
            .path()
            .resolve(target_dir, BaseDirectory::AppLog)
            .map_err(|error| LogError::CreateOrGet {
                e: error.to_string(),
            })?
    };
    std::fs::create_dir_all(&path).map_err(|error| LogError::CreateOrGet {
        e: error.to_string(),
    })?;
    Ok(path)
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
        let new_path = resolve_log_dir(new_dir)?;

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
        let log_dir_path = resolve_log_dir(&conf.log_dir)?;

        // 存储日志目录
        {
            let mut log_dir = LOG_DIR.write().await;
            *log_dir = log_dir_path.clone();
        }

        let date_str = Local::now().format("%y%m%d").to_string();
        let log_file = format!("{}_{}.log", app_name, date_str);

        // 创建文件日志 appender（按天滚动）
        let file_appender = RollingFileAppender::new(Rotation::NEVER, &log_dir_path, log_file);

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
            .with_target(false);

        // 控制台日志 layer
        let stdout_layer = fmt::Layer::new()
            .with_writer(std::io::stdout)
            .with_timer(LocalTimer::DayStamp)
            .with_ansi(true)
            .with_target(false);

        // 组合 subscriber
        let subscriber = Registry::default()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer);

        set_global_default(subscriber)
            .map_err(|e| LogError::SetRegistryErr { e: e.to_string() })?;

        // 记录启动日志
        //tracing::info!("===== {} 启动 =====", app_name);
        tracing::info!(
            "level: {:?}, Dir: {}",
            log_level_filter,
            log_dir_path.display()
        );

        // 启动日志清理器
        if conf.retention_days > 0 {
            let cleaner_dir = log_dir_path.clone();
            let days = conf.retention_days;
            tokio::spawn(async move {
                crate::infra::logging::log_cleaner::LogCleaner::start(cleaner_dir, days).await;
            });
        }

        Ok(conf)
    }
}

impl LogTrait for LogMain {
    fn is_debug_enabled(&self) -> bool {
        tracing::enabled!(tracing::Level::DEBUG)
    }

    fn is_info_enabled(&self) -> bool {
        tracing::enabled!(tracing::Level::INFO)
    }

    fn is_warn_enabled(&self) -> bool {
        tracing::enabled!(tracing::Level::WARN)
    }

    fn is_error_enabled(&self) -> bool {
        tracing::enabled!(tracing::Level::ERROR)
    }

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
