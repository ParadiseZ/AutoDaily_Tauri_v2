use super::bundle_loader::{
    load_runtime_queue, load_script_bundles, validate_recovery_task_config,
    validate_run_target_support, LoadedScriptBundle,
};
use crate::api::backend_dto::BackendApiRes;
use crate::api::infrastructure::profile_cache::{
    load_cached_user_profile, should_use_cached_profile,
};
use crate::app::config::vision_cache_conf::get_vision_text_cache_runtime_config_app;
use crate::domain::devices::device_conf::{
    DevicePlatform, DeviceTable, TimeoutAction as DeviceTimeoutAction,
    TimeoutNotifyChannel as DeviceTimeoutNotifyChannel,
};
use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::core::{DeviceId, ScheduleId, SessionId};
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::devices::device_launcher::launch_device;
use crate::infrastructure::http_client::HttpClient;
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem, RuntimeSessionSnapshot,
    RuntimeVisionTextCachePolicy, TimeoutAction as RuntimeTimeoutAction,
    TimeoutNotifyChannel as RuntimeTimeoutNotifyChannel,
};
use crate::{
    constant::table_name::DEVICE_TABLE,
    domain::scripts::script_info::ScriptType,
};
use tauri::Manager;

pub(super) async fn load_device_table(device_id: DeviceId) -> Result<DeviceTable, String> {
    DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备[{}]不存在", device_id))
}

pub(super) fn validate_runtime_platform_supported(device_table: &DeviceTable) -> Result<(), String> {
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

fn has_active_sponsor(sponsor_until: Option<&str>) -> bool {
    sponsor_until
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&chrono::Utc) > chrono::Utc::now())
        .unwrap_or(false)
}

async fn validate_published_script_runtime_access(
    app_handle: &tauri::AppHandle,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    if !bundles
        .iter()
        .any(|bundle| matches!(bundle.script_type, ScriptType::Published))
    {
        return Ok(());
    }

    let client = HttpClient::new(app_handle.clone());
    let session = client
        .get_auth_session()
        .ok_or_else(|| "请先登录后再运行云端下载脚本".to_string())?;

    if session.access_token.trim().is_empty() {
        return Err("请先登录后再运行云端下载脚本".to_string());
    }

    let payload = match client.get::<BackendApiRes<serde_json::Value>>("/user/profile").await {
        Ok(profile) if profile.code == 200 => profile
            .data
            .ok_or_else(|| "用户资料为空，无法校验云端脚本运行权限".to_string())?,
        Ok(profile) => {
            if should_use_cached_profile(profile.code, &profile.message) {
                load_cached_user_profile(app_handle, &session.username).ok_or_else(|| {
                    let message = profile.message.trim();
                    if message.is_empty() {
                        "校验云端脚本运行权限失败".to_string()
                    } else {
                        message.to_string()
                    }
                })?
            } else {
                let message = profile.message.trim();
                return Err(if message.is_empty() {
                    "校验云端脚本运行权限失败".to_string()
                } else {
                    message.to_string()
                });
            }
        }
        Err(error) => load_cached_user_profile(app_handle, &session.username)
            .ok_or_else(|| format!("校验云端脚本运行权限失败: {}", error))?,
    };
    let auth_stage = payload
        .get("authStage")
        .and_then(|value| value.as_i64())
        .unwrap_or(1);

    if auth_stage <= 1 {
        return Ok(());
    }

    let is_developer = payload
        .get("isDeveloper")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let is_sponsor = has_active_sponsor(payload.get("sponsorUntil").and_then(|value| value.as_str()));

    if auth_stage == 2 && (is_developer || is_sponsor) {
        return Ok(());
    }

    if auth_stage >= 3 && is_sponsor {
        return Ok(());
    }

    Err(if auth_stage == 2 {
        "当前阶段仅赞助用户或开发者可运行云端下载脚本".to_string()
    } else {
        "当前阶段仅赞助用户可运行云端下载脚本".to_string()
    })
}

pub(super) async fn load_runtime_session_for_target(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<RuntimeSessionSnapshot, String> {
    build_runtime_session_snapshot(app_handle, device_id, run_target).await
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
                    assignment_id: ScheduleId::new_v7(),
                    script_id,
                    time_template_id: None,
                    account_id: None,
                    account_data_json: None,
                    order_index: 0,
                    template_values_json: None,
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
    if force_prepare {
        launch_device(device_config).await.map_err(|error| {
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

    if device_config.auto_start && device_config.exe_path.is_some() {
        launch_device(device_config)
            .await
            .map_err(|e| format!("自动启动设备失败: {}", e))?;
    }

    Ok(())
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
