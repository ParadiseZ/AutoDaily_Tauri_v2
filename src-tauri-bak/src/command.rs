pub mod dev_test;
pub mod config;

// 重新导出所有命令函数，便于在lib.rs中统一注册
pub use dev_test::*;
pub use config::*;