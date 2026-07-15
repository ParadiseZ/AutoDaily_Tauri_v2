use super::super::bundle_loader::{load_runtime_queue_for_current_window, planner_queue_items};
use super::super::runtime_session::{load_device_profile, load_runtime_session_for_target};
use super::events::{
    device_log_label, emit_assignment_schedule_changed, emit_device_progress_status,
};
use super::runtime::{
    dispatch_queue_item_to_child, dispatch_session_to_child, ensure_device_ready,
};
use super::state::{
    ensure_device_dispatch_state, mark_active_dispatch, pop_debug_session,
    set_auto_dispatch_blocked, snapshot_device_dispatch_state,
};
use crate::api::local::execution::DeviceRuntimeProgressPhase;
use crate::infra::logging::log_trait::Log;
use ad_kernel::ids::{DeviceId, TemplateId};
use chrono::{Days, Local, TimeZone};
use domain_schedule::TimeWindow;
use domain_schedule::{AssignmentScheduleProfile, TimeTemplateProfile};
use domain_schedule::{AssignmentScheduleStatus, AssignmentTriggerSource};
use infra_sqlite::{get_all_devices, get_time_template, list_assignments};
use infra_sqlite::{
    has_complete_assignment_schedule_batch, insert_assignment_schedule_batch,
    load_next_planned_assignment_schedule, stop_planned_planner_schedules_by_device,
    update_assignment_schedule_status,
};
use runner_protocol::message::{DispatchSource, RunTarget, RuntimeQueueItem};
use tauri::AppHandle;

fn compute_next_due_from_template(
    template: &TimeTemplateProfile,
    now: chrono::DateTime<Local>,
) -> Result<Option<chrono::DateTime<Local>>, String> {
    let window = TimeWindow::parse(template.start_time.as_deref(), template.end_time.as_deref())
        .map_err(|error| error.to_string())?;
    let Some(start) = window.start() else {
        return Ok(None);
    };
    let today = now.date_naive();
    let today_due = Local
        .from_local_datetime(
            &today
                .and_hms_opt(start.hour() as u32, start.minute() as u32, 0)
                .ok_or_else(|| "构造下一次调度时间失败".to_string())?,
        )
        .single()
        .ok_or_else(|| "构造下一次调度时间失败".to_string())?;
    if today_due > now {
        return Ok(Some(today_due));
    }
    Ok(Some(
        Local
            .from_local_datetime(
                &(today + Days::new(1))
                    .and_hms_opt(start.hour() as u32, start.minute() as u32, 0)
                    .ok_or_else(|| "构造下一次调度时间失败".to_string())?,
            )
            .single()
            .ok_or_else(|| "构造下一次调度时间失败".to_string())?,
    ))
}

pub(super) fn dispatch_priority(source: &DispatchSource) -> u8 {
    match source {
        DispatchSource::Debug => 3,
        DispatchSource::User => 2,
        DispatchSource::Planner => 1,
    }
}

async fn load_time_template_by_id(
    template_id: TemplateId,
) -> Result<Option<TimeTemplateProfile>, String> {
    get_time_template(template_id).await
}

fn schedule_trigger_source(
    record: &AssignmentScheduleProfile,
) -> Result<AssignmentTriggerSource, String> {
    match record.trigger_source.as_str() {
        "planner" => Ok(AssignmentTriggerSource::Planner),
        "user" => Ok(AssignmentTriggerSource::User),
        "debug" => Ok(AssignmentTriggerSource::Debug),
        value => Err(format!("未知 dispatch 来源: {}", value)),
    }
}

fn schedule_dispatch_source(record: &AssignmentScheduleProfile) -> Result<DispatchSource, String> {
    match schedule_trigger_source(record)? {
        AssignmentTriggerSource::Planner => Ok(DispatchSource::Planner),
        AssignmentTriggerSource::User => Ok(DispatchSource::User),
        AssignmentTriggerSource::Debug => Ok(DispatchSource::Debug),
    }
}

