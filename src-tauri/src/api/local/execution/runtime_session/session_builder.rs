use super::access_control::validate_published_script_runtime_access;
use crate::api::local::execution::bundle_loader::{
    load_runtime_queue, load_script_bundles, validate_run_target_support,
};
use crate::app::config::vision_cache_conf::get_vision_text_cache_runtime_config_app;
use ad_kernel::ids::{AssignmentId, DeviceId, SessionId};
use domain_device::{DevicePlatform, DeviceProfile};
use infra_sqlite::get_device;
use runner_protocol::ChildProcessInitData;
use runner_protocol::message::{
    DispatchKind, DispatchSource, RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem,
    RuntimeSessionSnapshot,
};
use tauri::Manager;

pub(super) async fn load_device_profile(device_id: DeviceId) -> Result<DeviceProfile, String> {
    get_device(device_id)
        .await?
        .ok_or_else(|| "目标设备不存在".to_string())
}

pub(super) fn validate_runtime_platform_supported(
    device_profile: &DeviceProfile,
) -> Result<(), String> {
    match device_profile.config.platform {
        DevicePlatform::Android => Ok(()),
        DevicePlatform::Desktop => Err(format!(
            "设备[{}]当前为 desktop 平台，但本版本尚未实现 Desktop 运行时适配器",
            device_profile.config.device_name
        )),
    }
}

fn to_runtime_policy(device_profile: &DeviceProfile) -> RuntimeExecutionPolicy {
    let execution_policy = &device_profile.config.execution_policy;
    RuntimeExecutionPolicy {
        action_wait_ms: u64::from(execution_policy.action_wait_ms),
        progress_timeout_enabled: execution_policy.progress_timeout_enabled,
        progress_timeout_ms: u64::from(execution_policy.progress_timeout_ms),
        timeout_action: execution_policy.timeout_action.clone(),
        timeout_notify_channels: execution_policy.timeout_notify_channels.clone(),
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
    let device_profile = load_device_profile(device_id).await?;
    validate_runtime_platform_supported(&device_profile)?;
    let runtime_policy = to_runtime_policy(&device_profile);
    let queue = vec![queue_item];
    let run_target = RunTarget::DeviceQueue;
    let loaded_script_bundles = load_script_bundles(&run_target, &queue).await?;
    validate_published_script_runtime_access(app_handle, &loaded_script_bundles).await?;
    validate_run_target_support(&run_target, &loaded_script_bundles)?;
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
    let device_profile = load_device_profile(device_id).await?;
    validate_runtime_platform_supported(&device_profile)?;
    let queue = match &run_target {
        RunTarget::DeviceQueue => load_runtime_queue(device_id).await?,
        target => {
            let mut queue = Vec::new();
            if let Some(script_id) = target.script_id() {
                queue.push(RuntimeQueueItem {
                    dispatch_id: ad_kernel::ids::DispatchId::new_v7(),
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
    let runtime_policy = to_runtime_policy(&device_profile);
    let loaded_script_bundles = load_script_bundles(&run_target, &queue).await?;
    validate_published_script_runtime_access(app_handle, &loaded_script_bundles).await?;
    validate_run_target_support(&run_target, &loaded_script_bundles)?;
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
) -> Result<ChildProcessInitData, String> {
    let device_profile = load_device_profile(device_id).await?;
    validate_runtime_platform_supported(&device_profile)?;

    let device_config = device_profile.config;

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
