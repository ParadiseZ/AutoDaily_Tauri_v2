use ad_kernel::ids::UuidV7;
use tauri::command;

pub(crate) mod debug;
pub(crate) mod device;
pub(crate) mod device_dto;
pub(crate) mod execution;
pub(crate) mod schedule;
pub(crate) mod script;
pub(crate) mod settings;
pub(crate) mod vision;

/// 设备配置表名
#[command]
pub async fn get_uuid_v7() -> UuidV7 {
    UuidV7::new_v7()
}
