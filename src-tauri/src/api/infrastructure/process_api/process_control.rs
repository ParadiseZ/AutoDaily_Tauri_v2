use super::runtime_session::{
    build_child_init_data, load_device_table, load_runtime_session_for_target,
    validate_runtime_platform_supported,
};
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage, RunTarget,
    SessionControlMessage,
};
use tauri::{command, Manager};

async fn send_session_control(device_id: DeviceId, control: SessionControlMessage) {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::SessionControl(control),
    );
    IpcServer::send_to_client(&device_id, msg).await;
}

fn send_process_control(device_id: DeviceId, action: ProcessAction) {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ProcessControl(ProcessControlMessage { action }),
    );
    tauri::async_runtime::spawn(async move {
        IpcServer::send_to_client(&device_id, msg).await;
    });
}

async fn wait_for_ipc_client(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    timeout: std::time::Duration,
) -> Result<(), String> {
    let started_at = tokio::time::Instant::now();
    loop {
        {
            let ipc_servers = app_handle.state::<MainProcessCtx>().ipc_servers.clone();
            let guard = ipc_servers
                .read()
                .map_err(|_| "读取 IPC 状态失败".to_string())?;
            if guard
                .iter()
                .any(|(registered_device_id, _)| **registered_device_id == device_id)
            {
                return Ok(());
            }
        }

        if started_at.elapsed() >= timeout {
            return Err(format!("设备[{}]子进程启动后未及时连上 IPC", device_id));
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn ensure_device_online(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;

    if !manager.is_running(&device_id).await {
        let init_data = build_child_init_data(app_handle, device_id, false).await?;
        manager.spawn_child(init_data).await?;
    }

    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await
}

async fn restart_device_runtime_internal(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let was_running = manager.is_running(&device_id).await;

    if was_running {
        manager.stop_child(&device_id).await?;
    }

    let init_data = build_child_init_data(app_handle, device_id, false).await?;
    manager.spawn_child(init_data).await?;
    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;
    let session = load_runtime_session_for_target(app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;

    Ok(format!("设备[{}]子进程已重启并重新装填 session", device_id))
}

#[command]
pub async fn cmd_device_start(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    ensure_device_online(&app_handle, device_id).await?;
    let session = load_runtime_session_for_target(&app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;
    send_process_control(device_id, ProcessAction::Start);
    Ok(format!("已向设备[{}]发送启动命令", device_id))
}

#[command]
pub async fn cmd_device_stop(device_id: DeviceId) -> Result<String, String> {
    send_process_control(device_id, ProcessAction::Stop);
    Ok(format!("已向设备[{}]发送停止命令", device_id))
}

#[command]
pub async fn cmd_device_pause(device_id: DeviceId) -> Result<String, String> {
    send_process_control(device_id, ProcessAction::Pause);
    Ok(format!("已向设备[{}]发送暂停命令", device_id))
}

#[command]
pub async fn cmd_sync_device_runtime_session(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    ensure_device_online(&app_handle, device_id).await?;
    let session =
        load_runtime_session_for_target(&app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_session_control(device_id, SessionControlMessage::ReloadSession { session }).await;
    Ok(format!("已同步设备[{}]运行会话", device_id))
}

#[command]
pub async fn cmd_run_script_target(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    target: RunTarget,
) -> Result<String, String> {
    ensure_device_online(&app_handle, device_id).await?;
    let session = load_runtime_session_for_target(&app_handle, device_id, target.clone()).await?;
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;
    send_process_control(device_id, ProcessAction::Start);
    Ok(format!("已向设备[{}]发送运行目标: {:?}", device_id, target))
}

#[command]
pub async fn cmd_restart_device_runtime(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    restart_device_runtime_internal(&app_handle, device_id).await
}

#[command]
pub async fn cmd_device_shutdown(device_id: DeviceId) -> Result<String, String> {
    if let Some(manager) = get_process_manager() {
        manager.stop_child(&device_id).await?;
        Ok(format!("设备[{}]子进程已关闭", device_id))
    } else {
        Err("进程管理器未初始化".to_string())
    }
}

#[command]
pub async fn cmd_get_running_devices() -> Result<Vec<String>, String> {
    if let Some(manager) = get_process_manager() {
        let ids = manager.get_running_device_ids().await;
        Ok(ids.iter().map(|id| id.to_string()).collect())
    } else {
        Err("进程管理器未初始化".to_string())
    }
}

#[command]
pub async fn cmd_spawn_device(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let init_data = build_child_init_data(&app_handle, device_id, false).await?;
    let device_name = init_data.device_config.device_name.clone();
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    manager.spawn_child(init_data).await?;
    wait_for_ipc_client(&app_handle, device_id, std::time::Duration::from_secs(5)).await?;

    Ok(format!("设备[{}]({})子进程已启动", device_name, device_id))
}

#[command]
pub async fn cmd_prepare_device_capture(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    if manager.is_running(&device_id).await {
        return Ok(format!("设备[{}]子进程已在运行", device_id));
    }

    let init_data = build_child_init_data(&app_handle, device_id, true).await?;
    let device_name = init_data.device_config.device_name.clone();
    manager.spawn_child(init_data).await?;
    wait_for_ipc_client(&app_handle, device_id, std::time::Duration::from_secs(5)).await?;

    Ok(format!(
        "设备[{}]({})已启动并完成连接准备",
        device_name, device_id
    ))
}

#[command]
pub async fn cmd_is_device_running(device_id: DeviceId) -> Result<bool, String> {
    if let Some(manager) = get_process_manager() {
        Ok(manager.is_running(&device_id).await)
    } else {
        Err("进程管理器未初始化".to_string())
    }
}
