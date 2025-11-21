pub mod config; // 日志配置
pub mod handler;
pub mod logger; // 日志记录器 // 日志处理器
                              //pub mod batch;
pub mod child_log;
pub mod log_error;
pub mod log_trait;
pub mod main_process_log_handler;
// 批量日志处理

//pub use handler::{ConsoleHandler, FileHandler, LogHandler};
// 重新导出主要类型
pub use logger::LogLevel;
