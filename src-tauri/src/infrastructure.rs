//核心
pub mod core;
pub mod db;

// 进程管理器
// pub mod process;
// 日志服务
pub mod logging;
// 脚本管理
pub mod scripts;
// 共享内存模型管理器
//pub mod shared;
// 视觉服务
pub mod adb_cli_local;
pub mod app_handle;
pub mod capture;
pub mod context;
pub mod devices;
mod hash_calculated;
pub mod image;
pub mod ipc;
pub mod ort;
pub mod path_resolve;
pub mod vision;
pub mod store_local;
