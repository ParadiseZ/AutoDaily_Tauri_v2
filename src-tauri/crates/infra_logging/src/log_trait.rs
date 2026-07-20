use lazy_static::lazy_static;
use thiserror::Error;
use tokio::sync::OnceCell;

lazy_static! {
    static ref LOGGER: OnceCell<Box<dyn LogTrait>> = OnceCell::new();
}

pub trait LogTrait: Send + Sync {
    fn is_debug_enabled(&self) -> bool;
    fn is_info_enabled(&self) -> bool;
    fn is_warn_enabled(&self) -> bool;
    fn is_error_enabled(&self) -> bool;
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}

#[derive(Debug, Error)]
pub enum LoggerInitError {
    #[error("初始化全局 logger 失败: {0}")]
    AlreadyInitialized(String),
}

pub type LoggerInitResult<T> = Result<T, LoggerInitError>;

pub struct Log;

impl Log {
    pub fn init_logger(log: Box<dyn LogTrait>) -> LoggerInitResult<()> {
        LOGGER
            .set(log)
            .map_err(|error| LoggerInitError::AlreadyInitialized(error.to_string()))
    }

    pub fn debug(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.debug(msg);
        }
    }
    pub fn debug_lazy(message: impl FnOnce() -> String) {
        if let Some(log) = LOGGER.get().filter(|log| log.is_debug_enabled()) {
            let message = message();
            log.debug(&message);
        }
    }
    pub fn info(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.info(msg);
        }
    }
    pub fn info_lazy(message: impl FnOnce() -> String) {
        if let Some(log) = LOGGER.get().filter(|log| log.is_info_enabled()) {
            let message = message();
            log.info(&message);
        }
    }
    pub fn warn(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.warn(msg);
        }
    }
    pub fn warn_lazy(message: impl FnOnce() -> String) {
        if let Some(log) = LOGGER.get().filter(|log| log.is_warn_enabled()) {
            let message = message();
            log.warn(&message);
        }
    }
    pub fn error(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.error(msg);
        }
    }
    pub fn error_lazy(message: impl FnOnce() -> String) {
        if let Some(log) = LOGGER.get().filter(|log| log.is_error_enabled()) {
            let message = message();
            log.error(&message);
        }
    }
}
