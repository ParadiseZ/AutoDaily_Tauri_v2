use super::events::{device_log_label, emit_runtime_reconcile_event};
use super::runtime::{
    request_child_connection_action, restart_device_runtime_internal,
    send_device_config_update, shutdown_device_runtime_internal, spawn_device_runtime_internal,
};
use super::scheduler::{reevaluate_device_auto_dispatch, sync_device_runtime_session_internal};
use crate::constant::table_name::ASSIGNMENT_TABLE;
use crate::domain::devices::device_conf::DeviceTable;
use crate::domain::devices::device_runtime_event::{
    DeviceRuntimeReconcileAction, DeviceRuntimeReconcilePhase,
};
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::context::main_process::{MainProcessCtx, RuntimeReconcileJob};
use crate::infrastructure::core::{DeviceId, JobId, ScriptId};
use crate::infrastructure::db::get_pool;
use crate::infrastructure::logging::log_trait::Log;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

async fn load_assigned_device_ids_by_field(
    field_name: &str,
    field_value: &str,
) -> Result<Vec<DeviceId>, String> {
    let query = format!(
        "SELECT DISTINCT device_id FROM {} WHERE {} = ? ORDER BY device_id ASC",
        ASSIGNMENT_TABLE, field_name
    );
    let rows = sqlx::query_scalar::<_, String>(&query)
        .bind(field_value)
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

fn enqueue_runtime_reconcile_job(
    app_handle: &AppHandle,
    job: RuntimeReconcileJob,
) -> Result<(), String> {
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

pub(crate) async fn load_assigned_device_ids_by_script(
    script_id: ScriptId,
) -> Result<Vec<DeviceId>, String> {
    let script_id = script_id.to_string();
    load_assigned_device_ids_by_field("script_id", &script_id).await
}

pub(crate) async fn load_assigned_device_ids_by_time_template(
    template_id: &str,
) -> Result<Vec<DeviceId>, String> {
    load_assigned_device_ids_by_field("time_template_id", template_id).await
}

pub(crate) fn enqueue_device_config_reconcile_job(
    app_handle: &AppHandle,
    previous: Option<DeviceTable>,
    current: DeviceTable,
) -> Result<(), String> {
    let job = RuntimeReconcileJob::DeviceConfig {
        job_id: JobId::new_v7(),
        device_id: current.id,
        previous,
        current,
    };
    enqueue_runtime_reconcile_job(app_handle, job)
}

pub(crate) fn enqueue_device_runtime_session_refresh_jobs(
    app_handle: &AppHandle,
    device_ids: impl IntoIterator<Item = DeviceId>,
    sync_session: bool,
    reevaluate_dispatch: bool,
    reason: impl Into<String>,
) -> Result<(), String> {
    let reason = reason.into();
    let mut seen = HashSet::new();
    for device_id in device_ids {
        if !seen.insert(device_id) {
            continue;
        }
        let job = RuntimeReconcileJob::DeviceSessionRefresh {
            job_id: JobId::new_v7(),
            device_id,
            sync_session,
            reevaluate_dispatch,
            reason: reason.clone(),
        };
        enqueue_runtime_reconcile_job(app_handle, job)?;
    }
    Ok(())
}

async fn sync_device_runtime_session_if_online(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<Option<String>, String> {
    let Some(manager) = get_process_manager() else {
        return Ok(None);
    };

    if !manager.is_running(&device_id).await {
        return Ok(None);
    }

    sync_device_runtime_session_internal(app_handle, device_id)
        .await
        .map(Some)
}

async fn reconcile_saved_device_runtime(
    app_handle: &AppHandle,
    previous: Option<DeviceTable>,
    current: DeviceTable,
) -> Result<(Option<DeviceRuntimeReconcileAction>, String), String> {
    let Some(manager) = get_process_manager() else {
        Log::warn("进程管理器未初始化，跳过设备运行时协调");
        return Ok((None, "进程管理器未初始化，跳过设备运行时协调".to_string()));
    };

    let is_running = manager.is_running(&current.id).await;
    let is_enabled = current.data.0.enable;

    if !is_enabled {
        if is_running {
            let message = shutdown_device_runtime_internal(app_handle, current.id).await?;
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

    let previous_config = previous.data.0.clone();
    let current_config = current.data.0.clone();
    let should_probe_connection = previous_config.platform != current_config.platform
        || previous_config.transport_kind != current_config.transport_kind
        || previous_config.startup_delay_secs != current_config.startup_delay_secs
        || previous_config.connect_address != current_config.connect_address
        || previous_config.connect_identifier != current_config.connect_identifier
        || previous_config.adb_path != current_config.adb_path
        || previous_config.adb_server_connect != current_config.adb_server_connect
        || previous_config.exe_path != current_config.exe_path
        || previous_config.exe_args != current_config.exe_args;

    if !is_running {
        if is_enabled {
            let device_name = spawn_device_runtime_internal(app_handle, current.id).await?;
            let _ = request_child_connection_action(
                app_handle,
                current.id,
                crate::infrastructure::ipc::message::ConnectionAction::Probe,
                "检查设备连接...",
                None,
            )
            .await;
            super::state::notify_auto_dispatch_planner();
            return Ok((
                Some(DeviceRuntimeReconcileAction::Spawning),
                format!("设备[{}]({})子进程已自动启动", device_name, current.id),
            ));
        }
        return Ok((
            None,
            "设备当前未运行，本次配置变更不自动拉起子进程，等待调度或临时运行".to_string(),
        ));
    }

    if previous_config.cores != current_config.cores {
        let message = restart_device_runtime_internal(app_handle, current.id).await?;
        return Ok((Some(DeviceRuntimeReconcileAction::Restarting), message));
    }

    let mut messages = Vec::new();
    let mut action = None;

    if previous_config != current_config {
        send_device_config_update(app_handle, current.id, &current_config).await?;
        if should_probe_connection {
            let _ = request_child_connection_action(
                app_handle,
                current.id,
                crate::infrastructure::ipc::message::ConnectionAction::Probe,
                "设备配置已更新，正在重新检查设备连接",
                None,
            )
            .await;
        }
        action = Some(DeviceRuntimeReconcileAction::Syncing);
        messages.push("设备配置已热更新到子进程".to_string());
    }

    if previous_config.execution_policy != current_config.execution_policy {
        messages.push(sync_device_runtime_session_internal(app_handle, current.id).await?);
        action = Some(DeviceRuntimeReconcileAction::Syncing);
    }

    if !previous_config.auto_start && current_config.auto_start {
        let created = reevaluate_device_auto_dispatch(app_handle, current.id).await?;
        let device_label = device_log_label(app_handle, current.id);
        messages.push(format!(
            "设备[{}]已重新评估自动调度，新增/唤醒 {} 项",
            device_label, created
        ));
        action = Some(DeviceRuntimeReconcileAction::Syncing);
    }

    if messages.is_empty() {
        Ok((None, "设备配置未触发运行时协调".to_string()))
    } else {
        Ok((action, messages.join("；")))
    }
}

async fn run_runtime_reconcile_job(
    app_handle: &AppHandle,
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
                    let device_label = device_log_label(app_handle, device_id);
                    messages.push(format!("设备[{}]当前未在线，跳过运行会话同步", device_label));
                }
            }

            if reevaluate_dispatch {
                action = Some(DeviceRuntimeReconcileAction::Syncing);
                let created = reevaluate_device_auto_dispatch(app_handle, device_id).await?;
                let device_label = device_log_label(app_handle, device_id);
                messages.push(format!(
                    "设备[{}]已重新评估自动调度，新增/唤醒 {} 项",
                    device_label, created
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
                        if !enabled {
                            Some(DeviceRuntimeReconcileAction::ShuttingDown)
                        } else if let Some(previous) = previous {
                            if !previous.data.0.enable && current.data.0.enable {
                                Some(DeviceRuntimeReconcileAction::Spawning)
                            } else if previous.data.0.cores != current.data.0.cores {
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
                        }
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
                        let device_label = device_log_label(&app_handle, device_id);
                        Log::error(&format!(
                            "[ process ] 设备[{}] runtime 协调任务失败 type={} error={}",
                            device_label,
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
