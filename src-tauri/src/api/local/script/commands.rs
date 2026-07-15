use crate::api::local::execution::{
    enqueue_device_runtime_session_refresh_jobs, load_assigned_device_ids_by_script,
    notify_auto_dispatch_planner,
};
use crate::api::local::script::dto::{ScriptEditorSaveRequest, ScriptTable, ScriptTaskTable};
use crate::api::response::ApiResponse;
use crate::api::server::dto::apply_current_client_capability;
use crate::api::server::local_scripts_dir;
use crate::api::server::profile_cache::load_current_authenticated_user;
use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, ScriptId};
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptProfile, ScriptTaskProfile,
};
use domain_script::{clone_cloud_id, ensure_clone_allowed, ensure_editable};
use infra_sqlite::{
    delete_script, ensure_existing_script_editable, ensure_stored_script_editable,
    save_cloned_script_graph, save_script_editor_graph,
};
use infra_sqlite::{
    find_dev_script_by_cloud_id, get_script, list_group_policy_links, list_policies,
    list_policy_groups, list_policy_sets, list_script_tasks, list_scripts, list_set_group_links,
    save_script,
};
use std::collections::HashSet;
use tauri::command;

/// 获取所有脚本配置
#[command]
pub async fn get_all_scripts_cmd() -> Result<Vec<ScriptTable>, String> {
    list_scripts()
        .await
        .map(|scripts| scripts.into_iter().map(Into::into).collect())
}

/// 根据 ID 获取脚本配置
#[command]
pub async fn get_script_by_id_cmd(script_id: ScriptId) -> Result<Option<ScriptTable>, String> {
    get_script(script_id)
        .await
        .map(|script| script.map(Into::into))
}

/// 保存（新增或更新）脚本配置
#[command]
pub async fn save_script_cmd(
    app_handle: tauri::AppHandle,
    script: ScriptTable,
) -> Result<(), String> {
    let mut script: ScriptProfile = script.into();
    ensure_editable(&script.info.script_type).map_err(|error| error.to_string())?;

    ensure_stored_script_editable(script.id).await?;

    apply_current_client_capability(&mut script.info);

    let affected_device_ids = load_assigned_device_ids_by_script(script.id).await?;
    save_script(&script).await?;
    notify_auto_dispatch_planner();
    enqueue_device_runtime_session_refresh_jobs(
        &app_handle,
        affected_device_ids,
        true,
        false,
        "save_script",
    )?;
    Ok(())
}

fn parse_relation_ids(values: &[String], valid_ids: &HashSet<String>) -> Vec<String> {
    values
        .iter()
        .filter(|value| valid_ids.contains(value.as_str()))
        .cloned()
        .collect()
}

#[command]
pub async fn save_script_editor_cmd(
    app_handle: tauri::AppHandle,
    payload: ScriptEditorSaveRequest,
) -> Result<(), String> {
    let ScriptEditorSaveRequest {
        script,
        tasks,
        policies,
        policy_groups,
        policy_sets,
        group_policy_ids_by_group_id,
        set_group_ids_by_set_id,
    } = payload;
    let mut script: ScriptProfile = script.into();
    let tasks: Vec<ScriptTaskProfile> = tasks.into_iter().map(Into::into).collect();
    let policies: Vec<PolicyProfile> = policies.into_iter().map(Into::into).collect();
    let policy_groups: Vec<PolicyGroupProfile> =
        policy_groups.into_iter().map(Into::into).collect();
    let policy_sets: Vec<PolicySetProfile> = policy_sets.into_iter().map(Into::into).collect();

    ensure_editable(&script.info.script_type).map_err(|error| error.to_string())?;

    ensure_existing_script_editable(script.id).await?;
    apply_current_client_capability(&mut script.info);

    let policy_ids = policies
        .iter()
        .map(|policy| policy.id.to_string())
        .collect::<HashSet<_>>();
    let policy_group_ids = policy_groups
        .iter()
        .map(|group| group.id.to_string())
        .collect::<HashSet<_>>();
    let policy_set_ids = policy_sets
        .iter()
        .map(|set| set.id.to_string())
        .collect::<HashSet<_>>();

    let group_policies = group_policy_ids_by_group_id
        .iter()
        .filter(|(group_id, _)| policy_group_ids.contains(group_id.as_str()))
        .flat_map(|(group_id, policy_ids_for_group)| {
            parse_relation_ids(policy_ids_for_group, &policy_ids)
                .into_iter()
                .enumerate()
                .filter_map(|(order_index, policy_id)| {
                    Some(PolicyGroupPolicyLink {
                        group_id: PolicyGroupId::from(uuid::Uuid::parse_str(group_id).ok()?),
                        policy_id: PolicyId::from(uuid::Uuid::parse_str(&policy_id).ok()?),
                        order_index: order_index as i32,
                    })
                })
        })
        .collect::<Vec<_>>();
    let set_groups = set_group_ids_by_set_id
        .iter()
        .filter(|(set_id, _)| policy_set_ids.contains(set_id.as_str()))
        .flat_map(|(set_id, group_ids_for_set)| {
            parse_relation_ids(group_ids_for_set, &policy_group_ids)
                .into_iter()
                .enumerate()
                .filter_map(|(order_index, group_id)| {
                    Some(PolicySetGroupLink {
                        set_id: PolicySetId::from(uuid::Uuid::parse_str(set_id).ok()?),
                        group_id: PolicyGroupId::from(uuid::Uuid::parse_str(&group_id).ok()?),
                        order_index: order_index as i32,
                    })
                })
        })
        .collect::<Vec<_>>();

    let affected_device_ids = load_assigned_device_ids_by_script(script.id).await?;
    save_script_editor_graph(
        &script,
        &policies,
        &policy_groups,
        &policy_sets,
        &group_policies,
        &set_groups,
        &tasks,
    )
    .await?;
    notify_auto_dispatch_planner();
    enqueue_device_runtime_session_refresh_jobs(
        &app_handle,
        affected_device_ids,
        true,
        false,
        "save_script_editor",
    )?;
    Ok(())
}

