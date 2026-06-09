use super::bundle_loader::load_runtime_queue_for_current_window;
use super::dispatch_planner::{
    fail_active_assignment_schedules_by_device, has_complete_assignment_schedule_batch,
    insert_assignment_schedule, insert_assignment_schedule_batch,
    load_next_planned_assignment_schedule, reactivate_stopped_planner_schedules_for_device,
    stop_active_assignment_schedules_by_device, stop_planned_planner_schedules_by_device,
    update_assignment_schedule_status, update_assignment_schedule_status_by_dispatch_id,
};
use super::runtime_session::{
    build_child_init_data, load_device_table, load_runtime_session_for_queue_item,
    load_runtime_session_for_target, validate_runtime_platform_supported,
};
use crate::constant::project::MAIN_WINDOW;
use crate::constant::table_name::{ASSIGNMENT_TABLE, DEVICE_TABLE};
use crate::domain::devices::device_conf::DeviceTable;
use crate::domain::devices::device_runtime_event::{
    DeviceAssignmentScheduleChangedEventPayload, DeviceConnectionEventPayload,
    DeviceProgressEventPayload, DeviceRuntimeProgressPhase, DeviceRuntimeReconcileAction,
    DeviceRuntimeReconcileEventPayload, DeviceRuntimeReconcileJobType, DeviceRuntimeReconcilePhase,
};
use crate::domain::devices::device_schedule::{
    AssignmentSchedule, AssignmentScheduleStatus, AssignmentTriggerSource,
};
use crate::domain::schedule::time_template::TimeTemplate;
use crate::infrastructure::context::child_process_manager::{
    get_process_manager, set_child_process_exit_handler,
};
use crate::infrastructure::context::main_process::{
    ChildRuntimeStatus, DeviceDispatchSignal, MainProcessCtx, RuntimeReconcileJob,
};
use crate::infrastructure::core::{BatchId, DeviceId, ScriptId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    CaptureControlMessage, ConnectionAction, ConnectionControlMessage, ConnectionStatusKind,
    DispatchSource, IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    RunTarget, RuntimeDispatchPhase, RuntimeQueueItem, SessionControlMessage,
};
use crate::infrastructure::logging::log_trait::Log;
use chrono::{Days, Local, NaiveTime, TimeZone};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tauri::{command, Emitter, Manager};

static AUTO_DISPATCH_NOTIFY: OnceLock<Arc<tokio::sync::Notify>> = OnceLock::new();
const EMULATOR_CONNECTION_READY_GRACE_SECS: u64 = 65;
const DEVICE_CONNECTION_READY_TIMEOUT_SECS: u64 = 25;
const DEVICE_RUNTIME_RECONCILE_EVENT: &str = "device-runtime-reconcile";
const ASSIGNMENT_SCHEDULE_CHANGED_EVENT: &str = "assignment-schedule-changed";

fn now_millis_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn emit_device_connection_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    status: &ConnectionStatusKind,
    message: Option<&str>,
) {
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let payload = DeviceConnectionEventPayload {
            device_id,
            status: status.clone(),
            message: message.map(str::to_string),
            at: now_millis_string(),
        };
        let _ = main_window.emit("device-connection-status", payload);
    }
}

fn emit_device_progress_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    phase: DeviceRuntimeProgressPhase,
    message: impl Into<String>,
) {
    let message = message.into();
    let at = Local::now().to_rfc3339();
    let _ = app_handle.state::<MainProcessCtx>().set_device_progress(
        device_id,
        serde_json::to_value(&phase)
            .ok()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_default(),
        message.clone(),
        Some(at.clone()),
    );
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let payload = DeviceProgressEventPayload {
            device_id,
            session_id: None,
            assignment_id: None,
            script_id: None,
            task_id: None,
            step_id: None,
            phase,
            message: Some(message),
            at,
        };
        let _ = main_window.emit("device-progress", payload);
    }
}

