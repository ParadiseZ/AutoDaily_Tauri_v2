use crate::constant::sys_conf_path::{APP_STORE, DEVICES_CONFIG_KEY};
use crate::infrastructure::core::{DeviceId, HashMap};
use crate::infrastructure::devices::device_conf::DeviceConfig;
use tauri::{command, AppHandle};
use tauri_plugin_store::StoreExt;

#[command]
pub async fn get_all_devices_cmd(app_handle: AppHandle) -> Result<HashMap<DeviceId, DeviceConfig>, String> {
    let store = app_handle.store(APP_STORE).map_err(|e| e.to_string())?;
    
    if let Some(value) = store.get(DEVICES_CONFIG_KEY) {
        let devices: HashMap<DeviceId, DeviceConfig> = serde_json::from_value(value.clone())
            .map_err(|e| format!("Failed to deserialize devices: {}", e))?;
        Ok(devices)
    } else {
        Ok(HashMap::default())
    }
}

#[command]
pub async fn save_device_cmd(app_handle: AppHandle, device: DeviceConfig) -> Result<(), String> {
    let store = app_handle.store(APP_STORE).map_err(|e| e.to_string())?;
    
    let mut devices: HashMap<DeviceId, DeviceConfig> = if let Some(value) = store.get(DEVICES_CONFIG_KEY) {
        serde_json::from_value(value.clone()).unwrap_or_default()
    } else {
        HashMap::default()
    };

    devices.insert(device.device_id.clone(), device);
    
    store.set(DEVICES_CONFIG_KEY, serde_json::to_value(devices).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[command]
pub async fn delete_device_cmd(app_handle: AppHandle, device_id: DeviceId) -> Result<(), String> {
    let store = app_handle.store(APP_STORE).map_err(|e| e.to_string())?;
    
    let mut devices: HashMap<DeviceId, DeviceConfig> = if let Some(value) = store.get(DEVICES_CONFIG_KEY) {
        serde_json::from_value(value.clone()).unwrap_or_default()
    } else {
        HashMap::default()
    };

    devices.remove(&device_id);
    
    store.set(DEVICES_CONFIG_KEY, serde_json::to_value(devices).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}
