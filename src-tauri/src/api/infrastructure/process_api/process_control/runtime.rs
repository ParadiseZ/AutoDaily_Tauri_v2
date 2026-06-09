use super::events::{emit_assignment_schedule_changed, emit_device_connection_status, emit_device_progress_status};
use super::scheduler::{block_device_auto_dispatch, dispatch_next_scheduled_queue_item};
use super::state::{
    mark_active_dispatch, reset_device_dispatch_state, set_auto_dispatch_blocked,
    snapshot_device_dispatch_state,
};
use super::super::runtime_session::{
    build_child_init_data, load_device_table, load_runtime_session_for_queue_item,
    validate_runtime_platform_supported,
};
use crate::domain::devices::device_runtime_event::DeviceRuntimeProgressPhase;
use crate::domain::devices::device_conf::DeviceTable;
use crate::domain::devices::device_schedule::AssignmentScheduleStatus;
use crate::infrastructure::context::child_process_manager::{
    get_process_manager, set_child_process_exit_handler,
};
use crate::infrastructure::context::main_process::{
    ChildRuntimeStatus, DeviceCaptureResult, DeviceDispatchSignal, MainProcessCtx,
};
use crate::infrastructure::core::{DeviceId, DispatchId, MessageId};
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    CaptureControlMessage, ConnectionAction, ConnectionControlMessage, ConnectionStatusKind,
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    RuntimeDispatchPhase, RuntimeQueueItem, RuntimeSessionSnapshot, SessionControlMessage,
};
use crate::infrastructure::logging::log_trait::Log;
use chrono::Local;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

const EMULATOR_CONNECTION_READY_GRACE_SECS: u64 = 65;
const DEVICE_CONNECTION_READY_TIMEOUT_SECS: u64 = 25;

fn device_connection_timeout(device_table: &DeviceTable) -> std::time::Duration {
    if device_table.data.0.uses_emulator_transport() {
        std::time::Duration::from_secs(
            u64::from(device_table.data.0.startup_delay_secs)
                + EMULATOR_CONNECTION_READY_GRACE_SECS,
        )
    } else {
        std::time::Duration::from_secs(DEVICE_CONNECTION_READY_TIMEOUT_SECS)
    }
}

fn build_command_message(device_id: DeviceId, payload: MessagePayload) -> IpcMessage {
    IpcMessage::new(device_id, MessageType::Command, payload)
}

async fn send_command_payload(device_id: DeviceId, payload: MessagePayload) -> MessageId {
    let msg = build_command_message(device_id, payload);
    let request_id = msg.id;
    IpcServer::send_to_client(&device_id, msg).await;
    request_id
}

fn spawn_command_payload(device_id: DeviceId, payload: MessagePayload) {
    let msg = build_command_message(device_id, payload);
    tauri::async_runtime::spawn(async move {
        IpcServer::send_to_client(&device_id, msg).await;
    });
}

pub(super) async fn send_session_control(device_id: DeviceId, control: SessionControlMessage) {
    let _ = send_command_payload(
        device_id,
        MessagePayload::SessionControl(control),
    )
    .await;
}

pub(super) fn send_process_control(device_id: DeviceId, action: ProcessAction) {
    spawn_command_payload(
        device_id,
        MessagePayload::ProcessControl(ProcessControlMessage { action }),
    );
}

pub(super) async fn send_connection_control(device_id: DeviceId, action: ConnectionAction) {
    let _ = send_command_payload(
        device_id,
        MessagePayload::ConnectionControl(ConnectionControlMessage { action }),
    )
    .await;
}

pub(super) async fn probe_device_connection(
    app_handle: &AppHandle,
    device_id: DeviceId,
    message: &str,
) -> Result<(), String> {
    set_connection_status(
        app_handle,
        device_id,
        ConnectionStatusKind::DeviceChecking,
        Some(message.to_string()),
    )
    .await?;
    send_connection_control(device_id, ConnectionAction::Probe).await;
    Ok(())
}

pub(super) async fn send_capture_control(device_id: DeviceId) -> MessageId {
    send_command_payload(
        device_id,
        MessagePayload::CaptureControl(CaptureControlMessage),
    )
    .await
}

pub(super) async fn dispatch_queue_item_to_child(
    app_handle: &AppHandle,
    device_id: DeviceId,
    queue_item: RuntimeQueueItem,
) -> Result<(), String> {
    let session =
        load_runtime_session_for_queue_item(app_handle, device_id, queue_item.clone()).await?;
    dispatch_session_to_child(app_handle, device_id, session, queue_item.dispatch_id).await
}

