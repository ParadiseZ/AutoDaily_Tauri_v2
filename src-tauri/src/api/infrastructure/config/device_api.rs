use crate::constant::table_name::DEVICE_TABLE;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::devices::device_conf::DeviceTable;
use tauri::command;

/// 获取所有设备配置
#[command]
pub async fn get_all_devices_cmd() -> Result<Vec<DeviceTable>, String> {
    DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await
}

/// 根据 ID 获取设备配置
#[command]
pub async fn get_device_by_id_cmd(device_id: DeviceId) -> Result<Option<DeviceTable>, String> {
    DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string()).await
}

/// 保存（新增或更新）设备配置
#[command]
pub async fn save_device_cmd(device: DeviceTable) -> Result<(), String> {
    DbRepo::upsert_id_data(DEVICE_TABLE, &device.id.to_string(), &device.data).await
}

/// 删除设备配置
#[command]
pub async fn delete_device_cmd(device_id: DeviceId) -> Result<(), String> {
    DbRepo::delete(DEVICE_TABLE, &device_id.to_string()).await
}
