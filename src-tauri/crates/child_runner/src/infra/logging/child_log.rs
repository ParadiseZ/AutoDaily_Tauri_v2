use crate::infra::context::runtime_control::get_ipc_client;
use crate::infra::ipc::channel_client::IpcClient;
use crate::infra::logging::LogLevel;
use crate::infra::logging::log_trait::LogTrait;
use runner_protocol::message::{IpcMessage, LogMessage, MessagePayload, MessageType};
use std::sync::atomic::Ordering;

pub(crate) struct LogChild;
impl LogTrait for LogChild {
    fn debug(&self, msg: &str) {
        if let Some(client) = get_ipc_client() {
            client.debug(msg);
        }
    }
    fn info(&self, msg: &str) {
        if let Some(client) = get_ipc_client() {
            client.info(msg);
        }
    }
    fn warn(&self, msg: &str) {
        if let Some(client) = get_ipc_client() {
            client.warn(msg);
        }
    }
    fn error(&self, msg: &str) {
        if let Some(client) = get_ipc_client() {
            client.error(msg);
        }
    }
}

impl IpcClient {
    pub(crate) fn should_log(&self, level: LogLevel) -> bool {
        level as u8 >= self.log_level.load(Ordering::Acquire)
    }
    pub(crate) fn create_logger_and_send(&self, log_level: LogLevel, msg: &str) {
        self.send_uncertain(IpcMessage::new(
            *self.device_id,
            MessageType::Logger,
            MessagePayload::Logger(LogMessage {
                level: log_level,
                message: msg.to_string(),
                module: None,
            }),
        ));
    }
    pub(crate) fn debug(&self, msg: &str) {
        if !self.should_log(LogLevel::Debug) {
            return;
        }
        self.create_logger_and_send(LogLevel::Debug, msg);
    }
    pub(crate) fn info(&self, msg: &str) {
        if !self.should_log(LogLevel::Info) {
            return;
        }
        self.create_logger_and_send(LogLevel::Info, msg);
    }
    pub(crate) fn warn(&self, msg: &str) {
        if !self.should_log(LogLevel::Warn) {
            return;
        }
        self.create_logger_and_send(LogLevel::Warn, msg);
    }
    pub(crate) fn error(&self, msg: &str) {
        if !self.should_log(LogLevel::Error) {
            return;
        }
        self.create_logger_and_send(LogLevel::Error, msg);
    }
}