pub(super) async fn dispatch_session_to_child(
    app_handle: &AppHandle,
    device_id: DeviceId,
    session: RuntimeSessionSnapshot,
    dispatch_id: DispatchId,
) -> Result<(), String> {
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;
    send_process_control(device_id, ProcessAction::Start);
    mark_active_dispatch(app_handle, device_id, Some(dispatch_id))?;
    Ok(())
}

pub(super) async fn wait_for_ipc_client(
    app_handle: &AppHandle,
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

async fn ensure_child_runtime_ipc_ready(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;

    if !manager.is_running(&device_id).await {
        let _ = app_handle
            .state::<MainProcessCtx>()
            .set_child_runtime_status(device_id, ChildRuntimeStatus::Starting);
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::ChildRuntimeStarting,
            "正在启动设备运行时",
        );
        let init_data = build_child_init_data(app_handle, device_id).await?;
        manager.spawn_child(init_data).await?;
        let _ = app_handle
            .state::<MainProcessCtx>()
            .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcWaiting);
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::ChildIpcWaiting,
            "正在等待设备运行时 IPC 连接",
        );
        wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;
        let _ = app_handle
            .state::<MainProcessCtx>()
            .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcReady);
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::ChildIpcReady,
            "设备运行时 IPC 已连接，准备设备连接",
        );
        return Ok(());
    }

    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcWaiting);
    emit_device_progress_status(
        app_handle,
        device_id,
        DeviceRuntimeProgressPhase::ChildIpcWaiting,
        "正在确认设备运行时 IPC 连接",
    );
    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcReady);
    emit_device_progress_status(
        app_handle,
        device_id,
        DeviceRuntimeProgressPhase::ChildIpcReady,
        "设备运行时 IPC 已连接，准备设备连接",
    );
    Ok(())
}

pub(super) async fn set_connection_status(
    app_handle: &AppHandle,
    device_id: DeviceId,
    status: ConnectionStatusKind,
    message: Option<String>,
) -> Result<(), String> {
    let state = app_handle.state::<MainProcessCtx>();
    state.set_device_connection_state(device_id, status.clone(), message.clone())?;
    emit_device_connection_status(app_handle, device_id, &status, message.as_deref());
    if let Some(message) = message {
        emit_device_progress_status(app_handle, device_id, status.into(), message);
    }
    Ok(())
}

