use super::bundle_loader::load_runtime_queue_for_current_window;
use super::dispatch_planner::{
    insert_assignment_schedule, update_assignment_schedule_status_by_dispatch_id, DispatchPlanner,
};
use super::runtime_session::{
    build_child_init_data, load_device_table, load_runtime_session_for_queue_item,
    load_runtime_session_for_target, validate_runtime_platform_supported,
};
use crate::constant::table_name::DEVICE_TABLE;
use crate::domain::devices::device_schedule::{AssignmentScheduleStatus, AssignmentTriggerSource};
use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::context::main_process::{DeviceDispatchSignal, MainProcessCtx};
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    CaptureControlMessage, ConnectionAction, ConnectionControlMessage, ConnectionStatusKind,
    DispatchSource, IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    RunTarget, RuntimeDispatchPhase, RuntimeQueueItem, SessionControlMessage,
};
use crate::infrastructure::logging::log_trait::Log;
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

async fn send_connection_control(device_id: DeviceId, action: ConnectionAction) {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ConnectionControl(ConnectionControlMessage { action }),
    );
    IpcServer::send_to_client(&device_id, msg).await;
}

async fn send_capture_control(device_id: DeviceId) -> crate::infrastructure::core::MessageId {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::CaptureControl(CaptureControlMessage),
    );
    let request_id = msg.id;
    IpcServer::send_to_client(&device_id, msg).await;
    request_id
}

async fn dispatch_queue_item_to_child(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    queue_item: RuntimeQueueItem,
) -> Result<(), String> {
    let session = load_runtime_session_for_queue_item(app_handle, device_id, queue_item.clone()).await?;
    if matches!(queue_item.dispatch_source, DispatchSource::Planner) {
        let _ = insert_assignment_schedule(
            device_id,
            queue_item.assignment_id,
            queue_item.time_template_id,
            queue_item.window_start_at.clone(),
            queue_item.dispatch_id,
            AssignmentScheduleStatus::Dispatched,
            AssignmentTriggerSource::Planner,
            Some("dispatch 已派发到子进程".to_string()),
        )
        .await;
    }
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;
    send_process_control(device_id, ProcessAction::Start);
    DispatchPlanner::init().mark_active_dispatch(device_id, Some(queue_item.dispatch_id))?;
    Ok(())
}

