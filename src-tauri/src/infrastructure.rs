pub mod core;
// 进程管理器 
pub mod process;
// IPC通信框架
pub mod ipc;
// 程序配置
pub mod config;
// 日志服务
pub mod logging;
// 脚本管理
pub mod scripts;
// 共享内存模型管理器
//pub mod shared;
// 视觉服务
pub mod vision;
pub mod devices;
pub mod capture;
pub(crate) mod context;
pub mod image;
mod ort;
mod hash_calculated;
pub mod app_handle;
pub mod path_resolve;

pub mod adb_cli_local;