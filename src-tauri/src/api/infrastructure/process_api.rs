// 子进程管理 API — 供前端调用
use crate::constant::table_name::{
    ASSIGNMENT_TABLE, DEVICE_TABLE, GROUP_POLICIES, POLICY_GROUP_TABLE, POLICY_SET_TABLE,
    POLICY_TABLE, RECOVERY_CHECKPOINT_TABLE, SCRIPT_TABLE, SCRIPT_TASK_TABLE,
    SCRIPT_TIME_TEMPLATE_VALUES_TABLE, SET_GROUPS,
};
use crate::constant::sys_conf_path::{APP_STORE, VISION_TEXT_CACHE_CONFIG_KEY};
use crate::domain::config::vision_cache_conf::VisionTextCacheConfig;
use crate::domain::devices::device_conf::{
    DevicePlatform, DeviceTable, TimeoutAction as DeviceTimeoutAction,
    TimeoutNotifyChannel as DeviceTimeoutNotifyChannel,
};
use crate::domain::devices::device_schedule::DeviceScriptAssignment;
use crate::domain::schedule::script_time_template_values::ScriptTimeTemplateValuesDto;
use crate::domain::schedule::recovery_checkpoint::RecoveryCheckpointRow;
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::{
    AccountId, DeviceId, ScheduleId, ScriptId, SessionId, TaskId, TemplateId,
};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::devices::device_launcher::launch_device;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    ResumeCheckpoint, RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem,
    RuntimeRecoveryPhase, RuntimeSessionSnapshot, RuntimeVisionTextCachePolicy,
    ScriptBundleSnapshot, SessionCheckpointReason, SessionControlMessage,
    TimeoutAction as RuntimeTimeoutAction, TimeoutNotifyChannel as RuntimeTimeoutNotifyChannel,
};
use serde::Serialize;
use std::collections::HashSet;
use tauri::command;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

struct LoadedScriptBundle {
    script_id: ScriptId,
    script_name: String,
    recovery_task_id: Option<TaskId>,
    runnable_task_ids: HashSet<TaskId>,
    policy_group_ids: HashSet<crate::infrastructure::core::PolicyGroupId>,
    policy_set_ids: HashSet<crate::infrastructure::core::PolicySetId>,
    snapshot: ScriptBundleSnapshot,
}

async fn load_device_table(device_id: DeviceId) -> Result<DeviceTable, String> {
    DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备[{}]不存在", device_id))
}

fn validate_runtime_platform_supported(device_table: &DeviceTable) -> Result<(), String> {
    match device_table.data.0.platform {
        DevicePlatform::Android => Ok(()),
        DevicePlatform::Desktop => Err(format!(
            "设备[{}]当前为 desktop 平台，但本版本尚未实现 Desktop 运行时适配器",
            device_table.data.0.device_name
        )),
    }
}

fn load_vision_text_cache_runtime_config(
    app_handle: &tauri::AppHandle,
) -> Result<crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig, String> {
    let store = app_handle
        .store(APP_STORE)
        .map_err(|e| format!("读取 OCR 缓存配置失败: {}", e))?;

    let persisted = store
        .get(VISION_TEXT_CACHE_CONFIG_KEY)
        .and_then(|value| serde_json::from_value::<VisionTextCacheConfig>(value.clone()).ok())
        .unwrap_or_default();

    let fallback_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取 OCR 缓存默认目录失败: {}", e))?
        .join("ocr-text-cache");

    Ok(persisted.to_runtime_config(fallback_dir))
}

