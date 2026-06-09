use super::events::{emit_assignment_schedule_changed, emit_device_progress_status};
use super::runtime::{
    dispatch_session_to_child, ensure_device_capture_ready, ensure_device_ready_for_manual,
    probe_device_connection, restart_device_runtime_internal, send_capture_control,
    send_process_control, set_connection_status, shutdown_device_runtime_internal,
    spawn_device_runtime_internal, wait_for_capture_result, wait_for_ipc_client,
};
use super::scheduler::{
    dispatch_next_scheduled_queue_item, dispatch_priority, ensure_planner_batch_for_device,
    sync_device_runtime_session_internal,
};
use super::state::{
    ensure_device_dispatch_state, push_debug_session, reset_device_dispatch_state,
    set_auto_dispatch_blocked, snapshot_device_dispatch_state,
};
use super::super::dispatch_planner::{
    insert_assignment_schedule, load_next_planned_assignment_schedule,
    reactivate_stopped_planner_schedules_for_device, stop_active_assignment_schedules_by_device,
    update_assignment_schedule_status,
};
use super::super::runtime_session::{load_runtime_session_for_target};
use crate::constant::table_name::DEVICE_TABLE;
use crate::domain::devices::device_conf::DeviceTable;
use crate::domain::devices::device_runtime_event::DeviceRuntimeProgressPhase;
use crate::domain::devices::device_schedule::{AssignmentScheduleStatus, AssignmentTriggerSource};
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::{BatchId, DeviceId};
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::ipc::message::{
    ConnectionAction, ConnectionStatusKind, DispatchSource, ProcessAction, RunTarget,
};
use crate::infrastructure::logging::log_trait::Log;
use chrono::Local;
use tauri::command;

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
    sync_device_runtime_session_internal(&app_handle, device_id).await
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
    super::state::notify_auto_dispatch_planner();
    Ok(message)
}

#[command]
pub async fn cmd_device_shutdown(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    shutdown_device_runtime_internal(&app_handle, device_id).await
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
    let device_name = spawn_device_runtime_internal(&app_handle, device_id).await?;
    let _ = probe_device_connection(&app_handle, device_id, "正在检查设备连接").await;
    super::state::notify_auto_dispatch_planner();

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
                super::runtime::send_connection_control(device_id, ConnectionAction::Probe).await;
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