fn queue_item_matches_schedule(
    item: &RuntimeQueueItem,
    record: &AssignmentScheduleProfile,
) -> bool {
    record.assignment_id == Some(item.assignment_id)
        && record.window_start_at == item.window_start_at
        && record.scope_hash == item.dedup_scope_base_hash
}

pub(super) async fn block_device_auto_dispatch(
    app_handle: &AppHandle,
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

pub(super) async fn ensure_planner_batch_for_device(
    app_handle: &AppHandle,
    device_id: DeviceId,
    preserve_stopped: bool,
) -> Result<usize, String> {
    emit_device_progress_status(
        app_handle,
        device_id,
        DeviceRuntimeProgressPhase::Planning,
        "正在生成当前窗口调度记录",
    );
    let queue = planner_queue_items(&load_runtime_queue_for_current_window(device_id).await?);
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
    app_handle: &AppHandle,
    device_id: DeviceId,
    record: AssignmentScheduleProfile,
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
    app_handle: &AppHandle,
    device_id: DeviceId,
    record: AssignmentScheduleProfile,
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
    app_handle: &AppHandle,
    device_id: DeviceId,
    record: AssignmentScheduleProfile,
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

pub(super) async fn dispatch_next_scheduled_queue_item(
    app_handle: &AppHandle,
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
                "设备连接失败，已停止该设备后续自动派发，可手动重试",
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

pub(super) async fn reevaluate_device_auto_dispatch(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<usize, String> {
    let device = load_device_profile(device_id).await?;
    if !device.config.enable || !device.config.auto_start {
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

async fn reevaluate_all_auto_dispatches(app_handle: &AppHandle) -> Result<usize, String> {
    let devices = get_all_devices().await?;
    let mut total = 0usize;
    for device in devices {
        match reevaluate_device_auto_dispatch(app_handle, device.id).await {
            Ok(count) => total += count,
            Err(error) => {
                let device_label = device_log_label(app_handle, device.id);
                Log::error(&format!(
                    "[ process ] 设备[{}]自动派发失败，继续处理其它设备: {}",
                    device_label, error
                ));
            }
        }
    }
    Ok(total)
}

pub(super) async fn sync_device_runtime_session_internal(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let device = load_device_profile(device_id).await?;
    let state = snapshot_device_dispatch_state(app_handle, device_id)?;
    let mut created = 0usize;
    if device.config.auto_start && !state.auto_dispatch_blocked {
        created = ensure_planner_batch_for_device(app_handle, device_id, true).await?;
    }
    let device_label = device_log_label(app_handle, device_id);
    Ok(format!(
        "已同步设备[{}]运行会话，新增/补齐 planner 记录 {} 条，不触发自动派发",
        device_label, created
    ))
}

async fn compute_next_auto_due_at() -> Result<Option<chrono::DateTime<Local>>, String> {
    let devices = get_all_devices().await?;
    let now = Local::now();
    let mut next_due: Option<chrono::DateTime<Local>> = None;

    for device in devices {
        if !device.config.enable || !device.config.auto_start {
            continue;
        }
        let assignments = list_assignments(device.id)
            .await?
            .into_iter()
            .filter(|assignment| assignment.time_template_id.is_some())
            .collect::<Vec<_>>();

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
    let notify = super::state::auto_dispatch_notify();
    let reschedule_notify = super::state::auto_dispatch_reschedule_notify();
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
                    Log::info("[ process ] 暂无自动调度任务,将于24小时后再次检查");
                    std::time::Duration::from_secs(24 * 60 * 60)
                }
                Err(error) => {
                    Log::error(&format!(
                        "[ process ] 计算下一次自动调度时间失败: {}",
                        error
                    ));
                    std::time::Duration::from_secs(24 * 60 * 60)
                }
            };
            let mut should_dispatch = true;
            tokio::select! {
                _ = tokio::time::sleep(sleep_duration) => {}
                _ = notify.notified() => {}
                _ = reschedule_notify.notified() => {
                    should_dispatch = false;
                }
            }
            if !should_dispatch {
                continue;
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
