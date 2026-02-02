use crate::infrastructure::core::UuidV7;
use tauri::command;

pub mod config;
pub mod img;

/// 设备配置表名
#[command]
pub async fn get_uuid_v7() -> UuidV7 {
    UuidV7::new_v7()
}