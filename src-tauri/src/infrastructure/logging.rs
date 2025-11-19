pub mod logger;         // 日志记录器
pub mod config;         // 日志配置
pub mod handler;        // 日志处理器
//pub mod batch;
pub mod child_log;
pub mod main_process_log_handler;
pub mod log_error;
pub mod log_trait;
// 批量日志处理

//pub use handler::{ConsoleHandler, FileHandler, LogHandler};
// 重新导出主要类型
pub use logger::LogLevel;

