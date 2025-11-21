use crate::app::app_error::AppResult;
use crate::constant::sys_conf_path::DEVICES_CONFIG_PATH;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::core::{DeviceId, ScriptId};
use crate::infrastructure::devices::device_conf::{DeviceConfMap, DeviceConfig};
use crate::infrastructure::scripts::script_info_model::ScriptManager;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

pub async fn refresh_devices(device_id: DeviceId, device_conf: DeviceConfig) -> AppResult<()> {
    let handle = get_app_handle();
    // 更新设备信息
    handle
        .state::<Arc<Mutex<DeviceConfMap>>>()
        .lock()
        .await
        .insert(device_id, device_conf.clone());

    // 写入文件
    let mut devices = handle
        .state::<ConfigManager>()
        .get_conf_mut::<DeviceConfMap>(DEVICES_CONFIG_PATH)
        .await?;
    devices.config.insert(device_id, device_conf);
    Ok(())
}

pub async fn refresh_device_by_script_id(
    script_id: ScriptId,
    device_id: DeviceId,
) -> AppResult<()> {
    let handle = get_app_handle();
    let mut script_info = handle
        .state::<Arc<Mutex<ScriptManager>>>()
        .lock()
        .await
        .load_script_info(script_id)?;
    script_info.device_account.insert(device_id, None);

    //TODO 写入文件
}

pub async fn refresh_all(handle: &AppHandle) {
    let main_ctx = handle.state::<Arc<Mutex<MainProcessCtx>>>();
    let mut ctx = main_ctx.lock().await;
    // 以主上下文内的设备配置为准
    let _ = ctx
        .log_receiver
        .lock()
        .await
        .refresh_from_config(&ctx.device_config);
}

pub async fn refresh_log_receivers(handle: &AppHandle) {
    let main_ctx = handle.state::<Arc<Mutex<MainProcessCtx>>>();
    let mut ctx = main_ctx.lock().await;
    // 以主上下文内的设备配置为准
    let _ = ctx
        .log_receiver
        .lock()
        .await
        .refresh_from_config(&ctx.device_config);
}
