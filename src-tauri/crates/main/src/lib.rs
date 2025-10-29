//! # AutoDaily Main Library
//! 
//! 主库，整合所有模块并提供统一入口

// 重新导出所有子模块
pub use ad_core::*;
pub use ad_domain::*;
pub use ad_app::*;
pub use ad_api::*;

pub use ad_infra_core::*;
pub use ad_infra_device::*;
pub use ad_infra_ipc::*;
pub use ad_infra_process::*;
pub use ad_infra_script::*;
pub use ad_infra_vision::*;

// 主运行函数
pub use ad_api::run;
