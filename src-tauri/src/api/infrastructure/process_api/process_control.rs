use super::bundle_loader::load_runtime_queue_for_current_window;
use super::dispatch_planner::{
    has_complete_assignment_schedule_batch, insert_assignment_schedule,
    insert_assignment_schedule_batch, load_next_planned_assignment_schedule,
    update_assignment_schedule_status, update_assignment_schedule_status_by_dispatch_id,
    DispatchPlanner,
};
use super::runtime_session::{
    build_child_init_data, load_device_table, load_runtime_session_for_queue_item,
    load_runtime_session_for_target, validate_runtime_platform_supported,
};
use crate::constant::table_name::DEVICE_TABLE;
use crate::domain::devices::device_conf::{DeviceConfig, DeviceTable};
use crate::domain::devices::device_schedule::{
    AssignmentSchedule, AssignmentScheduleStatus, AssignmentTriggerSource,
};
use crate::domain::schedule::time_template::TimeTemplate;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::context::main_process::{DeviceDispatchSignal, MainProcessCtx};
use crate::infrastructure::core::{BatchId, DeviceId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    CaptureControlMessage, ConnectionAction, ConnectionControlMessage, ConnectionStatusKind,
    DispatchSource, IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    RunTarget, RuntimeDispatchPhase, RuntimeQueueItem, SessionControlMessage,
};
use crate::infrastructure::logging::log_trait::Log;
use chrono::{Days, Local, NaiveTime, TimeZone};
use runtime_engine::infrastructure::devices::device_launcher::start_device_process;
use std::sync::{Arc, OnceLock};
use tauri::{command, Manager};

static AUTO_DISPATCH_NOTIFY: OnceLock<Arc<tokio::sync::Notify>> = OnceLock::new();

pub(crate) fn notify_auto_dispatch_planner() {
    if let Some(notify) = AUTO_DISPATCH_NOTIFY.get() {
        notify.notify_one();
    }
}

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
    let session =
        load_runtime_session_for_queue_item(app_handle, device_id, queue_item.clone()).await?;
    dispatch_session_to_child(device_id, session, queue_item.dispatch_id).await
}

async fn dispatch_session_to_child(
    device_id: DeviceId,
    session: crate::infrastructure::ipc::message::RuntimeSessionSnapshot,
    dispatch_id: crate::infrastructure::core::DispatchId,
) -> Result<(), String> {
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;
    send_process_control(device_id, ProcessAction::Start);
    DispatchPlanner::init().mark_active_dispatch(device_id, Some(dispatch_id))?;
    Ok(())
}

fn to_planner_queue_item(mut queue_item: RuntimeQueueItem) -> RuntimeQueueItem {
    queue_item.dispatch_source = DispatchSource::Planner;
    queue_item
}

fn parse_hhmm(value: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .map_err(|error| format!("解析时间模板时间失败[{}]: {}", value, error))
}

fn compute_next_due_from_template(
    template: &TimeTemplate,
    now: chrono::DateTime<Local>,
) -> Result<Option<chrono::DateTime<Local>>, String> {
    let Some(start_text) = template.start_time.as_deref() else {
        return Ok(None);
    };
    let start = parse_hhmm(start_text)?;
    let today = now.date_naive();
    let today_due = Local
        .from_local_datetime(&today.and_time(start))
        .single()
        .ok_or_else(|| "构造下一次调度时间失败".to_string())?;
    if today_due > now {
        return Ok(Some(today_due));
    }
    Ok(Some(
        Local
            .from_local_datetime(&(today + Days::new(1)).and_time(start))
            .single()
            .ok_or_else(|| "构造下一次调度时间失败".to_string())?,
    ))
}

fn dispatch_priority(source: &DispatchSource) -> u8 {
    match source {
        DispatchSource::Debug => 3,
        DispatchSource::User => 2,
        DispatchSource::Planner => 1,
    }
}

async fn load_time_template_by_id(
    template_id: crate::infrastructure::core::TemplateId,
) -> Result<Option<TimeTemplate>, String> {
    sqlx::query_as::<_, TimeTemplate>(
        "SELECT id, name, start_time, end_time FROM time_templates WHERE id = ?",
    )
    .bind(template_id.to_string())
    .fetch_optional(get_pool())
    .await
    .map_err(|error| error.to_string())
}