fn subscribe_device_connection_status(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<
    tokio::sync::watch::Receiver<crate::infrastructure::context::main_process::DeviceRuntimeState>,
    String,
> {
    app_handle
        .state::<MainProcessCtx>()
        .subscribe_device_runtime_state(device_id)
}

async fn request_child_device_connection(
    app_handle: &AppHandle,
    device_id: DeviceId,
    timeout: std::time::Duration,
) -> Result<(), String> {
    let mut connection_rx = subscribe_device_connection_status(app_handle, device_id)?;
    set_connection_status(
        app_handle,
        device_id,
        ConnectionStatusKind::DeviceChecking,
        Some("正在准备设备连接".to_string()),
    )
    .await?;
    send_connection_control(device_id, ConnectionAction::EnsureReady).await;

    let wait_result = tokio::time::timeout(timeout, async {
        loop {
            connection_rx
                .changed()
                .await
                .map_err(|_| format!("设备[{}]连接状态通知已关闭", device_id))?;
            let state = connection_rx.borrow().clone();
            match state.connection.status {
                ConnectionStatusKind::DeviceConnected
                | ConnectionStatusKind::DeviceDisconnected => {
                    return Ok::<(ConnectionStatusKind, Option<String>), String>((
                        state.connection.status,
                        state.connection.message,
                    ));
                }
                ConnectionStatusKind::DeviceUnknown
                | ConnectionStatusKind::DeviceChecking
                | ConnectionStatusKind::ShellProbeChecking
                | ConnectionStatusKind::EmulatorStarting
                | ConnectionStatusKind::EmulatorWaiting => {}
            }
        }
    })
    .await;
    let (status, message) = match wait_result {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => {
            let _ = set_connection_status(
                app_handle,
                device_id,
                ConnectionStatusKind::DeviceDisconnected,
                Some(error.clone()),
            )
            .await;
            return Err(error);
        }
        Err(_) => {
            let error = format!("设备[{}]连接准备超时", device_id);
            let _ = set_connection_status(
                app_handle,
                device_id,
                ConnectionStatusKind::DeviceDisconnected,
                Some(error.clone()),
            )
            .await;
            return Err(error);
        }
    };

    match status {
        ConnectionStatusKind::DeviceConnected => Ok(()),
        ConnectionStatusKind::DeviceDisconnected => {
            let message = message.unwrap_or_else(|| format!("设备[{}]连接准备失败", device_id));
            let _ = set_connection_status(
                app_handle,
                device_id,
                ConnectionStatusKind::DeviceDisconnected,
                Some(message.clone()),
            )
            .await;
            Err(message)
        }
        ConnectionStatusKind::DeviceUnknown
        | ConnectionStatusKind::DeviceChecking
        | ConnectionStatusKind::ShellProbeChecking
        | ConnectionStatusKind::EmulatorStarting
        | ConnectionStatusKind::EmulatorWaiting => {
            let message = format!("设备[{}]连接状态未知", device_id);
            let _ = set_connection_status(
                app_handle,
                device_id,
                ConnectionStatusKind::DeviceDisconnected,
                Some(message.clone()),
            )
            .await;
            Err(message)
        }
    }
}

pub(super) async fn ensure_device_ready(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    prepare_device_connection(app_handle, device_id, true)
        .await
        .map(|_| ())
}

pub(super) async fn ensure_device_ready_for_manual(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    set_auto_dispatch_blocked(app_handle, device_id, false)?;
    if let Err(error) = ensure_device_ready(app_handle, device_id).await {
        if let Err(block_error) = block_device_auto_dispatch(
            app_handle,
            device_id,
            "手动派发连接失败，停止该设备后续自动派发".to_string(),
        )
        .await
        {
            Log::error(&format!(
                "[ process ] 设备[{}]持久化自动派发停止状态失败: {}",
                device_id, block_error
            ));
        }
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::Failed,
            "手动派发中该设备连接失败，该设备自动派发保持暂停，可再次手动重试",
        );
        return Err(error);
    }
    Ok(())
}

pub(super) async fn ensure_device_capture_ready(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    prepare_device_connection(app_handle, device_id, false)
        .await
        .map(|device_table| device_table.data.0.device_name.clone())
}

async fn prepare_device_connection(
    app_handle: &AppHandle,
    device_id: DeviceId,
    require_enabled: bool,
) -> Result<DeviceTable, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    if require_enabled && !device_table.data.0.enable {
        return Err(format!("设备[{}]未启用", device_table.data.0.device_name));
    }
    if let Err(error) = ensure_child_runtime_ipc_ready(app_handle, device_id).await {
        let _ = set_connection_status(
            app_handle,
            device_id,
            ConnectionStatusKind::DeviceDisconnected,
            Some(error.clone()),
        )
        .await;
        return Err(error);
    }
    request_child_device_connection(app_handle, device_id, device_connection_timeout(&device_table))
        .await?;
    Ok(device_table)
}

