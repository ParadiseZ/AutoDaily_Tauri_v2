use crate::infrastructure::core::UuidV7;
use tauri::command;

pub mod config;
pub mod frontend_debug;
pub mod img;
pub mod process_api;
pub mod runtime_sync;

/// 设备配置表名
#[command]
pub async fn get_uuid_v7() -> UuidV7 {
    UuidV7::new_v7()
}
