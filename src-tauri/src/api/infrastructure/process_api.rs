// 子进程管理 API — 供前端调用
use crate::constant::table_name::{
    ASSIGNMENT_TABLE, DEVICE_TABLE, GROUP_POLICIES, POLICY_GROUP_TABLE, POLICY_SET_TABLE,
    POLICY_TABLE, SCRIPT_TABLE, SCRIPT_TASK_TABLE, SCRIPT_TIME_TEMPLATE_VALUES_TABLE, SET_GROUPS,
};
use crate::constant::sys_conf_path::{APP_STORE, VISION_TEXT_CACHE_CONFIG_KEY};
use crate::domain::config::vision_cache_conf::VisionTextCacheConfig;
use crate::domain::devices::device_conf::DeviceTable;
use crate::domain::devices::device_schedule::DeviceScriptAssignment;
use crate::domain::schedule::script_time_template_values::ScriptTimeTemplateValuesDto;
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::{AccountId, DeviceId, ScheduleId, ScriptId, SessionId, TemplateId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::devices::device_launcher::launch_device;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem, RuntimeSessionSnapshot,
    RuntimeVisionTextCachePolicy, ScriptBundleSnapshot, SessionControlMessage, TimeoutAction,
    TimeoutNotifyPolicy,
};
use serde::Serialize;
use std::collections::HashSet;
use tauri::command;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

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
    cache_config: crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig,
) -> RuntimeExecutionPolicy {
    RuntimeExecutionPolicy {
        ocr_text_cache: RuntimeVisionTextCachePolicy {
            enabled: cache_config.enabled,
            dir: cache_config
                .dir
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            signature_grid_size: cache_config.signature_grid_size,
        },
        action_wait_ms: 500,
        step_timeout_ms: 30_000,
        timeout_action: TimeoutAction::Stop,
        timeout_notify: TimeoutNotifyPolicy::None,
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

async fn load_script_bundle(script_id: ScriptId) -> Result<ScriptBundleSnapshot, String> {
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

    Ok(ScriptBundleSnapshot {
        script_id,
        script_json: serialize_to_json_string(&script)?,
        tasks_json: serialize_to_json_string(&tasks)?,
        policies_json: serialize_to_json_string(&policies)?,
        policy_groups_json: serialize_to_json_string(&policy_groups)?,
        policy_sets_json: serialize_to_json_string(&policy_sets)?,
        group_policies_json: serialize_to_json_string(&group_policies)?,
        set_groups_json: serialize_to_json_string(&set_groups)?,
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

async fn load_script_bundles(run_target: &RunTarget, queue: &[RuntimeQueueItem]) -> Result<Vec<ScriptBundleSnapshot>, String> {
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
    bundles.sort_by_key(|bundle| bundle.script_id.to_string());
    Ok(bundles)
}

async fn build_runtime_session_snapshot(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<RuntimeSessionSnapshot, String> {
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
    let script_bundles = load_script_bundles(&run_target, &queue).await?;

    Ok(RuntimeSessionSnapshot {
        session_id: SessionId::new_v7(),
        device_id,
        run_target,
        runtime_policy: to_runtime_policy(load_vision_text_cache_runtime_config(app_handle)?),
        queue,
        script_bundles,
        issued_at: chrono::Local::now().to_rfc3339(),
    })
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

/// 向已运行的子进程发送 Start 命令（开始执行脚本队列）
#[command]
pub async fn cmd_device_start(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    let session = build_runtime_session_snapshot(&app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_session_control(
        device_id,
        SessionControlMessage::LoadSession {
            session,
            checkpoint: None,
        },
    )
    .await;
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
    let session = build_runtime_session_snapshot(&app_handle, device_id, RunTarget::DeviceQueue).await?;
    send_session_control(
        device_id,
        SessionControlMessage::ReloadSession {
            session,
            checkpoint: None,
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
    let session = build_runtime_session_snapshot(&app_handle, device_id, target.clone()).await?;
    send_session_control(
        device_id,
        SessionControlMessage::LoadSession {
            session,
            checkpoint: None,
        },
    )
    .await;
    send_process_control(device_id, ProcessAction::Start);
    Ok(format!("已向设备[{}]发送运行目标: {:?}", device_id, target))
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
    // 1. 从数据库加载设备配置
    let device_table: DeviceTable = DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备[{}]不存在", device_id))?;

    let device_config = device_table.data.0;

    // 2. auto_start 时先启动模拟器并等待连接
    if device_config.auto_start {
        if device_config.exe_path.is_some() {
            launch_device(&device_config).await
                .map_err(|e| format!("自动启动设备失败: {}", e))?;
        }
    }

    // 3. 获取数据库路径
    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据库路径失败: {}", e))?;

    // 4. 构造初始化数据
    let init_data = ChildProcessInitData {
        device_id,
        device_config: device_config.clone(),
        shm_name: format!("autodaily_shm_{}", device_id),
        log_level: device_config.log_level.clone(),
        cpu_cores: device_config.cores.iter().map(|c| *c as usize).collect(),
        db_path,
        vision_text_cache_config: load_vision_text_cache_runtime_config(&app_handle)?,
    };

    // 5. 获取进程管理器并启动子进程
    let manager = get_process_manager()
        .ok_or_else(|| "进程管理器未初始化".to_string())?;

    manager.spawn_child(init_data).await?;

    Ok(format!(
        "设备[{}]({})子进程已启动",
        device_config.device_name, device_id
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
