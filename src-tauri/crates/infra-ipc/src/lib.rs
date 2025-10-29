//! Infrastructure IPC模块
//! 
//! 提供进程间通信功能，包括：
//! - IPC通信框架
//! - 共享内存管理
//! - 消息传递

pub mod ipc;
pub mod shared;

// 重新导出主要类型
pub use ipc::*;
pub use shared::*;