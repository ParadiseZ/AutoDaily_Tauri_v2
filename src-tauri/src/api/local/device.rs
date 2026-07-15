use crate::api::local::device_dto::DeviceTable;
use crate::api::local::execution::{
    enqueue_device_config_reconcile_job, notify_auto_dispatch_planner,
};
use crate::infra::context::child_process_manager::get_process_manager;
use crate::infra::context::main_process::MainProcessCtx;
use crate::infra::logging::log_trait::Log;
use ad_kernel::ids::DeviceId;
use domain_device::DeviceProfile;
use infra_sqlite::{
    delete_device_with_assignments, get_all_devices, get_device, has_active_assignment_schedules,
    save_device,
};
use tauri::{Manager, command};

async fn resolve_device_log_label(app_handle: &tauri::AppHandle, device_id: DeviceId) -> String {
    if let Ok(Some(profile)) = get_device(device_id).await {
        let name = profile.config.device_name.trim().to_string();
        if !name.is_empty() {
            return name;
        }
    }

    app_handle
        .state::<MainProcessCtx>()
        .snapshot_device_runtime_state(device_id)
        .ok()
        .and_then(|state| state.device_name)
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "未知设备".to_string())
}

/// 获取所有设备配置
#[command]
pub async fn get_all_devices_cmd() -> Result<Vec<DeviceTable>, String> {
    get_all_devices()
        .await
        .map(|devices| devices.into_iter().map(Into::into).collect())
}

/// 根据 ID 获取设备配置
#[command]
pub async fn get_device_by_id_cmd(device_id: DeviceId) -> Result<Option<DeviceTable>, String> {
    get_device(device_id)
        .await
        .map(|device| device.map(Into::into))
}

/// 保存（新增或更新）设备配置
#[command]
pub async fn save_device_cmd(
    app_handle: tauri::AppHandle,
    device: DeviceTable,
) -> Result<(), String> {
    let device: DeviceProfile = device.into();
    let device_name = device.config.device_name.clone();
    let result = async {
        let previous = get_device(device.id).await?;
        save_device(&device).await?;
        notify_auto_dispatch_planner();
        enqueue_device_config_reconcile_job(&app_handle, previous, device)?;
        Ok(())
    }
    .await;

    if let Err(error) = &result {
        Log::error(&format!(
            "[ device ] 保存设备失败 device_name={} error={}",
            device_name, error
        ));
    }

    result
}

fn ensure_device_deletable(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let runtime_state = app_handle
        .state::<MainProcessCtx>()
        .snapshot_device_dispatch_state(device_id)?;
    if runtime_state.active_dispatch.is_some()
        || !runtime_state.pending_dispatches.is_empty()
        || !runtime_state.pending_debug_sessions.is_empty()
    {
        return Err("设备仍有活动或待派发的运行任务，请停止并清空运行状态后再删除。".to_string());
    }
    Ok(())
}

async fn ensure_device_deletable_async(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    ensure_device_deletable(app_handle, device_id)?;
    if let Some(manager) = get_process_manager() {
        if manager.is_running(&device_id).await {
            return Err("设备子进程仍在运行，请先关闭设备后再删除。".to_string());
        }
    }

    if has_active_assignment_schedules(device_id).await? {
        return Err("设备仍有未完成的调度记录，请先停止或清理调度后再删除。".to_string());
    }

    Ok(())
}

/// 删除设备配置
#[command]
pub async fn delete_device_cmd(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let device_label = resolve_device_log_label(&app_handle, device_id).await;
    let result = async {
        ensure_device_deletable_async(&app_handle, device_id).await?;

        delete_device_with_assignments(device_id).await?;

        let _ = app_handle
            .state::<crate::infra::context::main_process::MainProcessCtx>()
            .clear_device_runtime_state(device_id);
        notify_auto_dispatch_planner();
        Ok(())
    }
    .await;

    if let Err(error) = &result {
        Log::error(&format!(
            "[ device ] 删除设备失败 device_name={} error={}",
            device_label, error
        ));
    }

    result
}

/// 获取当前 CPU 核心数
#[command]
pub fn get_cpu_count_cmd() -> usize {
    let physical = num_cpus::get_physical();
    if physical > 0 {
        physical
    } else {
        num_cpus::get().max(1)
    }
}