fn schedule_trigger_source(record: &AssignmentSchedule) -> Result<AssignmentTriggerSource, String> {
    match record.trigger_source.as_str() {
        "planner" => Ok(AssignmentTriggerSource::Planner),
        "user" => Ok(AssignmentTriggerSource::User),
        "debug" => Ok(AssignmentTriggerSource::Debug),
        value => Err(format!("未知 dispatch 来源: {}", value)),
    }
}

fn schedule_dispatch_source(record: &AssignmentSchedule) -> Result<DispatchSource, String> {
    match schedule_trigger_source(record)? {
        AssignmentTriggerSource::Planner => Ok(DispatchSource::Planner),
        AssignmentTriggerSource::User => Ok(DispatchSource::User),
        AssignmentTriggerSource::Debug => Ok(DispatchSource::Debug),
    }
}

fn queue_item_matches_schedule(item: &RuntimeQueueItem, record: &AssignmentSchedule) -> bool {
    record.assignment_id == Some(item.assignment_id)
        && record.window_start_at == item.window_start_at
        && record.scope_hash == item.dedup_scope_base_hash
}

async fn ensure_planner_batch_for_device(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<usize, String> {
    ensure_device_ready(app_handle, device_id).await?;
    let queue = load_runtime_queue_for_current_window(device_id)
        .await?
        .into_iter()
        .map(to_planner_queue_item)
        .collect::<Vec<_>>();
    if queue.is_empty() {
        return Ok(0);
    }
    if has_complete_assignment_schedule_batch(device_id, AssignmentTriggerSource::Planner, &queue)
        .await?
    {
        return Ok(0);
    }
    insert_assignment_schedule_batch(
        device_id,
        AssignmentTriggerSource::Planner,
        &queue,
        Some("planner 生成当前批次".to_string()),
    )
    .await?;
    Ok(queue.len())
}

async fn dispatch_planner_schedule_to_child(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    record: AssignmentSchedule,
) -> Result<(), String> {
    let queue = load_runtime_queue_for_current_window(device_id).await?;
    let Some(mut queue_item) = queue
        .into_iter()
        .find(|item| queue_item_matches_schedule(item, &record))
    else {
        update_assignment_schedule_status(
            record.id,
            AssignmentScheduleStatus::Cancelled,
            None,
            Some(Local::now().to_rfc3339()),
            Some("当前 assignment/window/scope 已不存在，取消派发".to_string()),
        )
        .await?;
        return Err("调度记录已过期，已取消".to_string());
    };
    queue_item.dispatch_id = record.dispatch_id;
    queue_item.dispatch_source = schedule_dispatch_source(&record)?;
    queue_item.order_index = record.order_index;
    update_assignment_schedule_status(
        record.id,
        AssignmentScheduleStatus::Dispatched,
        None,
        None,
        Some("dispatch 已派发到子进程".to_string()),
    )
    .await?;
    dispatch_queue_item_to_child(app_handle, device_id, queue_item).await
}

async fn dispatch_user_schedule_to_child(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    record: AssignmentSchedule,
) -> Result<(), String> {
    let Some(run_target_json) = record.run_target_json.as_deref() else {
        return Err("user 调度记录缺少 run_target_json".to_string());
    };
    let target: RunTarget = serde_json::from_str(run_target_json)
        .map_err(|error| format!("解析 user 调度运行目标失败: {}", error))?;
    let mut session = load_runtime_session_for_target(app_handle, device_id, target).await?;
    if let Some(queue_item) = session.queue.first_mut() {
        queue_item.dispatch_id = record.dispatch_id;
        queue_item.dispatch_source = DispatchSource::User;
        if let Some(assignment_id) = record.assignment_id {
            queue_item.assignment_id = assignment_id;
        }
    }
    update_assignment_schedule_status(
        record.id,
        AssignmentScheduleStatus::Dispatched,
        None,
        None,
        Some("user dispatch 已派发到子进程".to_string()),
    )
    .await?;
    dispatch_session_to_child(device_id, session, record.dispatch_id).await
}

async fn dispatch_schedule_to_child(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    record: AssignmentSchedule,
) -> Result<(), String> {
    match schedule_trigger_source(&record)? {
        AssignmentTriggerSource::User => {
            dispatch_user_schedule_to_child(app_handle, device_id, record).await
        }
        AssignmentTriggerSource::Planner => {
            dispatch_planner_schedule_to_child(app_handle, device_id, record).await
        }
        AssignmentTriggerSource::Debug => Err("debug 调度不应持久化".to_string()),
    }
}

async fn dispatch_next_scheduled_queue_item(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<bool, String> {
    if let Some(session) = DispatchPlanner::init().pop_debug_session(device_id)? {
        let dispatch_id = session
            .queue
            .first()
            .map(|queue_item| queue_item.dispatch_id)
            .ok_or_else(|| "debug session 缺少 dispatch 队列项".to_string())?;
        dispatch_session_to_child(device_id, session, dispatch_id).await?;
        return Ok(true);
    }

    for _ in 0..8 {
        let record = match load_next_planned_assignment_schedule(device_id).await? {
            Some(record) => Some(record),
            None => {
                let _ = ensure_planner_batch_for_device(app_handle, device_id).await?;
                load_next_planned_assignment_schedule(device_id).await?
            }
        };
        let Some(record) = record else {
            DispatchPlanner::init().mark_active_dispatch(device_id, None)?;
            return Ok(false);
        };
        match dispatch_schedule_to_child(app_handle, device_id, record).await {
            Ok(()) => return Ok(true),
            Err(error) if error == "调度记录已过期，已取消" => continue,
            Err(error) => return Err(error),
        }
    }
    DispatchPlanner::init().mark_active_dispatch(device_id, None)?;
    Ok(false)
}

pub(crate) async fn reevaluate_device_auto_dispatch(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<usize, String> {
    let device = load_device_table(device_id).await?;
    if !device.data.0.enable || !device.data.0.auto_start {
        return Ok(0);
    }

    ensure_device_ready(app_handle, device_id).await?;

    let planner = DispatchPlanner::init();
    planner.ensure_device_state(device_id)?;
    let state = planner.snapshot_device_state(device_id)?;
    if state.active_dispatch.is_some() {
        return Ok(0);
    }

    let created = ensure_planner_batch_for_device(app_handle, device_id).await?;
    let dispatched = dispatch_next_scheduled_queue_item(app_handle, device_id).await?;
    Ok(if dispatched { created.max(1) } else { created })
}

pub(crate) async fn reevaluate_all_auto_dispatches(
    app_handle: &tauri::AppHandle,
) -> Result<usize, String> {
    let devices = DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await?;
    let mut total = 0usize;
    for device in devices {
        total += reevaluate_device_auto_dispatch(app_handle, device.id).await?;
    }
    Ok(total)
}

async fn compute_next_auto_due_at() -> Result<Option<chrono::DateTime<Local>>, String> {
    let devices = DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await?;
    let now = Local::now();
    let mut next_due: Option<chrono::DateTime<Local>> = None;

    for device in devices {
        if !device.data.0.enable || !device.data.0.auto_start {
            continue;
        }
        let query = "SELECT id, device_id, script_id, time_template_id, account_data, `index` FROM device_script_assignments WHERE device_id = ? AND time_template_id IS NOT NULL";
        let assignments = sqlx::query_as::<
            _,
            crate::domain::devices::device_schedule::DeviceScriptAssignment,
        >(query)
        .bind(device.id.to_string())
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

        for assignment in assignments {
            let Some(template_id) = assignment.time_template_id else {
                continue;
            };
            let Some(template) = load_time_template_by_id(template_id).await? else {
                continue;
            };
            let Some(candidate) = compute_next_due_from_template(&template, now)? else {
                continue;
            };
            next_due = match next_due {
                Some(current) if current <= candidate => Some(current),
                _ => Some(candidate),
            };
        }
    }

    Ok(next_due)
}

pub(crate) fn spawn_auto_dispatch_planner_loop(app_handle: tauri::AppHandle) {
    let notify = AUTO_DISPATCH_NOTIFY
        .get_or_init(|| Arc::new(tokio::sync::Notify::new()))
        .clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let sleep_duration = match compute_next_auto_due_at().await {
                Ok(Some(next_due)) => {
                    let now = Local::now();
                    (next_due - now)
                        .to_std()
                        .unwrap_or_else(|_| std::time::Duration::from_secs(1))
                }
                Ok(None) => std::time::Duration::from_secs(300),
                Err(error) => {
                    Log::error(&format!(
                        "[ process ] 计算下一次自动调度时间失败: {}",
                        error
                    ));
                    std::time::Duration::from_secs(60)
                }
            };
            tokio::select! {
                _ = tokio::time::sleep(sleep_duration) => {}
                _ = notify.notified() => {}
            }
            match reevaluate_all_auto_dispatches(&app_handle).await {
                Ok(count) if count > 0 => {
                    Log::info(&format!("[ process ] 自动调度派发了 {} 个队列项", count));
                }
                Ok(_) => {}
                Err(error) => {
                    Log::error(&format!("[ process ] 自动调度 reevaluate 失败: {}", error));
                }
            }
        }
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

fn should_launch_emulator_in_main(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<bool, String> {
    let state = app_handle.state::<MainProcessCtx>();
    let guard = state
        .device_connections
        .read()
        .map_err(|_| "读取连接状态失败".to_string())?;
    Ok(!matches!(
        guard.get(&device_id).map(|item| &item.status),
        Some(ConnectionStatusKind::Connected)
    ))
}

async fn ensure_emulator_launch_ready_in_main(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    device_config: &DeviceConfig,
) -> Result<bool, String> {
    if !device_config.uses_emulator_transport() {
        return Ok(false);
    }

    if !should_launch_emulator_in_main(app_handle, device_id)? {
        return Ok(false);
    }

    if device_config
        .exe_path
        .as_deref()
        .is_none_or(|path| path.trim().is_empty())
    {
        return Err(
            "当前设备为模拟器 TCP连接，但未填写设备启动路径，无法在主线程自动启动模拟器"
                .to_string(),
        );
    }

    start_device_process(device_config).await?;
    Ok(true)
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
    device_config: &DeviceConfig,
    started_in_main: bool,
) -> Result<(), String> {
    set_connection_status(
        app_handle,
        device_id,
        ConnectionStatusKind::Checking,
        Some("正在准备设备连接".to_string()),
    )
    .await?;
    let action = if started_in_main {
        ConnectionAction::EnsureReadyAfterLaunch
    } else {
        ConnectionAction::EnsureReady
    };
    send_connection_control(device_id, action).await;

    let timeout = if device_config.uses_emulator_transport() {
        std::time::Duration::from_secs(device_config.startup_delay_secs + 65)
    } else {
        std::time::Duration::from_secs(25)
    };

    match wait_for_connection_status(app_handle, device_id, timeout).await? {
        (ConnectionStatusKind::Connected, _) => Ok(()),
        (ConnectionStatusKind::Disconnected, message) => {
            Err(message.unwrap_or_else(|| format!("设备[{}]连接准备失败", device_id)))
        }
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

    let started_in_main =
        ensure_emulator_launch_ready_in_main(app_handle, device_id, &device_table.data.0).await?;
    ensure_device_online(app_handle, device_id).await?;
    ensure_device_connection_ready(app_handle, device_id, &device_table.data.0, started_in_main)
        .await
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

    ensure_device_connection_ready(app_handle, device_id, &device_table.data.0, false).await?;
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
    let created = ensure_planner_batch_for_device(&app_handle, device_id).await?;
    let state = DispatchPlanner::init().snapshot_device_state(device_id)?;
    if state.active_dispatch.is_some() {
        return Ok(format!(
            "设备[{}]当前已有运行中的 dispatch，已唤醒 planner",
            device_id
        ));
    }
    let dispatched = dispatch_next_scheduled_queue_item(&app_handle, device_id).await?;
    if dispatched {
        Ok(format!(
            "已唤醒设备[{}]调度，新增 {} 条 planner 记录并开始执行下一项",
            device_id, created
        ))
    } else {
        Ok(format!(
            "设备[{}]当前时间窗口下没有可运行的 planner 记录",
            device_id
        ))
    }
}

#[command]
pub async fn cmd_device_stop(device_id: DeviceId) -> Result<String, String> {
    send_process_control(device_id, ProcessAction::Stop);
    let _ = DispatchPlanner::init().clear_device_state(device_id);
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
    let device = load_device_table(device_id).await?;
    let state = DispatchPlanner::init().snapshot_device_state(device_id)?;
    let mut created = 0usize;
    let mut dispatched = false;
    if device.data.0.auto_start && state.active_dispatch.is_none() {
        created = ensure_planner_batch_for_device(&app_handle, device_id).await?;
        dispatched = dispatch_next_scheduled_queue_item(&app_handle, device_id).await?;
    }
    Ok(format!(
        "已同步设备[{}]运行会话，新增 planner 记录 {} 条，派发下一项={}",
        device_id, created, dispatched
    ))
}

#[command]
pub async fn cmd_run_script_target(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    target: RunTarget,
) -> Result<String, String> {
    ensure_device_ready(&app_handle, device_id).await?;
    let planner = DispatchPlanner::init();
    planner.ensure_device_state(device_id)?;
    let state = planner.snapshot_device_state(device_id)?;
    let mut session =
        load_runtime_session_for_target(&app_handle, device_id, target.clone()).await?;
    for queue_item in &mut session.queue {
        queue_item.dispatch_source = DispatchSource::Debug;
    }

    if state.active_dispatch.is_some() {
        planner.push_debug_session(device_id, session)?;
        Log::info(&format!(
            "[ process ] 设备[{}]调试运行已加入内存队列，request-next 优先级={}",
            device_id,
            dispatch_priority(&DispatchSource::Debug)
        ));
        return Ok(format!(
            "设备[{}]正在运行，已加入调试队列并将在当前 dispatch 后优先执行: {:?}",
            device_id, target
        ));
    }

    let dispatch_id = session
        .queue
        .first()
        .map(|queue_item| queue_item.dispatch_id)
        .ok_or_else(|| "调试运行目标未生成可派发队列项".to_string())?;
    dispatch_session_to_child(device_id, session, dispatch_id).await?;
    Ok(format!("已向设备[{}]发送运行目标: {:?}", device_id, target))
}

#[command]
pub async fn cmd_run_user_script_target(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    target: RunTarget,
) -> Result<String, String> {
    ensure_device_ready(&app_handle, device_id).await?;
    let mut session =
        load_runtime_session_for_target(&app_handle, device_id, target.clone()).await?;
    let Some(first_item) = session.queue.first_mut() else {
        return Err("临时运行目标未生成可派发队列项".to_string());
    };
    first_item.dispatch_source = DispatchSource::User;
    let dispatch_id = first_item.dispatch_id;
    let script_id = first_item.script_id;
    let run_target_json = serde_json::to_string(&target)
        .map_err(|error| format!("序列化临时运行目标失败: {}", error))?;

    let record = insert_assignment_schedule(
        BatchId::new_v7(),
        device_id,
        None,
        Some(script_id),
        None,
        None,
        String::new(),
        dispatch_id,
        0,
        Local::now().to_rfc3339(),
        Some(run_target_json),
        AssignmentScheduleStatus::Planned,
        AssignmentTriggerSource::User,
        Some("任务管理页临时运行".to_string()),
    )
    .await?;

    update_assignment_schedule_status(
        record.id,
        AssignmentScheduleStatus::Dispatched,
        None,
        None,
        Some("user dispatch 已派发到子进程".to_string()),
    )
    .await?;
    DispatchPlanner::init().clear_device_state(device_id)?;
    dispatch_session_to_child(device_id, session, dispatch_id).await?;
    Ok(format!(
        "已向设备[{}]发送临时运行目标: {:?}",
        device_id, target
    ))
}

#[command]
pub async fn cmd_restart_device_runtime(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let message = restart_device_runtime_internal(&app_handle, device_id).await?;
    let _ = reevaluate_device_auto_dispatch(&app_handle, device_id).await;
    Ok(message)
}

#[command]
pub async fn cmd_device_shutdown(device_id: DeviceId) -> Result<String, String> {
    if let Some(manager) = get_process_manager() {
        manager.stop_child(&device_id).await?;
        let _ = DispatchPlanner::init().clear_device_state(device_id);
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
    let _ = reevaluate_device_auto_dispatch(&app_handle, device_id).await;

    Ok(format!("设备[{}]({})子进程已启动", device_name, device_id))
}

#[command]
pub async fn cmd_bootstrap_enabled_devices(app_handle: tauri::AppHandle) -> Result<String, String> {
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
    Ok(format!(
        "设备[{}]({})已启动并完成连接准备",
        device_name, device_id
    ))
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