fn to_runtime_policy(
    device_table: &DeviceTable,
    cache_config: crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig,
) -> RuntimeExecutionPolicy {
    let execution_policy = &device_table.data.0.execution_policy;
    RuntimeExecutionPolicy {
        ocr_text_cache: RuntimeVisionTextCachePolicy {
            enabled: cache_config.enabled,
            dir: cache_config
                .dir
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            signature_grid_size: cache_config.signature_grid_size,
        },
        action_wait_ms: execution_policy.action_wait_ms,
        progress_timeout_enabled: execution_policy.progress_timeout_enabled,
        progress_timeout_ms: execution_policy.progress_timeout_ms,
        timeout_action: map_timeout_action(&execution_policy.timeout_action),
        timeout_notify_channels: execution_policy
            .timeout_notify_channels
            .iter()
            .map(map_timeout_notify_channel)
            .collect(),
    }
}

fn map_timeout_action(action: &DeviceTimeoutAction) -> RuntimeTimeoutAction {
    match action {
        DeviceTimeoutAction::NotifyOnly => RuntimeTimeoutAction::NotifyOnly,
        DeviceTimeoutAction::PauseExecution => RuntimeTimeoutAction::PauseExecution,
        DeviceTimeoutAction::StopExecution => RuntimeTimeoutAction::StopExecution,
        DeviceTimeoutAction::RestartApp => RuntimeTimeoutAction::RestartApp,
        DeviceTimeoutAction::RunRecoveryTask => RuntimeTimeoutAction::RunRecoveryTask,
        DeviceTimeoutAction::SkipCurrentTask => RuntimeTimeoutAction::SkipCurrentTask,
    }
}

fn map_timeout_notify_channel(channel: &DeviceTimeoutNotifyChannel) -> RuntimeTimeoutNotifyChannel {
    match channel {
        DeviceTimeoutNotifyChannel::SystemNotification => {
            RuntimeTimeoutNotifyChannel::SystemNotification
        }
        DeviceTimeoutNotifyChannel::Email => RuntimeTimeoutNotifyChannel::Email,
    }
}

fn normalize_account_id(account_id: Option<AccountId>) -> Option<AccountId> {
    account_id.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn serialize_to_json_string<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

async fn find_template_values_with_fallback(
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<Option<ScriptTimeTemplateValuesDto>, String> {
    let pool = get_pool();
    let account_id = normalize_account_id(account_id);
    let query = format!(
        "SELECT id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at
         FROM {}
         WHERE script_id = ?1
           AND time_template_id = ?2
           AND (
                (device_id = ?3 AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4))
             OR (device_id = ?3 AND account_id IS NULL)
             OR (device_id IS NULL AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4))
             OR (device_id IS NULL AND account_id IS NULL)
           )
         ORDER BY
            CASE
                WHEN device_id = ?3 AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4) THEN 0
                WHEN device_id = ?3 AND account_id IS NULL THEN 1
                WHEN device_id IS NULL AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4) THEN 2
                ELSE 3
            END
         LIMIT 1",
        SCRIPT_TIME_TEMPLATE_VALUES_TABLE
    );

    sqlx::query_as::<_, ScriptTimeTemplateValuesDto>(&query)
        .bind(script_id.to_string())
        .bind(time_template_id.to_string())
        .bind(device_id.to_string())
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
}