pub(crate) fn emit_assignment_schedule_changed(app_handle: &tauri::AppHandle, device_id: DeviceId) {
    let payload = DeviceAssignmentScheduleChangedEventPayload {
        device_id,
        at: Local::now().to_rfc3339(),
    };
    let _ = app_handle.emit(ASSIGNMENT_SCHEDULE_CHANGED_EVENT, payload);
}

fn ensure_device_dispatch_state(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .ensure_device_runtime_state(device_id)
}

fn snapshot_device_dispatch_state(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<crate::infrastructure::context::main_process::DeviceDispatchState, String> {
    app_handle
        .state::<MainProcessCtx>()
        .snapshot_device_dispatch_state(device_id)
}

fn mark_active_dispatch(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    dispatch_id: Option<crate::infrastructure::core::DispatchId>,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .mark_active_dispatch(device_id, dispatch_id)
}

fn set_auto_dispatch_blocked(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    blocked: bool,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .set_auto_dispatch_blocked(device_id, blocked)
}

fn push_debug_session(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    session: crate::infrastructure::ipc::message::RuntimeSessionSnapshot,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .push_debug_session(device_id, session)
}

fn pop_debug_session(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<Option<crate::infrastructure::ipc::message::RuntimeSessionSnapshot>, String> {
    app_handle
        .state::<MainProcessCtx>()
        .pop_debug_session(device_id)
}

fn reset_device_dispatch_state(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .reset_device_dispatch_state(device_id)
}

pub(crate) fn notify_auto_dispatch_planner() {
    if let Some(notify) = AUTO_DISPATCH_NOTIFY.get() {
        notify.notify_one();
    }
}

fn emit_runtime_reconcile_event(
    app_handle: &tauri::AppHandle,
    job: &RuntimeReconcileJob,
    phase: DeviceRuntimeReconcilePhase,
    action: Option<DeviceRuntimeReconcileAction>,
    message: Option<String>,
) {
    let payload = DeviceRuntimeReconcileEventPayload {
        job_id: job.job_id().to_string(),
        job_type: match job.job_type() {
            "deviceConfig" => DeviceRuntimeReconcileJobType::DeviceConfig,
            "deviceSessionRefresh" => DeviceRuntimeReconcileJobType::DeviceSessionRefresh,
            _ => DeviceRuntimeReconcileJobType::DeviceSessionRefresh,
        },
        device_id: job.device_id(),
        phase,
        action,
        message,
        at: Local::now().to_rfc3339(),
    };
    let _ = app_handle.emit(DEVICE_RUNTIME_RECONCILE_EVENT, payload);
}

fn runtime_job_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

pub(crate) async fn load_assigned_device_ids_by_script(
    script_id: ScriptId,
) -> Result<Vec<DeviceId>, String> {
    let query = format!(
        "SELECT DISTINCT device_id FROM {} WHERE script_id = ? ORDER BY device_id ASC",
        ASSIGNMENT_TABLE
    );
    let rows = sqlx::query_scalar::<_, String>(&query)
        .bind(script_id.to_string())
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

    rows.into_iter()
        .map(|value| {
            uuid::Uuid::parse_str(&value)
                .map(DeviceId::from)
                .map_err(|error| error.to_string())
        })
        .collect()
}

pub(crate) async fn load_assigned_device_ids_by_time_template(
    template_id: &str,
) -> Result<Vec<DeviceId>, String> {
    let query = format!(
        "SELECT DISTINCT device_id FROM {} WHERE time_template_id = ? ORDER BY device_id ASC",
        ASSIGNMENT_TABLE
    );
    let rows = sqlx::query_scalar::<_, String>(&query)
        .bind(template_id)
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

    rows.into_iter()
        .map(|value| {
            uuid::Uuid::parse_str(&value)
                .map(DeviceId::from)
                .map_err(|error| error.to_string())
        })
        .collect()
}

pub(crate) fn enqueue_device_config_reconcile_job(
    app_handle: &tauri::AppHandle,
    previous: Option<DeviceTable>,
    current: DeviceTable,
) -> Result<(), String> {
    let job = RuntimeReconcileJob::DeviceConfig {
        job_id: runtime_job_id(),
        device_id: current.id,
        previous,
        current,
    };
    app_handle
        .state::<MainProcessCtx>()
        .runtime_reconcile_tx
        .send(job.clone())
        .map_err(|error| error.to_string())?;
    emit_runtime_reconcile_event(
        app_handle,
        &job,
        DeviceRuntimeReconcilePhase::Queued,
        None,
        None,
    );
    Ok(())
}

pub(crate) fn enqueue_device_runtime_session_refresh_jobs(
    app_handle: &tauri::AppHandle,
    device_ids: impl IntoIterator<Item = DeviceId>,
    sync_session: bool,
    reevaluate_dispatch: bool,
    reason: impl Into<String>,
) -> Result<(), String> {
    let reason = reason.into();
    let mut seen = std::collections::HashSet::new();
    for device_id in device_ids {
        if !seen.insert(device_id) {
            continue;
        }
        let job = RuntimeReconcileJob::DeviceSessionRefresh {
            job_id: runtime_job_id(),
            device_id,
            sync_session,
            reevaluate_dispatch,
            reason: reason.clone(),
        };
        app_handle
            .state::<MainProcessCtx>()
            .runtime_reconcile_tx
            .send(job.clone())
            .map_err(|error| error.to_string())?;
        emit_runtime_reconcile_event(
            app_handle,
            &job,
            DeviceRuntimeReconcilePhase::Queued,
            None,
            None,
        );
    }
    Ok(())
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

async fn probe_device_connection(
    app_handle: &tauri::AppHandle,
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
    dispatch_session_to_child(app_handle, device_id, session, queue_item.dispatch_id).await
}

async fn dispatch_session_to_child(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    session: crate::infrastructure::ipc::message::RuntimeSessionSnapshot,
    dispatch_id: crate::infrastructure::core::DispatchId,
) -> Result<(), String> {
    send_session_control(device_id, SessionControlMessage::LoadSession { session }).await;
    send_process_control(device_id, ProcessAction::Start);
    mark_active_dispatch(app_handle, device_id, Some(dispatch_id))?;
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

async fn block_device_auto_dispatch(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    message: String,
) -> Result<(), String> {
    set_auto_dispatch_blocked(app_handle, device_id, true)?;
    let stopped =
        stop_planned_planner_schedules_by_device(device_id, Local::now().to_rfc3339(), message)
            .await?;
    if stopped > 0 {
        emit_assignment_schedule_changed(app_handle, device_id);
    }
    Ok(())
}

async fn ensure_planner_batch_for_device(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    preserve_stopped: bool,
) -> Result<usize, String> {
    emit_device_progress_status(
        app_handle,
        device_id,
        DeviceRuntimeProgressPhase::Planning,
        "正在生成当前窗口调度记录",
    );
    let queue = load_runtime_queue_for_current_window(device_id)
        .await?
        .into_iter()
        .map(to_planner_queue_item)
        .collect::<Vec<_>>();
    if queue.is_empty() {
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::Idle,
            "当前设备无可运行队列",
        );
        return Ok(0);
    }
    if has_complete_assignment_schedule_batch(device_id, AssignmentTriggerSource::Planner, &queue)
        .await?
    {
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::Planning,
            "已生成/已补齐当前窗口调度记录",
        );
        return Ok(0);
    }
    insert_assignment_schedule_batch(
        device_id,
        AssignmentTriggerSource::Planner,
        &queue,
        Some("planner 生成当前批次".to_string()),
        preserve_stopped,
    )
    .await?;
    emit_assignment_schedule_changed(app_handle, device_id);
    emit_device_progress_status(
        app_handle,
        device_id,
        DeviceRuntimeProgressPhase::Planning,
        "已生成/已补齐当前窗口调度记录",
    );
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
    dispatch_session_to_child(app_handle, device_id, session, record.dispatch_id).await
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
    if let Some(session) = pop_debug_session(app_handle, device_id)? {
        let dispatch_id = session
            .queue
            .first()
            .map(|queue_item| queue_item.dispatch_id)
            .ok_or_else(|| "debug session 缺少 dispatch 队列项".to_string())?;
        dispatch_session_to_child(app_handle, device_id, session, dispatch_id).await?;
        return Ok(true);
    }

    for _ in 0..8 {
        let record = match load_next_planned_assignment_schedule(device_id).await? {
            Some(record) => Some(record),
            None => {
                let _ = ensure_planner_batch_for_device(app_handle, device_id, true).await?;
                load_next_planned_assignment_schedule(device_id).await?
            }
        };
        let Some(record) = record else {
            mark_active_dispatch(app_handle, device_id, None)?;
            return Ok(false);
        };
        if let Err(error) = ensure_device_ready(app_handle, device_id).await {
            update_assignment_schedule_status(
                record.id,
                AssignmentScheduleStatus::Failed,
                None,
                Some(Local::now().to_rfc3339()),
                Some(error.clone()),
            )
            .await?;
            emit_assignment_schedule_changed(app_handle, device_id);
            mark_active_dispatch(app_handle, device_id, None)?;
            block_device_auto_dispatch(
                app_handle,
                device_id,
                "设备连接失败，停止该设备后续自动派发".to_string(),
            )
            .await?;
            emit_device_progress_status(
                app_handle,
                device_id,
                DeviceRuntimeProgressPhase::Failed,
                "设备连接失败，已停止该设备后续自动派发，可手动重试；其它设备继续运行",
            );
            return Err(error);
        }
        match dispatch_schedule_to_child(app_handle, device_id, record).await {
            Ok(()) => return Ok(true),
            Err(error) if error == "调度记录已过期，已取消" => continue,
            Err(error) => return Err(error),
        }
    }
    mark_active_dispatch(app_handle, device_id, None)?;
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

    ensure_device_dispatch_state(app_handle, device_id)?;
    let state = snapshot_device_dispatch_state(app_handle, device_id)?;
    if state.auto_dispatch_blocked {
        emit_device_progress_status(
            app_handle,
            device_id,
            DeviceRuntimeProgressPhase::Failed,
            "该设备自动派发已因设备连接失败暂停，等待手动运行重新尝试",
        );
        return Ok(0);
    }
    if state.active_dispatch.is_some() {
        return Ok(0);
    }

    let created = ensure_planner_batch_for_device(app_handle, device_id, true).await?;
    if load_next_planned_assignment_schedule(device_id)
        .await?
        .is_none()
    {
        return Ok(created);
    }
    let dispatched = dispatch_next_scheduled_queue_item(app_handle, device_id).await?;
    Ok(if dispatched { created.max(1) } else { created })
}

pub(crate) async fn reevaluate_all_auto_dispatches(
    app_handle: &tauri::AppHandle,
) -> Result<usize, String> {
    let devices = DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await?;
    let mut total = 0usize;
    for device in devices {
        match reevaluate_device_auto_dispatch(app_handle, device.id).await {
            Ok(count) => total += count,
            Err(error) => {
                Log::error(&format!(
                    "[ process ] 设备[{}]自动派发失败，继续处理其它设备: {}",
                    device.id, error
                ));
            }
        }
    }
    Ok(total)
}

async fn sync_device_runtime_session_if_online(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<Option<String>, String> {
    let Some(manager) = get_process_manager() else {
        return Ok(None);
    };

    if !manager.is_running(&device_id).await {
        return Ok(None);
    }

    cmd_sync_device_runtime_session(app_handle.clone(), device_id)
        .await
        .map(Some)
}

async fn reconcile_saved_device_runtime(
    app_handle: &tauri::AppHandle,
    previous: Option<DeviceTable>,
    current: DeviceTable,
) -> Result<(Option<DeviceRuntimeReconcileAction>, String), String> {
    let Some(manager) = get_process_manager() else {
        return Ok((None, "进程管理器未初始化，跳过设备运行时协调".to_string()));
    };

    let is_running = manager.is_running(&current.id).await;
    let is_enabled = current.data.0.enable;

    if !is_enabled {
        if is_running {
            let message = cmd_device_shutdown(app_handle.clone(), current.id).await?;
            return Ok((Some(DeviceRuntimeReconcileAction::ShuttingDown), message));
        }
        return Ok((None, "设备未启用且子进程未运行，无需协调".to_string()));
    }

    let Some(previous) = previous else {
        return Ok((
            None,
            "新增设备配置已保存，等待调度或临时运行时启动运行时".to_string(),
        ));
    };

    if !is_running {
        return Ok((
            None,
            "设备当前未运行，本次配置变更不自动拉起子进程，等待调度或临时运行".to_string(),
        ));
    }

    if previous.data.0.cores != current.data.0.cores {
        let message = cmd_restart_device_runtime(app_handle.clone(), current.id).await?;
        return Ok((Some(DeviceRuntimeReconcileAction::Restarting), message));
    }

    if previous.data.0.execution_policy != current.data.0.execution_policy {
        let message = cmd_sync_device_runtime_session(app_handle.clone(), current.id).await?;
        return Ok((Some(DeviceRuntimeReconcileAction::Syncing), message));
    }

    if !previous.data.0.auto_start && current.data.0.auto_start {
        let created = reevaluate_device_auto_dispatch(app_handle, current.id).await?;
        return Ok((
            Some(DeviceRuntimeReconcileAction::Syncing),
            format!(
                "设备[{}]已重新评估自动调度，新增/唤醒 {} 项",
                current.id, created
            ),
        ));
    }

    Ok((None, "设备配置未触发运行时协调".to_string()))
}

async fn run_runtime_reconcile_job(
    app_handle: &tauri::AppHandle,
    job: RuntimeReconcileJob,
) -> Result<(Option<DeviceRuntimeReconcileAction>, String), String> {
    match job {
        RuntimeReconcileJob::DeviceConfig {
            previous, current, ..
        } => reconcile_saved_device_runtime(app_handle, previous, current).await,
        RuntimeReconcileJob::DeviceSessionRefresh {
            device_id,
            sync_session,
            reevaluate_dispatch,
            reason,
            ..
        } => {
            let mut messages = vec![format!("reason={}", reason)];
            let mut action = None;

            if sync_session {
                action = Some(DeviceRuntimeReconcileAction::Syncing);
                if let Some(message) =
                    sync_device_runtime_session_if_online(app_handle, device_id).await?
                {
                    messages.push(message);
                } else {
                    messages.push(format!("设备[{}]当前未在线，跳过运行会话同步", device_id));
                }
            }

            if reevaluate_dispatch {
                action = Some(DeviceRuntimeReconcileAction::Syncing);
                let created = reevaluate_device_auto_dispatch(app_handle, device_id).await?;
                messages.push(format!(
                    "设备[{}]已重新评估自动调度，新增/唤醒 {} 项",
                    device_id, created
                ));
            }

            Ok((action, messages.join("；")))
        }
    }
}

pub(crate) fn spawn_runtime_reconcile_loop(
    app_handle: tauri::AppHandle,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<RuntimeReconcileJob>,
) {
    let device_locks: Arc<tokio::sync::Mutex<HashMap<DeviceId, Arc<tokio::sync::Mutex<()>>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    tauri::async_runtime::spawn(async move {
        while let Some(job) = rx.recv().await {
            let app_handle = app_handle.clone();
            let device_locks = device_locks.clone();
            tauri::async_runtime::spawn(async move {
                let device_id = job.device_id();
                let job_for_event = job.clone();
                let device_lock = {
                    let mut guard = device_locks.lock().await;
                    guard
                        .entry(device_id)
                        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                        .clone()
                };
                let _guard = device_lock.lock().await;

                let action_hint = match &job_for_event {
                    RuntimeReconcileJob::DeviceConfig {
                        previous, current, ..
                    } => {
                        let enabled = current.data.0.enable;
                        let action = if !enabled {
                            Some(DeviceRuntimeReconcileAction::ShuttingDown)
                        } else if let Some(previous) = previous {
                            if previous.data.0.cores != current.data.0.cores {
                                Some(DeviceRuntimeReconcileAction::Restarting)
                            } else if previous.data.0.execution_policy
                                != current.data.0.execution_policy
                                || (!previous.data.0.auto_start && current.data.0.auto_start)
                            {
                                Some(DeviceRuntimeReconcileAction::Syncing)
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        action
                    }
                    RuntimeReconcileJob::DeviceSessionRefresh {
                        sync_session,
                        reevaluate_dispatch,
                        ..
                    } => {
                        if *sync_session || *reevaluate_dispatch {
                            Some(DeviceRuntimeReconcileAction::Syncing)
                        } else {
                            None
                        }
                    }
                };
                emit_runtime_reconcile_event(
                    &app_handle,
                    &job_for_event,
                    DeviceRuntimeReconcilePhase::Running,
                    action_hint.clone(),
                    None,
                );

                match run_runtime_reconcile_job(&app_handle, job).await {
                    Ok((action, message)) => {
                        emit_runtime_reconcile_event(
                            &app_handle,
                            &job_for_event,
                            DeviceRuntimeReconcilePhase::Succeeded,
                            action,
                            Some(message),
                        );
                    }
                    Err(error) => {
                        Log::error(&format!(
                            "[ process ] 设备[{}] runtime 协调任务失败 type={} error={}",
                            device_id,
                            job_for_event.job_type(),
                            error
                        ));
                        emit_runtime_reconcile_event(
                            &app_handle,
                            &job_for_event,
                            DeviceRuntimeReconcilePhase::Failed,
                            action_hint,
                            Some(error),
                        );
                    }
                }
            });
        }
    });
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
                    Log::info(&format!("[ process ] 下一次设备任务调度时间：{}", next_due));
                    let now = Local::now();
                    (next_due - now)
                        .to_std()
                        .unwrap_or_else(|_| std::time::Duration::from_secs(1))
                }
                Ok(None) => {
                    Log::info(&format!("[ process ] 暂无自动调度任务,将于24小时后再次检查"));
                    std::time::Duration::from_days(1)
                },
                Err(error) => {
                    Log::error(&format!(
                        "[ process ] 计算下一次自动调度时间失败: {}",
                        error
                    ));
                    std::time::Duration::from_days(1)
                }
            };
            tokio::select! {
                _ = tokio::time::sleep(sleep_duration) => {}
                _ = notify.notified() => {}
            }
            match reevaluate_all_auto_dispatches(&app_handle).await {
                Ok(count) if count > 0 => {
                    Log::info(&format!("[ process ] 派发了 {} 个设备的任务队列", count));
                }
                Ok(_) => {}
                Err(error) => {
                    Log::error(&format!("[ process ] 自动调度派发任务失败: {}", error));
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

async fn ensure_child_runtime_ipc_ready(
    app_handle: &tauri::AppHandle,
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

async fn set_connection_status(
    app_handle: &tauri::AppHandle,
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
    app_handle: &tauri::AppHandle,
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
    app_handle: &tauri::AppHandle,
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

async fn ensure_device_ready(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    if !device_table.data.0.enable {
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
    let timeout = if device_table.data.0.uses_emulator_transport() {
        std::time::Duration::from_secs(
            u64::from(device_table.data.0.startup_delay_secs)
                + EMULATOR_CONNECTION_READY_GRACE_SECS,
        )
    } else {
        std::time::Duration::from_secs(DEVICE_CONNECTION_READY_TIMEOUT_SECS)
    };
    request_child_device_connection(app_handle, device_id, timeout).await
}

async fn ensure_device_ready_for_manual(
    app_handle: &tauri::AppHandle,
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

async fn ensure_device_capture_ready(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let device_name = device_table.data.0.device_name.clone();
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

    let timeout = if device_table.data.0.uses_emulator_transport() {
        std::time::Duration::from_secs(
            u64::from(device_table.data.0.startup_delay_secs)
                + EMULATOR_CONNECTION_READY_GRACE_SECS,
        )
    } else {
        std::time::Duration::from_secs(DEVICE_CONNECTION_READY_TIMEOUT_SECS)
    };
    request_child_device_connection(app_handle, device_id, timeout).await?;
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
            let failed = fail_active_assignment_schedules_by_device(
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

async fn restart_device_runtime_internal(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let was_running = manager.is_running(&device_id).await;

    if was_running {
        manager.stop_child(&device_id).await?;
    }

    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::Starting);
    let init_data = build_child_init_data(app_handle, device_id).await?;
    manager.spawn_child(init_data).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcWaiting);
    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcReady);
    let _ = probe_device_connection(app_handle, device_id, "正在检查设备连接").await;

    Ok(format!("设备[{}]子进程已重启", device_id))
}

#[command]
pub async fn cmd_device_start(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    ensure_device_dispatch_state(&app_handle, device_id)?;
    set_auto_dispatch_blocked(&app_handle, device_id, false)?;
    let state = snapshot_device_dispatch_state(&app_handle, device_id)?;
    if state.active_dispatch.is_some() {
        return Ok(format!(
            "设备[{}]当前已有运行中的 dispatch，已唤醒 planner",
            device_id
        ));
    }
    let reactivated = reactivate_stopped_planner_schedules_for_device(
        device_id,
        Local::now().format("%Y-%m-%d").to_string(),
        "用户重新开始设备调度".to_string(),
    )
    .await?;
    if reactivated > 0 {
        emit_assignment_schedule_changed(&app_handle, device_id);
    }
    let created = ensure_planner_batch_for_device(&app_handle, device_id, false).await?;
    if load_next_planned_assignment_schedule(device_id)
        .await?
        .is_none()
    {
        emit_device_progress_status(
            &app_handle,
            device_id,
            DeviceRuntimeProgressPhase::Idle,
            "当前设备无可运行队列",
        );
        return Ok(format!(
            "设备[{}]当前时间窗口下没有可运行的 planner 记录",
            device_id
        ));
    }
    let dispatched = dispatch_next_scheduled_queue_item(&app_handle, device_id).await?;
    if dispatched {
        Ok(format!(
            "已唤醒设备[{}]调度，新增 {} 条 planner 记录并开始执行下一项",
            device_id,
            created + reactivated as usize
        ))
    } else {
        emit_device_progress_status(
            &app_handle,
            device_id,
            DeviceRuntimeProgressPhase::Idle,
            "当前设备无可运行队列",
        );
        Ok(format!(
            "设备[{}]当前时间窗口下没有可运行的 planner 记录",
            device_id
        ))
    }
}

#[command]
pub async fn cmd_device_stop(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    send_process_control(device_id, ProcessAction::Stop);
    let stopped = stop_active_assignment_schedules_by_device(
        device_id,
        Local::now().to_rfc3339(),
        "用户停止设备调度".to_string(),
    )
    .await?;
    let _ = reset_device_dispatch_state(&app_handle, device_id);
    Ok(format!(
        "已向设备[{}]发送停止命令，并持久化停止 {} 条调度记录",
        device_id, stopped
    ))
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
    let state = snapshot_device_dispatch_state(&app_handle, device_id)?;
    let mut created = 0usize;
    let mut dispatched = false;
    if device.data.0.auto_start && state.active_dispatch.is_none() && !state.auto_dispatch_blocked {
        created = ensure_planner_batch_for_device(&app_handle, device_id, true).await?;
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
    ensure_device_dispatch_state(&app_handle, device_id)?;
    let state = snapshot_device_dispatch_state(&app_handle, device_id)?;
    let mut session =
        load_runtime_session_for_target(&app_handle, device_id, target.clone()).await?;
    for queue_item in &mut session.queue {
        queue_item.dispatch_source = DispatchSource::Debug;
    }

    if state.active_dispatch.is_some() {
        push_debug_session(&app_handle, device_id, session)?;
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

    ensure_device_ready_for_manual(&app_handle, device_id).await?;
    let dispatch_id = session
        .queue
        .first()
        .map(|queue_item| queue_item.dispatch_id)
        .ok_or_else(|| "调试运行目标未生成可派发队列项".to_string())?;
    dispatch_session_to_child(&app_handle, device_id, session, dispatch_id).await?;
    Ok(format!("已向设备[{}]发送运行目标: {:?}", device_id, target))
}

#[command]
pub async fn cmd_run_user_script_target(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    target: RunTarget,
) -> Result<String, String> {
    ensure_device_dispatch_state(&app_handle, device_id)?;
    if snapshot_device_dispatch_state(&app_handle, device_id)?
        .active_dispatch
        .is_some()
    {
        return Err(format!(
            "设备[{}]仍有运行中的 dispatch，请先停止当前设备调度",
            device_id
        ));
    }

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

    if let Err(error) = ensure_device_ready_for_manual(&app_handle, device_id).await {
        update_assignment_schedule_status(
            record.id,
            AssignmentScheduleStatus::Failed,
            None,
            Some(Local::now().to_rfc3339()),
            Some(error.clone()),
        )
        .await?;
        emit_assignment_schedule_changed(&app_handle, device_id);
        return Err(error);
    }

    update_assignment_schedule_status(
        record.id,
        AssignmentScheduleStatus::Dispatched,
        None,
        None,
        Some("user dispatch 已派发到子进程".to_string()),
    )
    .await?;
    emit_assignment_schedule_changed(&app_handle, device_id);
    dispatch_session_to_child(&app_handle, device_id, session, dispatch_id).await?;
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
    notify_auto_dispatch_planner();
    Ok(message)
}

#[command]
pub async fn cmd_device_shutdown(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    if let Some(manager) = get_process_manager() {
        manager.stop_child(&device_id).await?;
        let _ = reset_device_dispatch_state(&app_handle, device_id);
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
    let init_data = build_child_init_data(&app_handle, device_id).await?;
    let device_name = init_data.device_config.device_name.clone();
    let manager = get_process_manager().ok_or_else(|| "进程管理器未初始化".to_string())?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::Starting);
    manager.spawn_child(init_data).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcWaiting);
    wait_for_ipc_client(&app_handle, device_id, std::time::Duration::from_secs(5)).await?;
    let _ = app_handle
        .state::<MainProcessCtx>()
        .set_child_runtime_status(device_id, ChildRuntimeStatus::IpcReady);
    let _ = probe_device_connection(&app_handle, device_id, "正在检查设备连接").await;
    notify_auto_dispatch_planner();

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
            let _ = set_connection_status(
                &app_handle,
                device_id,
                ConnectionStatusKind::DeviceDisconnected,
                Some("设备运行时未启动，跳过连接探测".to_string()),
            )
            .await;
            continue;
        }

        match wait_for_ipc_client(&app_handle, device_id, std::time::Duration::from_secs(2)).await {
            Ok(()) => {
                let _ = set_connection_status(
                    &app_handle,
                    device_id,
                    ConnectionStatusKind::DeviceChecking,
                    Some("正在检查设备连接".to_string()),
                )
                .await;
                send_connection_control(device_id, ConnectionAction::Probe).await;
                queued += 1;
            }
            Err(error) => {
                skipped += 1;
                let _ = set_connection_status(
                    &app_handle,
                    device_id,
                    ConnectionStatusKind::DeviceDisconnected,
                    Some(error.clone()),
                )
                .await;
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
