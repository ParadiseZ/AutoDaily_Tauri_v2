use lazy_static::lazy_static;
use thiserror::Error;
use tokio::sync::OnceCell;

lazy_static! {
    pub static ref LOGGER: OnceCell<Box<dyn LogTrait>> = OnceCell::new();
}

pub trait LogTrait: Send + Sync {
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
}

#[derive(Debug, Error)]
pub enum LoggerInitError {
    #[error("初始化全局 logger 失败: {e}")]
    AlreadyInitialized { e: String },
}

pub type LoggerInitResult<T> = Result<T, LoggerInitError>;

pub struct Log;

impl Log {
    pub fn init_logger(log: Box<dyn LogTrait>) -> LoggerInitResult<()> {
        LOGGER
            .set(log)
            .map_err(|e| LoggerInitError::AlreadyInitialized { e: e.to_string() })
    }

    pub fn debug(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.debug(msg);
        }
    }

    pub fn info(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.info(msg);
        }
    }

    pub fn warn(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.warn(msg);
        }
    }

    pub fn error(msg: &str) {
        if let Some(log) = LOGGER.get() {
            log.error(msg);
        }
    }
}