/// 删除脚本配置
#[command]
pub async fn delete_script_cmd(
    app_handle: tauri::AppHandle,
    script_id: ScriptId,
) -> Result<(), String> {
    let script_dir = local_scripts_dir(&app_handle).join(script_id.to_string());
    let affected_device_ids = load_assigned_device_ids_by_script(script_id).await?;
    delete_script(script_id).await?;
    if script_dir.exists() {
        std::fs::remove_dir_all(&script_dir)
            .map_err(|error| format!("删除脚本目录 {} 失败: {}", script_dir.display(), error))?;
    }
    notify_auto_dispatch_planner();
    enqueue_device_runtime_session_refresh_jobs(
        &app_handle,
        affected_device_ids,
        true,
        false,
        "delete_script",
    )?;
    Ok(())
}

/// 获取脚本关联的所有任务逻辑
#[command]
pub async fn get_script_tasks_cmd(script_id: ScriptId) -> Result<Vec<ScriptTaskTable>, String> {
    list_script_tasks(script_id)
        .await
        .map(|tasks| tasks.into_iter().map(Into::into).collect())
}

/// 读取 YOLO 标签文件
#[command]
pub async fn get_yolo_labels_cmd(
    path: String,
) -> Result<std::collections::HashMap<u16, String>, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let values: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(names) = values.get("names") {
        let labels: std::collections::HashMap<u16, String> =
            serde_yaml::from_value(names.clone()).map_err(|e| e.to_string())?;
        Ok(labels)
    } else {
        Err("Yolo标签文件格式错误：未找到 names 属性".to_string())
    }
}

