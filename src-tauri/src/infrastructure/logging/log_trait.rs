use std::sync::OnceLock;
use crate::infrastructure::context::init_error::{InitError, InitResult};

/// 全局日志
static LOGGER: OnceLock<Box<dyn LogTrait + Send + Sync>> = OnceLock::new();

pub trait LogTrait {
    fn debug(msg: &str);
    fn info(msg: &str);
    fn warn(msg: &str);
    fn error(msg: &str);
}

pub struct Log;

impl Log{
    pub fn init_logger(log: Box<dyn LogTrait + Send + Sync>) -> InitResult<()> {

        LOGGER.set(log).map_err(|e| InitError::InitLoggerFailed {e: e.to_string()})
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