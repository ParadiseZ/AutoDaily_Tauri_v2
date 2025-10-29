//! Infrastructure Device模块
//! 
//! 提供设备管理功能，包括：
//! - 设备连接管理
//! - ADB交互
//! - 屏幕捕获

pub mod devices;
pub mod adb_cli_local;
pub mod capture;

// 重新导出主要类型
pub use devices::*;
pub use adb_cli_local::*;
pub use capture::*;