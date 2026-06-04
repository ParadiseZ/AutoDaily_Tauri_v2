use super::access_control::validate_published_script_runtime_access;
use crate::api::infrastructure::process_api::bundle_loader::{
    load_runtime_queue, load_script_bundles, validate_recovery_task_config,
    validate_run_target_support,
};
use crate::app::config::vision_cache_conf::get_vision_text_cache_runtime_config_app;
use crate::constant::table_name::DEVICE_TABLE;
use crate::domain::devices::device_conf::{
    DevicePlatform, DeviceTable, TimeoutAction as DeviceTimeoutAction,
    TimeoutNotifyChannel as DeviceTimeoutNotifyChannel,
};
use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::core::{AssignmentId, DeviceId, SessionId};
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::devices::device_launcher::start_device_process;
use crate::infrastructure::ipc::message::{
    DispatchKind, DispatchSource, RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem,
    RuntimeSessionSnapshot, RuntimeVisionTextCachePolicy, TimeoutAction as RuntimeTimeoutAction,
    TimeoutNotifyChannel as RuntimeTimeoutNotifyChannel,
};
use tauri::Manager;

pub(super) async fn load_device_table(device_id: DeviceId) -> Result<DeviceTable, String> {
    DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备[{}]不存在", device_id))
}

pub(super) fn validate_runtime_platform_supported(
    device_table: &DeviceTable,
) -> Result<(), String> {
    match device_table.data.0.platform {
        DevicePlatform::Android => Ok(()),
        DevicePlatform::Desktop => Err(format!(
            "设备[{}]当前为 desktop 平台，但本版本尚未实现 Desktop 运行时适配器",
            device_table.data.0.device_name
        )),
    }
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
        DeviceTimeoutAction::StopExecution => RuntimeTimeoutAction::StopExecution,
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

pub(super) async fn load_runtime_session_for_target(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<RuntimeSessionSnapshot, String> {
    build_runtime_session_snapshot(app_handle, device_id, run_target).await
}

pub(super) async fn load_runtime_session_for_queue_item(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    queue_item: RuntimeQueueItem,
) -> Result<RuntimeSessionSnapshot, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let runtime_policy = to_runtime_policy(
        &device_table,
        get_vision_text_cache_runtime_config_app(app_handle)
            .await
            .map_err(|e| format!("读取 OCR 缓存配置失败: {}", e))?,
    );
    let queue = vec![queue_item];
    let run_target = RunTarget::DeviceQueue;
    let loaded_script_bundles = load_script_bundles(&run_target, &queue).await?;
    validate_published_script_runtime_access(app_handle, &loaded_script_bundles).await?;
    validate_run_target_support(&run_target, &loaded_script_bundles)?;
    validate_recovery_task_config(&run_target, &runtime_policy, &loaded_script_bundles)?;
    let script_bundles = loaded_script_bundles
        .into_iter()
        .map(|bundle| bundle.snapshot)
        .collect();
    Ok(RuntimeSessionSnapshot {
        session_id: SessionId::new_v7(),
        device_id,
        run_target,
        runtime_policy,
        queue,
        script_bundles,
        issued_at: chrono::Local::now().to_rfc3339(),
    })
}

pub(super) async fn build_runtime_session_snapshot(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<RuntimeSessionSnapshot, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;
    let queue = match &run_target {
        RunTarget::DeviceQueue => load_runtime_queue(device_id).await?,
        target => {
            let mut queue = Vec::new();
            if let Some(script_id) = target.script_id() {
                queue.push(RuntimeQueueItem {
                    dispatch_id: crate::infrastructure::core::DispatchId::new_v7(),
                    dispatch_kind: dispatch_kind_for_run_target(target),
                    dispatch_source: dispatch_source_for_run_target(target),
                    assignment_id: AssignmentId::new_v7(),
                    script_id,
                    time_template_id: None,
                    account_id: None,
                    account_data_json: None,
                    order_index: 0,
                    window_start_at: None,
                    template_values_json: None,
                    dedup_scope_base_hash: String::new(),
                });
            }
            queue
        }
    };
    let runtime_policy = to_runtime_policy(
        &device_table,
        get_vision_text_cache_runtime_config_app(app_handle)
            .await
            .map_err(|e| format!("读取 OCR 缓存配置失败: {}", e))?,
    );
    let loaded_script_bundles = load_script_bundles(&run_target, &queue).await?;
    validate_published_script_runtime_access(app_handle, &loaded_script_bundles).await?;
    validate_run_target_support(&run_target, &loaded_script_bundles)?;
    validate_recovery_task_config(&run_target, &runtime_policy, &loaded_script_bundles)?;
    let script_bundles = loaded_script_bundles
        .into_iter()
        .map(|bundle| bundle.snapshot)
        .collect();
    Ok(RuntimeSessionSnapshot {
        session_id: SessionId::new_v7(),
        device_id,
        run_target,
        runtime_policy,
        queue,
        script_bundles,
        issued_at: chrono::Local::now().to_rfc3339(),
    })
}

async fn prepare_device_launch(
    device_config: &crate::domain::devices::device_conf::DeviceConfig,
    force_prepare: bool,
) -> Result<(), String> {
    if force_prepare && device_config.uses_emulator_transport() {
        start_device_process(device_config).await.map_err(|error| {
            if device_config
                .exe_path
                .as_deref()
                .is_none_or(|path| path.trim().is_empty())
            {
                format!(
                    "启动设备失败: {}。当前未填写模拟器程序路径；如需自动启动，请先在设备编辑中补全路径。",
                    error
                )
            } else {
                format!("启动设备失败: {}", error)
            }
        })?;
        return Ok(());
    }

    Ok(())
}

fn dispatch_kind_for_run_target(run_target: &RunTarget) -> DispatchKind {
    match run_target {
        RunTarget::DeviceQueue => DispatchKind::QueueAssignment,
        RunTarget::FullScript { .. } => DispatchKind::TemporaryFullScript,
        RunTarget::Task { .. } => DispatchKind::TemporaryTask,
        RunTarget::PolicyGroup { .. } => DispatchKind::DebugGroup,
        RunTarget::PolicySet { .. } => DispatchKind::DebugSet,
        RunTarget::Policy { .. } => DispatchKind::DebugPolicy,
    }
}

fn dispatch_source_for_run_target(run_target: &RunTarget) -> DispatchSource {
    match run_target {
        RunTarget::PolicyGroup { .. } | RunTarget::PolicySet { .. } | RunTarget::Policy { .. } => {
            DispatchSource::Debug
        }
        _ => DispatchSource::User,
    }
}

pub(super) async fn build_child_init_data(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    force_prepare_device: bool,
) -> Result<ChildProcessInitData, String> {
    let device_table = load_device_table(device_id).await?;
    validate_runtime_platform_supported(&device_table)?;

    let device_config = device_table.data.0;
    prepare_device_launch(&device_config, force_prepare_device).await?;

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
        vision_text_cache_config: get_vision_text_cache_runtime_config_app(app_handle)
            .await
            .map_err(|e| format!("读取 OCR 缓存配置失败: {}", e))?,
    })
}
