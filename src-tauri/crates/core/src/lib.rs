//! # AutoDaily Core Module
//! 
//! 核心模块提供CPU核心分配、进程管理等基础功能

pub mod cpu;
pub mod process;
pub mod memory;

// 重新导出主要类型
pub use cpu::*;
pub use process::*;
pub use memory::*;

// 全局类型别名
pub type DeviceId = uuid::Uuid;
pub type ProcessId = uuid::Uuid;
pub type ScriptId = uuid::Uuid;