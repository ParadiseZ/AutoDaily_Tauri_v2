use crate::infrastructure::core::{DeviceId, HashMap};
use crate::infrastructure::devices::device_conf::DeviceConfig;
use crate::infrastructure::db::DbRepo;
use tauri::command;

/// 设备配置表名
const DEVICE_TABLE: &str = "device_configs";

/// 获取所有设备配置
#[command]
pub async fn get_all_devices_cmd() -> Result<Vec<DeviceConfig>, String> {
    DbRepo::get_content::<DeviceConfig>(DEVICE_TABLE).await?
}

/// 根据 ID 获取设备配置
#[command]
pub async fn get_device_by_id_cmd(device_id: DeviceId) -> Result<Option<DeviceConfig>, String> {
    match DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string()).await{
        Ok(dev) => Ok(dev),
        Err(err) => Err(err),
    }
}

/// 保存（新增或更新）设备配置
#[command]
pub async fn save_device_cmd(device: DeviceConfig) -> Result<(), String> {
    match DbRepo::upsert(DEVICE_TABLE, &device.device_id.to_string(), &device).await{
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

/// 删除设备配置
#[command]
pub async fn delete_device_cmd(device_id: DeviceId) -> Result<(), String> {
    match DbRepo::delete(DEVICE_TABLE, &device_id.to_string()).await{
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}
