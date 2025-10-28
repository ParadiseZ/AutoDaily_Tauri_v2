use crate::infrastructure::ipc::chanel_client::IpcClient;
use crate::infrastructure::ipc::message::{IpcMessage, LogMessage, MessagePayload, MessageType};
use crate::infrastructure::logging::LogLevel;
use std::sync::atomic::Ordering;
use crate::infrastructure::context::child_process_sec::get_ipc_client;
use crate::infrastructure::logging::log_trait::{Log, LogTrait};

pub struct LogChild;
impl LogTrait for LogChild{
    fn debug(msg : &str){
        get_ipc_client().debug(msg);
    }
    fn info(msg : &str){
        get_ipc_client().info(msg);
    }
    fn warn(msg : &str){
        get_ipc_client().warn(msg);
    }
    fn error(msg : &str){
        get_ipc_client().error(msg)
    }
}
impl IpcClient{
    pub fn set_log_level(&self, level: LogLevel) {
        self.log_level.store(level as u8, Ordering::Relaxed);
    }

    pub fn should_log(&self, level: LogLevel) -> bool {
        level as u8 >= self.log_level.load(Ordering::Acquire)
    }
    pub fn create_logger_and_send(&self, log_level: LogLevel,msg: &str){
        self.send(
            IpcMessage::new(
                *self.device_id,
                MessageType::Logger,
                MessagePayload::Logger(
                    LogMessage {
                        level:log_level,
                        message: msg.into_string(),
                        module: None,
                    }
                ))
        );
    }
    pub fn debug(&self, msg: &str) {
        if !self.should_log(LogLevel::Debug) {
            return;
        }
        self.create_logger_and_send(LogLevel::Debug, msg);
    }
    pub fn info(&self, msg: &str) {
        if !self.should_log(LogLevel::Info) {
            return;
        }
       self.create_logger_and_send(LogLevel::Info, msg);
    }
    pub fn warn(&self, msg: &str) {
        if !self.should_log(LogLevel::Warn) {
            return;
        }
        self.create_logger_and_send(LogLevel::Warn, msg);
    }
    pub fn error(&self, msg: &str) {
        if !self.should_log(LogLevel::Error) {
            return;
        }
        self.create_logger_and_send(LogLevel::Error, msg);
    }
}