async fn dispatch_next_pending_queue_item(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<bool, String> {
    let planner = DispatchPlanner::init();
    let Some(queue_item) = planner.pop_next_dispatch(device_id)? else {
        planner.mark_active_dispatch(device_id, None)?;
        return Ok(false);
    };
    dispatch_queue_item_to_child(app_handle, device_id, queue_item).await?;
    Ok(true)
}

async fn rebuild_device_queue_pending_dispatches(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    ensure_device_ready(app_handle, device_id).await?;
    let queue = load_runtime_queue_for_current_window(device_id).await?;
    let planner = DispatchPlanner::init();
    planner.ensure_device_state(device_id)?;
    planner.replace_pending_dispatches(device_id, queue.clone())?;
    planner.mark_active_dispatch(device_id, None)?;
    Ok(queue)
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

async fn set_connection_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    status: ConnectionStatusKind,
    message: Option<String>,
) -> Result<(), String> {
    let state = app_handle.state::<MainProcessCtx>();
    let mut guard = state
        .device_connections
        .write()
        .map_err(|_| "写入连接状态失败".to_string())?;
    guard.insert(
        device_id,
        crate::infrastructure::context::main_process::DeviceConnectionState { status, message },
    );
    Ok(())
}

async fn wait_for_connection_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    timeout: std::time::Duration,
) -> Result<(ConnectionStatusKind, Option<String>), String> {
    let started_at = tokio::time::Instant::now();
    loop {
        {
            let state = app_handle.state::<MainProcessCtx>();
            let guard = state
                .device_connections
                .read()
                .map_err(|_| "读取连接状态失败".to_string())?;
            if let Some(status) = guard.get(&device_id) {
                match status.status {
                    ConnectionStatusKind::Connected | ConnectionStatusKind::Disconnected => {
                        return Ok((status.status.clone(), status.message.clone()));
                    }
                    ConnectionStatusKind::Unknown | ConnectionStatusKind::Checking => {}
                }
            }
        }

        if started_at.elapsed() >= timeout {
            return Err(format!("设备[{}]连接准备超时", device_id));
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn ensure_device_connection_ready(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    set_connection_status(
        app_handle,
        device_id,
        ConnectionStatusKind::Checking,
        Some("正在准备设备连接".to_string()),
    )
    .await?;
    send_connection_control(device_id, ConnectionAction::EnsureReady).await;

    match wait_for_connection_status(app_handle, device_id, std::time::Duration::from_secs(35)).await?
    {
        (ConnectionStatusKind::Connected, _) => Ok(()),
        (ConnectionStatusKind::Disconnected, message) => Err(
            message.unwrap_or_else(|| format!("设备[{}]连接准备失败", device_id)),
        ),
        (ConnectionStatusKind::Unknown, _) | (ConnectionStatusKind::Checking, _) => {
            Err(format!("设备[{}]连接状态未知", device_id))
        }
    }
}

async fn ensure_device_ready(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    if !device_table.data.0.enable {
        return Err(format!("设备[{}]未启用", device_table.data.0.device_name));
    }

    ensure_device_online(app_handle, device_id).await?;
    ensure_device_connection_ready(app_handle, device_id).await
}

async fn ensure_device_capture_ready(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let device_name = device_table.data.0.device_name.clone();
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;

    if !manager.is_running(&device_id).await {
        let init_data = build_child_init_data(app_handle, device_id, true).await?;
        manager.spawn_child(init_data).await?;
        wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;
    }

    ensure_device_connection_ready(app_handle, device_id).await?;
    Ok(device_name)
}

async fn wait_for_capture_result(
    app_handle: &tauri::AppHandle,
    request_id: crate::infrastructure::core::MessageId,
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
                let crate::infrastructure::context::main_process::DeviceCaptureResult {
                    device_id,
                    image_data,
                    message,
                } = result;
                return image_data
                    .ok_or_else(|| message.unwrap_or_else(|| format!("设备[{}]截图失败", device_id)));
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
            let result = match signal.phase {
                RuntimeDispatchPhase::Started => {
                    if let Some(dispatch_id) = signal.dispatch_id {
                        update_assignment_schedule_status_by_dispatch_id(
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
                        update_assignment_schedule_status_by_dispatch_id(
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
                        update_assignment_schedule_status_by_dispatch_id(
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
                    match DispatchPlanner::init().mark_active_dispatch(signal.device_id, None) {
                        Ok(()) => dispatch_next_pending_queue_item(&app_handle, signal.device_id)
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
            }
        }
    });
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

    Ok(format!("设备[{}]子进程已重启", device_id))
}

#[command]
pub async fn cmd_device_start(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let queue = rebuild_device_queue_pending_dispatches(&app_handle, device_id).await?;
    if queue.is_empty() {
        return Ok(format!("设备[{}]当前时间窗口下没有可运行的队列项", device_id));
    }
    dispatch_next_pending_queue_item(&app_handle, device_id).await?;
    Ok(format!("已向设备[{}]派发 {} 个队列项，开始执行第一项", device_id, queue.len()))
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
    let queue = rebuild_device_queue_pending_dispatches(&app_handle, device_id).await?;
    Ok(format!(
        "已同步设备[{}]待执行队列，当前时间窗口下共有 {} 个候选项",
        device_id,
        queue.len()
    ))
}

#[command]
pub async fn cmd_run_script_target(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    target: RunTarget,
) -> Result<String, String> {
    ensure_device_ready(&app_handle, device_id).await?;
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
pub async fn cmd_bootstrap_enabled_devices(
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let devices = DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await?;
    let enabled_devices: Vec<DeviceTable> = devices
        .into_iter()
        .filter(|device| device.data.0.enable)
        .collect();

    let total = enabled_devices.len();
    let mut started = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for device in enabled_devices {
        if manager.is_running(&device.id).await {
            skipped += 1;
            continue;
        }

        match cmd_spawn_device(app_handle.clone(), device.id).await {
            Ok(_) => {
                started += 1;
            }
            Err(error) => {
                failed += 1;
                Log::error(&format!(
                    "[ process ] 启动阶段自动拉起设备[{}]子进程失败: {}",
                    device.data.0.device_name, error
                ));
            }
        }
    }

    let summary = format!(
        "启动阶段已检查 {} 台启用设备，启动 {} 台，跳过 {} 台，失败 {} 台",
        total, started, skipped, failed
    );
    Log::info(&format!("[ process ] {}", summary));
    Ok(summary)
}

#[command]
pub async fn cmd_probe_device_connections(
    app_handle: tauri::AppHandle,
    device_ids: Vec<DeviceId>,
) -> Result<String, String> {
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let mut queued = 0usize;
    let mut skipped = 0usize;

    for device_id in device_ids {
        if !manager.is_running(&device_id).await {
            skipped += 1;
            continue;
        }

        match wait_for_ipc_client(&app_handle, device_id, std::time::Duration::from_secs(2)).await {
            Ok(()) => {
                let _ = set_connection_status(
                    &app_handle,
                    device_id,
                    ConnectionStatusKind::Checking,
                    Some("正在检查设备连接".to_string()),
                )
                .await;
                send_connection_control(device_id, ConnectionAction::Probe).await;
                queued += 1;
            }
            Err(error) => {
                skipped += 1;
                Log::warn(&format!(
                    "[ process ] 跳过设备[{}]连接探测：{}",
                    device_id, error
                ));
            }
        }
    }

    Ok(format!(
        "已发起 {} 台设备连接探测，跳过 {} 台",
        queued, skipped
    ))
}

#[command]
pub async fn cmd_prepare_device_capture(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let device_name = ensure_device_capture_ready(&app_handle, device_id).await?;
    Ok(format!("设备[{}]({})已启动并完成连接准备", device_name, device_id))
}

#[command]
pub async fn cmd_capture_device_image(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    ensure_device_capture_ready(&app_handle, device_id).await?;
    let request_id = send_capture_control(device_id).await;
    wait_for_capture_result(&app_handle, request_id, std::time::Duration::from_secs(20)).await
}

#[command]
pub async fn cmd_is_device_running(device_id: DeviceId) -> Result<bool, String> {
    if let Some(manager) = get_process_manager() {
        Ok(manager.is_running(&device_id).await)
    } else {
        Err("进程管理器未初始化".to_string())
    }
}