async fn load_script_bundle(script_id: ScriptId) -> Result<LoadedScriptBundle, String> {
    let pool = get_pool();
    let script = DbRepo::get_by_id::<ScriptTable>(SCRIPT_TABLE, &script_id.to_string())
        .await?
        .ok_or_else(|| format!("脚本[{}]不存在", script_id))?;

    let tasks_query = format!(
        "SELECT * FROM {} WHERE script_id = ? ORDER BY `index` ASC, created_at ASC",
        SCRIPT_TASK_TABLE
    );
    let tasks: Vec<ScriptTaskTable> = sqlx::query_as(&tasks_query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let policies_query = format!(
        "SELECT id, script_id, order_index, `data` FROM {} WHERE script_id = ? ORDER BY order_index ASC",
        POLICY_TABLE
    );
    let policies: Vec<PolicyTable> = sqlx::query_as(&policies_query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let groups_query = format!(
        "SELECT id, script_id, order_index, `data` FROM {} WHERE script_id = ? ORDER BY order_index ASC",
        POLICY_GROUP_TABLE
    );
    let policy_groups: Vec<PolicyGroupTable> = sqlx::query_as(&groups_query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let sets_query = format!(
        "SELECT id, script_id, order_index, `data` FROM {} WHERE script_id = ? ORDER BY order_index ASC",
        POLICY_SET_TABLE
    );
    let policy_sets: Vec<PolicySetTable> = sqlx::query_as(&sets_query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let group_policies_query = format!(
        "SELECT gp.group_id, gp.policy_id, gp.order_index
         FROM {} gp
         JOIN {} g ON gp.group_id = g.id
         WHERE g.script_id = ?
         ORDER BY g.order_index ASC, gp.order_index ASC",
        GROUP_POLICIES, POLICY_GROUP_TABLE
    );
    let group_policies: Vec<GroupPolicyRelation> = sqlx::query_as(&group_policies_query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let set_groups_query = format!(
        "SELECT sg.set_id, sg.group_id, sg.order_index
         FROM {} sg
         JOIN {} s ON sg.set_id = s.id
         WHERE s.script_id = ?
         ORDER BY s.order_index ASC, sg.order_index ASC",
        SET_GROUPS, POLICY_SET_TABLE
    );
    let set_groups: Vec<SetGroupRelation> = sqlx::query_as(&set_groups_query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let runnable_task_ids = tasks
        .iter()
        .filter(|task| task.row_type == TaskRowType::Task && !task.is_deleted)
        .map(|task| task.id)
        .collect();

    let script_name = script.data.0.name.clone();
    let recovery_task_id = script.data.0.runtime_settings.recovery_task_id;
    let policy_group_ids = policy_groups.iter().map(|group| group.id).collect();
    let policy_set_ids = policy_sets.iter().map(|set| set.id).collect();

    Ok(LoadedScriptBundle {
        script_id,
        script_name,
        recovery_task_id,
        runnable_task_ids,
        policy_group_ids,
        policy_set_ids,
        snapshot: ScriptBundleSnapshot {
            script_id,
            script_json: serialize_to_json_string(&script)?,
            tasks_json: serialize_to_json_string(&tasks)?,
            policies_json: serialize_to_json_string(&policies)?,
            policy_groups_json: serialize_to_json_string(&policy_groups)?,
            policy_sets_json: serialize_to_json_string(&policy_sets)?,
            group_policies_json: serialize_to_json_string(&group_policies)?,
            set_groups_json: serialize_to_json_string(&set_groups)?,
        },
    })
}

async fn load_runtime_queue(device_id: DeviceId) -> Result<Vec<RuntimeQueueItem>, String> {
    let query = format!(
        "SELECT id, device_id, script_id, time_template_id, account_data, `index` FROM {} WHERE device_id = ? ORDER BY `index` ASC",
        ASSIGNMENT_TABLE
    );
    let assignments = sqlx::query_as::<_, DeviceScriptAssignment>(&query)
        .bind(device_id.to_string())
        .fetch_all(get_pool())
        .await
        .map_err(|e| e.to_string())?;

    let mut queue = Vec::with_capacity(assignments.len());
    for assignment in assignments {
        let account_data_json =
            serde_json::to_string(&assignment.account_data.0).map_err(|e| e.to_string())?;
        let account_id = None;
        let template_values_json = match assignment.time_template_id {
            Some(time_template_id) => find_template_values_with_fallback(
                device_id,
                assignment.script_id,
                time_template_id,
                account_id.clone(),
            )
            .await?
            .map(|record| serde_json::to_string(&record.values_json.0).map_err(|e| e.to_string()))
            .transpose()?,
            None => None,
        };
        queue.push(RuntimeQueueItem {
            assignment_id: assignment.id,
            script_id: assignment.script_id,
            time_template_id: assignment.time_template_id,
            account_id,
            account_data_json: Some(account_data_json),
            order_index: assignment.index,
            template_values_json,
        });
    }

    Ok(queue)
}

async fn load_script_bundles(
    run_target: &RunTarget,
    queue: &[RuntimeQueueItem],
) -> Result<Vec<LoadedScriptBundle>, String> {
    let mut script_ids = HashSet::new();

    if let Some(script_id) = run_target.script_id() {
        script_ids.insert(script_id);
    }
    for item in queue {
        script_ids.insert(item.script_id);
    }

    let mut bundles = Vec::with_capacity(script_ids.len());
    for script_id in script_ids {
        bundles.push(load_script_bundle(script_id).await?);
    }
    bundles.sort_by_key(|bundle| bundle.snapshot.script_id.to_string());
    Ok(bundles)
}

fn validate_recovery_task_config(
    run_target: &RunTarget,
    runtime_policy: &RuntimeExecutionPolicy,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    if !matches!(runtime_policy.timeout_action, RuntimeTimeoutAction::RunRecoveryTask) {
        return Ok(());
    }

    let required_script_ids: HashSet<ScriptId> = match run_target {
        RunTarget::DeviceQueue => bundles.iter().map(|bundle| bundle.script_id).collect(),
        _ => run_target.script_id().into_iter().collect(),
    };

    for bundle in bundles
        .iter()
        .filter(|bundle| required_script_ids.contains(&bundle.script_id))
    {
        let recovery_task_id = bundle.recovery_task_id.ok_or_else(|| {
            format!(
                "脚本[{}]未配置恢复任务，无法使用 RunRecoveryTask 策略",
                bundle.script_name
            )
        })?;

        if !bundle.runnable_task_ids.contains(&recovery_task_id) {
            return Err(format!(
                "脚本[{}]的恢复任务不存在，或不是可执行 Task",
                bundle.script_name
            ));
        }
    }

    Ok(())
}

fn validate_run_target_support(
    run_target: &RunTarget,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    let find_bundle = |script_id: ScriptId| bundles.iter().find(|bundle| bundle.script_id == script_id);

    match run_target {
        RunTarget::DeviceQueue | RunTarget::FullScript { .. } => Ok(()),
        RunTarget::Task { script_id, task_id } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if bundle.runnable_task_ids.contains(task_id) {
                Ok(())
            } else {
                Err(format!(
                    "脚本[{}]中的任务[{}]不存在，或不是可执行 Task",
                    bundle.script_name, task_id
                ))
            }
        }
        RunTarget::PolicyGroup {
            script_id,
            policy_group_id,
        } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if !bundle.policy_group_ids.contains(policy_group_id) {
                return Err(format!(
                    "脚本[{}]中的策略组[{}]不存在",
                    bundle.script_name, policy_group_id
                ));
            }

            Err(format!(
                "策略组[{}]运行目标的执行计划尚未接入，当前版本仅支持任务与整脚本运行",
                policy_group_id
            ))
        }
        RunTarget::PolicySet {
            script_id,
            policy_set_id,
        } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if !bundle.policy_set_ids.contains(policy_set_id) {
                return Err(format!(
                    "脚本[{}]中的策略集[{}]不存在",
                    bundle.script_name, policy_set_id
                ));
            }

            Err(format!(
                "策略集[{}]运行目标的执行计划尚未接入，当前版本仅支持任务与整脚本运行",
                policy_set_id
            ))
        }
    }
}

fn checkpoint_matches_run_target(run_target: &RunTarget, checkpoint: &ResumeCheckpoint) -> bool {
    match run_target {
        RunTarget::DeviceQueue => matches!(checkpoint.run_target, RunTarget::DeviceQueue),
        _ => run_target.script_id() == Some(checkpoint.script_id),
    }
}

async fn load_recovery_checkpoint(
    device_id: DeviceId,
    run_target: &RunTarget,
) -> Result<Option<ResumeCheckpoint>, String> {
    let checkpoint = load_latest_recovery_checkpoint(device_id).await?;
    Ok(checkpoint.filter(|item| checkpoint_matches_run_target(run_target, item)))
}

async fn load_latest_recovery_checkpoint(
    device_id: DeviceId,
) -> Result<Option<ResumeCheckpoint>, String> {
    let query = format!(
        "SELECT execution_id, source_session_id, device_id, run_target_json, assignment_id, script_id, time_template_id, account_id, task_id, step_id, resume_mode, definition_fingerprint, updated_at
         FROM {}
         WHERE device_id = ?
         ORDER BY updated_at DESC
         LIMIT 1",
        RECOVERY_CHECKPOINT_TABLE
    );
    let checkpoint = sqlx::query_as::<_, RecoveryCheckpointRow>(&query)
        .bind(device_id.to_string())
        .fetch_optional(get_pool())
        .await
        .map_err(|e| e.to_string())?
        .map(RecoveryCheckpointRow::into_checkpoint);
    Ok(checkpoint)
}

async fn load_runtime_session_for_target(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<(), String> {
    let (session, checkpoint) =
        build_runtime_session_snapshot(app_handle, device_id, run_target).await?;
    send_session_control(
        device_id,
        SessionControlMessage::LoadSession {
            session,
            checkpoint,
        },
    )
    .await;
    Ok(())
}

async fn restart_device_runtime_internal(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    reason: SessionCheckpointReason,
) -> Result<String, String> {
    let manager = get_process_manager()
        .ok_or_else(|| "进程管理器未初始化".to_string())?;
    let was_running = manager.is_running(&device_id).await;

    let previous_checkpoint = latest_checkpoint_updated_at(device_id).await?;
    let previous_sequence = current_recovery_sequence(app_handle, device_id);
    if was_running {
        send_session_control(
            device_id,
            SessionControlMessage::PrepareCheckpoint {
                reason: reason.clone(),
            },
        )
        .await;
        let restart_ready = wait_for_restart_ready_event(
            app_handle,
            device_id,
            previous_sequence,
            std::time::Duration::from_secs(3),
        )
        .await?;
        if !restart_ready {
            let _ = wait_for_checkpoint_refresh_fallback(
                device_id,
                previous_checkpoint,
                std::time::Duration::from_secs(2),
            )
            .await?;
        }
        manager.stop_child(&device_id).await?;
    }

    let init_data = build_child_init_data(app_handle, device_id).await?;
    manager.spawn_child(init_data).await?;
    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await?;

    let run_target = load_latest_recovery_checkpoint(device_id)
        .await?
        .map(|checkpoint| checkpoint.run_target)
        .unwrap_or(RunTarget::DeviceQueue);
    load_runtime_session_for_target(app_handle, device_id, run_target).await?;

    Ok(format!(
        "设备[{}]子进程已按 checkpoint 流程重启并重新装填 session",
        device_id
    ))
}

async fn build_runtime_session_snapshot(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<(RuntimeSessionSnapshot, Option<ResumeCheckpoint>), String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let queue = match &run_target {
        RunTarget::DeviceQueue => load_runtime_queue(device_id).await?,
        target => target
            .script_id()
            .map(|script_id| RuntimeQueueItem {
                assignment_id: ScheduleId::new_v7(),
                script_id,
                time_template_id: None,
                account_id: None,
                account_data_json: None,
                order_index: 0,
                template_values_json: None,
            })
            .into_iter()
            .collect(),
    };
    let runtime_policy =
        to_runtime_policy(&device_table, load_vision_text_cache_runtime_config(app_handle)?);
    let loaded_script_bundles = load_script_bundles(&run_target, &queue).await?;
    validate_run_target_support(&run_target, &loaded_script_bundles)?;
    validate_recovery_task_config(&run_target, &runtime_policy, &loaded_script_bundles)?;
    let compatible_script_ids: HashSet<ScriptId> =
        loaded_script_bundles.iter().map(|bundle| bundle.script_id).collect();
    let checkpoint = load_recovery_checkpoint(device_id, &run_target)
        .await?
        .filter(|item| compatible_script_ids.contains(&item.script_id));
    let script_bundles = loaded_script_bundles
        .into_iter()
        .map(|bundle| bundle.snapshot)
        .collect();
    Ok((RuntimeSessionSnapshot {
        session_id: SessionId::new_v7(),
        device_id,
        run_target,
        runtime_policy,
        queue,
        script_bundles,
        issued_at: chrono::Local::now().to_rfc3339(),
    }, checkpoint))
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

async fn latest_checkpoint_updated_at(device_id: DeviceId) -> Result<Option<String>, String> {
    Ok(load_latest_recovery_checkpoint(device_id)
        .await?
        .map(|checkpoint| checkpoint.updated_at))
}

async fn wait_for_restart_ready_event(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    previous_sequence: u64,
    timeout: std::time::Duration,
) -> Result<bool, String> {
    let recovery_runtime = app_handle
        .state::<MainProcessCtx>()
        .recovery_runtime
        .clone();
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        if let Some(signal) = recovery_runtime.latest_signal(device_id) {
            if signal.sequence > previous_sequence
                && signal.phase == RuntimeRecoveryPhase::RestartReady
            {
                return Ok(true);
            }
        }

        let now = tokio::time::Instant::now();
        if now >= deadline {
            return Ok(false);
        }

        if tokio::time::timeout_at(deadline, recovery_runtime.notify.notified())
            .await
            .is_err()
        {
            return Ok(false);
        }
    }
}

fn current_recovery_sequence(app_handle: &tauri::AppHandle, device_id: DeviceId) -> u64 {
    app_handle
        .state::<MainProcessCtx>()
        .recovery_runtime
        .current_sequence(device_id)
}

async fn wait_for_checkpoint_refresh_fallback(
    device_id: DeviceId,
    previous_updated_at: Option<String>,
    timeout: std::time::Duration,
) -> Result<bool, String> {
    let started_at = tokio::time::Instant::now();
    loop {
        let latest = latest_checkpoint_updated_at(device_id).await?;
        if latest.is_some() && latest != previous_updated_at {
            return Ok(true);
        }

        if started_at.elapsed() >= timeout {
            return Ok(false);
        }

        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    }
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
            if guard.iter().any(|(registered_device_id, _)| **registered_device_id == device_id) {
                return Ok(());
            }
        }

        if started_at.elapsed() >= timeout {
            return Err(format!("设备[{}]子进程启动后未及时连上 IPC", device_id));
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn build_child_init_data(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<ChildProcessInitData, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;

    let device_config = device_table.data.0;

    if device_config.auto_start && device_config.exe_path.is_some() {
        launch_device(&device_config)
            .await
            .map_err(|e| format!("自动启动设备失败: {}", e))?;
    }

    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据库路径失败: {}", e))?;

    Ok(ChildProcessInitData {
        device_id,
        device_config: device_config.clone(),
        shm_name: format!("autodaily_shm_{}", device_id),
        log_level: device_config.log_level.clone(),
        cpu_cores: device_config.cores.iter().map(|c| *c as usize).collect(),
        db_path,
        vision_text_cache_config: load_vision_text_cache_runtime_config(app_handle)?,
    })
}

async fn ensure_device_online(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let manager = get_process_manager()
        .ok_or_else(|| "进程管理器未初始化".to_string())?;

    if !manager.is_running(&device_id).await {
        let init_data = build_child_init_data(app_handle, device_id).await?;
        manager.spawn_child(init_data).await?;
    }

    wait_for_ipc_client(app_handle, device_id, std::time::Duration::from_secs(5)).await
}

/// 向已运行的子进程发送 Start 命令（开始执行脚本队列）
#[command]
pub async fn cmd_device_start(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    ensure_device_online(&app_handle, device_id).await?;
    load_runtime_session_for_target(&app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_process_control(device_id, ProcessAction::Start);
    Ok(format!("已向设备[{}]发送启动命令", device_id))
}

/// 向子进程发送 Stop 命令（停止当前脚本但不退出）
#[command]
pub async fn cmd_device_stop(device_id: DeviceId) -> Result<String, String> {
    send_process_control(device_id, ProcessAction::Stop);
    Ok(format!("已向设备[{}]发送停止命令", device_id))
}

/// 向子进程发送 Pause 命令
#[command]
pub async fn cmd_device_pause(device_id: DeviceId) -> Result<String, String> {
    send_process_control(device_id, ProcessAction::Pause);
    Ok(format!("已向设备[{}]发送暂停命令", device_id))
}

/// 同步设备的完整运行会话
#[command]
pub async fn cmd_sync_device_runtime_session(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    ensure_device_online(&app_handle, device_id).await?;
    let (session, checkpoint) =
        build_runtime_session_snapshot(&app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_session_control(
        device_id,
        SessionControlMessage::ReloadSession {
            session,
            checkpoint,
        },
    )
    .await;
    Ok(format!("已同步设备[{}]运行会话", device_id))
}

/// 编辑器调试运行指定目标
#[command]
pub async fn cmd_run_script_target(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    target: RunTarget,
) -> Result<String, String> {
    ensure_device_online(&app_handle, device_id).await?;
    load_runtime_session_for_target(&app_handle, device_id, target.clone()).await?;
    send_process_control(device_id, ProcessAction::Start);
    Ok(format!("已向设备[{}]发送运行目标: {:?}", device_id, target))
}

/// 请求子进程在安全点准备恢复检查点
#[command]
pub async fn cmd_prepare_device_checkpoint(
    device_id: DeviceId,
    reason: SessionCheckpointReason,
) -> Result<String, String> {
    send_session_control(
        device_id,
        SessionControlMessage::PrepareCheckpoint { reason: reason.clone() },
    )
    .await;
    Ok(format!(
        "已向设备[{}]发送 checkpoint 准备命令: {:?}",
        device_id, reason
    ))
}

/// 请求主进程按 checkpoint 流程重启 child，并重新装填 session
#[command]
pub async fn cmd_restart_device_runtime(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    reason: SessionCheckpointReason,
) -> Result<String, String> {
    restart_device_runtime_internal(&app_handle, device_id, reason).await
}

/// 关闭子进程
#[command]
pub async fn cmd_device_shutdown(device_id: DeviceId) -> Result<String, String> {
    if let Some(manager) = get_process_manager() {
        manager.stop_child(&device_id).await?;
        Ok(format!("设备[{}]子进程已关闭", device_id))
    } else {
        Err("进程管理器未初始化".to_string())
    }
}

/// 获取所有运行中的设备
#[command]
pub async fn cmd_get_running_devices() -> Result<Vec<String>, String> {
    if let Some(manager) = get_process_manager() {
        let ids = manager.get_running_device_ids().await;
        Ok(ids.iter().map(|id| id.to_string()).collect())
    } else {
        Err("进程管理器未初始化".to_string())
    }
}

/// 启动设备的子进程
#[command]
pub async fn cmd_spawn_device(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let init_data = build_child_init_data(&app_handle, device_id).await?;
    let device_name = init_data.device_config.device_name.clone();
    let manager = get_process_manager()
        .ok_or_else(|| "进程管理器未初始化".to_string())?;
    manager.spawn_child(init_data).await?;
    wait_for_ipc_client(&app_handle, device_id, std::time::Duration::from_secs(5)).await?;

    Ok(format!(
        "设备[{}]({})子进程已启动",
        device_name, device_id
    ))
}

/// 检查设备子进程是否在运行
#[command]
pub async fn cmd_is_device_running(device_id: DeviceId) -> Result<bool, String> {
    if let Some(manager) = get_process_manager() {
        Ok(manager.is_running(&device_id).await)
    } else {
        Err("进程管理器未初始化".to_string())
    }
}
