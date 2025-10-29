//! Infrastructure Process模块
//! 
//! 提供进程管理功能，包括：
//! - 进程生命周期管理
//! - 子进程启动和监控
//! - 应用程序句柄管理

pub mod process;
pub mod app_handle;

// 重新导出主要类型
pub use process::*;
pub use app_handle::*;