/// 克隆本地脚本字典逻辑
/// 1. 权限控制: 如果 allow_clone 为 false 且不属于当前登录用户，拒绝
/// 2. 克隆 published -> Dev: 根据入参 `overwrite_cloud_id` 决定是覆盖还是新建
/// 3. 克隆 Dev -> Dev: 始终作为独立新副本
#[command]
pub async fn clone_local_script_cmd(
    app_handle: tauri::AppHandle,
    source_script_id: String,
    overwrite_cloud_id: bool, // 是否覆盖已存在的 cloud_id Dev
) -> ApiResponse<String> {
    use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, TaskId, UserId};
    use domain_script::ScriptType;
    // 1. Fetch Script
    let script = match uuid::Uuid::parse_str(&source_script_id) {
        Ok(id) => get_script(ScriptId::from(id)).await.unwrap_or(None),
        Err(_) => None,
    };

    let mut script = match script {
        Some(s) => s,
        None => return ApiResponse::error(Some("源脚本不存在".to_string())),
    };

    let current_user = load_current_authenticated_user(&app_handle);
    let current_user_id = current_user.as_ref().and_then(|user| user.id.as_deref());
    let current_username = current_user.as_ref().map(|user| user.username.as_str());
    let owner_user_id = script.info.user_id.to_string();

    // 2. Permission Check
    let is_script_owner = current_user_id == Some(owner_user_id.as_str())
        || current_username == script.info.user_name.as_deref();
    if let Err(error) = ensure_clone_allowed(script.info.allow_clone, is_script_owner) {
        return ApiResponse::error(Some(error.to_string()));
    }

    // 3. Setup cloning IDs mappings
    let new_script_id = ScriptId::new_v7();
    let mut policy_map: std::collections::HashMap<PolicyId, PolicyId> =
        std::collections::HashMap::new();
    let mut group_map: std::collections::HashMap<PolicyGroupId, PolicyGroupId> =
        std::collections::HashMap::new();
    let mut set_map: std::collections::HashMap<PolicySetId, PolicySetId> =
        std::collections::HashMap::new();

    // 4. Handle cloud_id and script_type updates
    let source_type = script.info.script_type.clone();
    script.info.name = format!("{} (Clone)", script.info.name);
    script.info.script_type = ScriptType::Dev;

    if let Some(user_uuid) = current_user_id.and_then(|value| uuid::Uuid::parse_str(value).ok()) {
        script.info.user_id = UserId::from(user_uuid);
    }

    let mut target_delete_id: Option<ScriptId> = None;

    let cloud_id = clone_cloud_id(
        &source_type,
        script.id,
        script.info.cloud_id,
        overwrite_cloud_id,
    );
    if let Some(cloud_id) = cloud_id {
        let existing_dev = find_dev_script_by_cloud_id(cloud_id).await.unwrap_or(None);
        target_delete_id = existing_dev.map(|script| script.id);
    }
    script.info.cloud_id = cloud_id;

    script.id = new_script_id.clone();

    // 5. Gather all tables
    let source_id = match uuid::Uuid::parse_str(&source_script_id) {
        Ok(id) => ScriptId::from(id),
        Err(_) => return ApiResponse::error(Some("源脚本 ID 格式无效".to_string())),
    };
    let mut policies: Vec<PolicyProfile> = list_policies(source_id).await.unwrap_or_default();
    let mut policy_groups: Vec<PolicyGroupProfile> =
        list_policy_groups(source_id).await.unwrap_or_default();
    let mut policy_sets: Vec<PolicySetProfile> =
        list_policy_sets(source_id).await.unwrap_or_default();
    let mut tasks: Vec<ScriptTaskProfile> = list_script_tasks(source_id).await.unwrap_or_default();
    let mut group_policies: Vec<PolicyGroupPolicyLink> =
        list_group_policy_links(source_id).await.unwrap_or_default();
    let mut set_groups: Vec<PolicySetGroupLink> =
        list_set_group_links(source_id).await.unwrap_or_default();

    // 6. Rewrite UUIDs
    for p in policies.iter_mut() {
        let n = PolicyId::new_v7();
        policy_map.insert(p.id.clone(), n.clone());
        p.id = n;
        p.script_id = new_script_id.clone();
    }
    for g in policy_groups.iter_mut() {
        let n = PolicyGroupId::new_v7();
        group_map.insert(g.id.clone(), n.clone());
        g.id = n;
        g.script_id = new_script_id.clone();
    }
    for s in policy_sets.iter_mut() {
        let n = PolicySetId::new_v7();
        set_map.insert(s.id.clone(), n.clone());
        s.id = n;
        s.script_id = new_script_id.clone();
    }
    for t in tasks.iter_mut() {
        t.id = TaskId::new_v7();
        t.script_id = new_script_id.clone();
    }
    for gp in group_policies.iter_mut() {
        if let Some(n) = group_map.get(&gp.group_id) {
            gp.group_id = n.clone();
        }
        if let Some(n) = policy_map.get(&gp.policy_id) {
            gp.policy_id = n.clone();
        }
    }
    for sg in set_groups.iter_mut() {
        if let Some(n) = set_map.get(&sg.set_id) {
            sg.set_id = n.clone();
        }
        if let Some(n) = group_map.get(&sg.group_id) {
            sg.group_id = n.clone();
        }
    }

    // 7. Push to Transaction
    let affected_device_ids = match target_delete_id {
        Some(target_script_id) => load_assigned_device_ids_by_script(target_script_id).await,
        None => Ok(Vec::new()),
    };
    let affected_device_ids = match affected_device_ids {
        Ok(device_ids) => device_ids,
        Err(error) => return ApiResponse::error(Some(error)),
    };

    if let Err(e) = save_cloned_script_graph(
        target_delete_id,
        &script,
        &policies,
        &policy_groups,
        &policy_sets,
        &group_policies,
        &set_groups,
        &tasks,
    )
    .await
    {
        return ApiResponse::error(Some(e));
    }

    notify_auto_dispatch_planner();
    if let Err(error) = enqueue_device_runtime_session_refresh_jobs(
        &app_handle,
        affected_device_ids,
        true,
        false,
        "clone_local_script",
    ) {
        return ApiResponse::error(Some(error));
    }

    ApiResponse::success(
        Some(new_script_id.to_string()),
        Some("复制成功".to_string()),
    )
}
