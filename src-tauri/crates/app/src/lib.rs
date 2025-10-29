//! # AutoDaily App Module
//! 
//! 应用模块提供应用服务和业务流程

pub mod services;
pub mod workflows;
pub mod managers;

// 重新导出主要类型
pub use services::*;
pub use workflows::*;
pub use managers::*;