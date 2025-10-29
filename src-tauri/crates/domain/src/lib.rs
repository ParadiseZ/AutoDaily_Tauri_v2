//! # AutoDaily Domain Module
//! 
//! 领域模块包含业务实体和领域逻辑

pub mod config;
pub mod scripts;
pub mod vision;

// 重新导出主要类型pub use config::*;
pub use scripts::*;
pub use vision::*;