pub(super) async fn wait_for_capture_result(
    app_handle: &AppHandle,
    request_id: MessageId,
    timeout: std::time::Duration,
) -> Result<String, String> {
    let started_at = tokio::time::Instant::now();
    loop {
        {
            let state = app_handle.state::<MainProcessCtx>();
            let mut guard = state
                .device_capture_results
                .write()
                .map_err(|_| "读取设备截图结果失败".to_string())?;
            if let Some(result) = guard.remove(&request_id) {
                let DeviceCaptureResult {
                    device_id,
                    image_data,
                    message,
                } = result;
                return image_data.ok_or_else(|| {
                    message.unwrap_or_else(|| format!("设备[{}]截图失败", device_id))
                });
            }
        }

        if started_at.elapsed() >= timeout {
            return Err("等待设备截图结果超时".to_string());
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub(crate) fn spawn_dispatch_signal_loop(
    app_handle: tauri::AppHandle,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<DeviceDispatchSignal>,
) {
    tauri::async_runtime::spawn(async move {
        while let Some(signal) = rx.recv().await {
            let assignment_schedule_changed = signal.dispatch_id.is_some()
                && matches!(
                    &signal.phase,
                    RuntimeDispatchPhase::Started
                        | RuntimeDispatchPhase::Finished
                        | RuntimeDispatchPhase::Failed
                );
            let result = match signal.phase {
                RuntimeDispatchPhase::Started => {
                    if let Some(dispatch_id) = signal.dispatch_id {
                        super::super::dispatch_planner::update_assignment_schedule_status_by_dispatch_id(
                            dispatch_id,
                            AssignmentScheduleStatus::Running,
                            Some(signal.at.clone()),
                            None,
                            signal.message.clone(),
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                RuntimeDispatchPhase::Finished => {
                    if let Some(dispatch_id) = signal.dispatch_id {
                        super::super::dispatch_planner::update_assignment_schedule_status_by_dispatch_id(
                            dispatch_id,
                            AssignmentScheduleStatus::Success,
                            None,
                            Some(signal.at.clone()),
                            signal.message.clone(),
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                RuntimeDispatchPhase::Failed => {
                    if let Some(dispatch_id) = signal.dispatch_id {
                        super::super::dispatch_planner::update_assignment_schedule_status_by_dispatch_id(
                            dispatch_id,
                            AssignmentScheduleStatus::Failed,
                            None,
                            Some(signal.at.clone()),
                            signal.message.clone(),
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                RuntimeDispatchPhase::RequestNext => {
                    match mark_active_dispatch(&app_handle, signal.device_id, None) {
                        Ok(()) => dispatch_next_scheduled_queue_item(&app_handle, signal.device_id)
                            .await
                            .map(|_| ()),
                        Err(error) => Err(error),
                    }
                }
            };

            if let Err(error) = result {
                Log::error(&format!(
                    "[ process ] 处理设备[{}] dispatch 信号失败: {}",
                    signal.device_id, error
                ));
            } else if assignment_schedule_changed {
                emit_assignment_schedule_changed(&app_handle, signal.device_id);
            }
        }
    });
}

pub(crate) fn register_child_process_exit_handler(app_handle: tauri::AppHandle) {
    let result = set_child_process_exit_handler(Arc::new(move |device_id, success, message| {
        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let had_active = snapshot_device_dispatch_state(&app_handle, device_id)
                .map(|state| state.active_dispatch.is_some())
                .unwrap_or(false);
            let _ = mark_active_dispatch(&app_handle, device_id, None);
            if let Err(error) = block_device_auto_dispatch(
                &app_handle,
                device_id,
                "设备运行时已退出，停止该设备后续自动派发".to_string(),
            )
            .await
            {
                Log::error(&format!(
                    "[ process ] 设备[{}]子进程退出后持久化自动派发停止状态失败: {}",
                    device_id, error
                ));
            }

            let completed_at = Local::now().to_rfc3339();
            let failed = super::super::dispatch_planner::fail_active_assignment_schedules_by_device(
                device_id,
                completed_at,
                message.clone(),
            )
            .await;
            match failed {
                Ok(count) => {
                    if count > 0 || had_active {
                        emit_assignment_schedule_changed(&app_handle, device_id);
                    }
                    let phase = if success {
                        DeviceRuntimeProgressPhase::ChildProcessExited
                    } else {
                        DeviceRuntimeProgressPhase::ChildProcessCrashed
                    };
                    emit_device_progress_status(&app_handle, device_id, phase, message);
                }
                Err(error) => {
                    Log::error(&format!(
                        "[ process ] 设备[{}]子进程退出后更新 assignment_schedules 失败: {}",
                        device_id, error
                    ));
                }
            }
        });
    }));
    if let Err(error) = result {
        Log::warn(&format!("[ process ] 注册子进程退出处理器失败: {}", error));
    }
}

pub(super) async fn spawn_device_runtime_internal(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let init_data = build_child_init_data(app_handle, device_id).await?;
    let device_name = init_data.device_config.device_name.clone();
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::Starting);
    manager.spawn_child(init_data).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcWaiting);
    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcReady);
    Ok(device_name)
}

pub(super) async fn restart_device_runtime_internal(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let was_running = manager.is_running(&device_id).await;

    if was_running {
        manager.stop_child(&device_id).await?;
    }

    spawn_device_runtime_internal(app_handle, device_id).await?;
    let _ = probe_device_connection(app_handle, device_id, "正在检查设备连接").await;

    Ok(format!("设备[{}]子进程已重启", device_id))
}

pub(super) async fn shutdown_device_runtime_internal(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    if let Some(manager) = get_process_manager() {
        manager.stop_child(&device_id).await?;
        let _ = reset_device_dispatch_state(app_handle, device_id);
        Ok(format!("设备[{}]子进程已关闭", device_id))
    } else {
        Err("进程管理器未初始化".to_string())
    }
}
