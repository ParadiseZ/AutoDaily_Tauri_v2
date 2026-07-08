use crate::api::api_response::ApiResponse;
use crate::api::backend_cmd::local_scripts_dir;
use crate::api::backend_dto::apply_current_client_capability;
use crate::api::backend_dto::ScriptEditorSaveRequest;
use crate::api::infrastructure::process_api::{
    enqueue_device_runtime_session_refresh_jobs, load_assigned_device_ids_by_script,
    notify_auto_dispatch_planner,
};
use crate::api::infrastructure::profile_cache::load_current_authenticated_user;
use crate::constant::table_name::{SCRIPT_TABLE, SCRIPT_TASK_TABLE};
use crate::domain::scripts::policy::*;
use crate::domain::scripts::script_info::{ScriptTable, ScriptType};
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::infrastructure::core::{PolicyGroupId, PolicyId, PolicySetId, ScriptId};
use crate::infrastructure::db::{get_pool, DbRepo};
use std::collections::HashSet;
use tauri::command;

const PUBLISHED_SCRIPT_EDIT_ERROR: &str = "云端下载脚本不可直接编辑，请先克隆为本地脚本";

async fn load_script_for_write(script_id: ScriptId) -> Result<ScriptTable, String> {
    DbRepo::get_by_id(SCRIPT_TABLE, &script_id.to_string())
        .await?
        .ok_or_else(|| "脚本不存在".to_string())
}

fn ensure_script_is_editable(script: &ScriptTable) -> Result<(), String> {
    if script.data.script_type == ScriptType::Published {
        return Err(PUBLISHED_SCRIPT_EDIT_ERROR.to_string());
    }
    Ok(())
}

/// 获取所有脚本配置
#[command]
pub async fn get_all_scripts_cmd() -> Result<Vec<ScriptTable>, String> {
    DbRepo::get_all::<ScriptTable>(SCRIPT_TABLE).await
}

/// 根据 ID 获取脚本配置
#[command]
pub async fn get_script_by_id_cmd(script_id: ScriptId) -> Result<Option<ScriptTable>, String> {
    DbRepo::get_by_id(SCRIPT_TABLE, &script_id.to_string()).await
}

/// 保存（新增或更新）脚本配置
#[command]
pub async fn save_script_cmd(
    app_handle: tauri::AppHandle,
    mut script: ScriptTable,
) -> Result<(), String> {
    if script.data.script_type == ScriptType::Published {
        return Err(PUBLISHED_SCRIPT_EDIT_ERROR.to_string());
    }

    if let Some(existing) =
        DbRepo::get_by_id::<ScriptTable>(SCRIPT_TABLE, &script.id.to_string()).await?
    {
        ensure_script_is_editable(&existing)?;
    }

    apply_current_client_capability(&mut script.data);

    let affected_device_ids = load_assigned_device_ids_by_script(script.id).await?;
    DbRepo::upsert_id_data(SCRIPT_TABLE, &script.id.to_string(), &script.data).await?;
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
    mut payload: ScriptEditorSaveRequest,
) -> Result<(), String> {
    if payload.script.data.script_type == ScriptType::Published {
        return Err(PUBLISHED_SCRIPT_EDIT_ERROR.to_string());
    }

    let existing = load_script_for_write(payload.script.id).await?;
    ensure_script_is_editable(&existing)?;
    apply_current_client_capability(&mut payload.script.data);

    let script_id = payload.script.id.to_string();
    let policy_ids = payload
        .policies
        .iter()
        .map(|policy| policy.id.to_string())
        .collect::<HashSet<_>>();
    let policy_group_ids = payload
        .policy_groups
        .iter()
        .map(|group| group.id.to_string())
        .collect::<HashSet<_>>();
    let policy_set_ids = payload
        .policy_sets
        .iter()
        .map(|set| set.id.to_string())
        .collect::<HashSet<_>>();

    let group_policies = payload
        .group_policy_ids_by_group_id
        .iter()
        .filter(|(group_id, _)| policy_group_ids.contains(group_id.as_str()))
        .flat_map(|(group_id, policy_ids_for_group)| {
            parse_relation_ids(policy_ids_for_group, &policy_ids)
                .into_iter()
                .enumerate()
                .filter_map(|(order_index, policy_id)| {
                    Some(GroupPolicyRelation {
                        group_id: PolicyGroupId::from(uuid::Uuid::parse_str(group_id).ok()?),
                        policy_id: PolicyId::from(uuid::Uuid::parse_str(&policy_id).ok()?),
                        order_index: order_index as i32,
                    })
                })
        })
        .collect::<Vec<_>>();
    let set_groups = payload
        .set_group_ids_by_set_id
        .iter()
        .filter(|(set_id, _)| policy_set_ids.contains(set_id.as_str()))
        .flat_map(|(set_id, group_ids_for_set)| {
            parse_relation_ids(group_ids_for_set, &policy_group_ids)
                .into_iter()
                .enumerate()
                .filter_map(|(order_index, group_id)| {
                    Some(SetGroupRelation {
                        set_id: PolicySetId::from(uuid::Uuid::parse_str(set_id).ok()?),
                        group_id: PolicyGroupId::from(uuid::Uuid::parse_str(&group_id).ok()?),
                        order_index: order_index as i32,
                    })
                })
        })
        .collect::<Vec<_>>();

    let affected_device_ids = load_assigned_device_ids_by_script(payload.script.id).await?;
    let pool = get_pool();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        "DELETE FROM group_policies WHERE group_id IN (SELECT id FROM policy_groups WHERE script_id = ?)",
    )
    .bind(&script_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("清理 group_policies 失败: {}", e))?;

    sqlx::query(
        "DELETE FROM set_groups WHERE set_id IN (SELECT id FROM policy_sets WHERE script_id = ?)",
    )
    .bind(&script_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("清理 set_groups 失败: {}", e))?;

    for table in ["script_tasks", "policies", "policy_groups", "policy_sets"] {
        let query = format!("DELETE FROM {} WHERE script_id = ?", table);
        sqlx::query(&query)
            .bind(&script_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("清理 {} 失败: {}", table, e))?;
    }

    crate::api::domain::script_batch_insert::batch_insert_script_related(
        &mut tx,
        &payload.script,
        &payload.policies,
        &payload.policy_groups,
        &payload.policy_sets,
        &group_policies,
        &set_groups,
        &payload.tasks,
    )
    .await?;

    tx.commit().await.map_err(|e| e.to_string())?;
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
    DbRepo::delete(SCRIPT_TABLE, &script_id.to_string()).await?;
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
    let pool = get_pool();
    let query = format!(
        "SELECT * FROM {} WHERE script_id = ? ORDER BY `index` ASC, created_at ASC",
        SCRIPT_TASK_TABLE
    );
    let rows: Vec<ScriptTaskTable> = sqlx::query_as(&query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows)
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
    use crate::domain::scripts::script_info::ScriptType;
    use crate::infrastructure::core::{PolicyGroupId, PolicyId, PolicySetId, TaskId, UserId};
    let pool = get_pool();

    // 1. Fetch Script
    let script: Option<ScriptTable> = sqlx::query_as("SELECT id, `data` FROM scripts WHERE id = ?")
        .bind(&source_script_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    let mut script = match script {
        Some(s) => s,
        None => return ApiResponse::error(Some("源脚本不存在".to_string())),
    };

    let current_user = load_current_authenticated_user(&app_handle);
    let current_user_id = current_user.as_ref().and_then(|user| user.id.as_deref());
    let current_username = current_user.as_ref().map(|user| user.username.as_str());
    let owner_user_id = script.data.user_id.to_string();

    // 2. Permission Check
    let is_script_owner = current_user_id == Some(owner_user_id.as_str())
        || current_username == script.data.user_name.as_deref();
    if !script.data.allow_clone && !is_script_owner {
        return ApiResponse::error(Some("该脚本作者未开放克隆权限".to_string()));
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
    let is_published = script.data.script_type == ScriptType::Published;

    script.data.name = format!("{} (Clone)", script.data.name);
    script.data.script_type = ScriptType::Dev;

    if let Some(user_uuid) = current_user_id.and_then(|value| uuid::Uuid::parse_str(value).ok()) {
        script.data.user_id = UserId::from(user_uuid);
    }

    let mut target_delete_id: Option<ScriptId> = None;

    if is_published {
        if overwrite_cloud_id {
            // Check if there is an existing Dev version with this cloud_id
            let existing_dev: Option<ScriptTable> = sqlx::query_as(
                "SELECT id, `data` FROM scripts WHERE json_extract(data, '$.scriptType') = 'Dev' AND json_extract(data, '$.cloudId') = ?"
            ).bind(&script.data.cloud_id.unwrap_or_else(|| script.id).to_string()).fetch_optional(pool).await.unwrap_or(None);

            if let Some(existing) = existing_dev {
                // If the user wants to overwrite their local Dev clone of this Published cloud_id
                target_delete_id = Some(existing.id);
            }
            // Keep the cloud_id
            if script.data.cloud_id.is_none() {
                script.data.cloud_id = Some(script.id);
            }
        } else {
            // New independent copy
            script.data.cloud_id = None;
        }
    } else {
        // Dev clone Dev always creates independent
        script.data.cloud_id = None;
    }

    script.id = new_script_id.clone();

    // 5. Gather all tables
    let mut policies: Vec<PolicyTable> = sqlx::query_as(
        "SELECT id, script_id, order_index, `data` FROM policies WHERE script_id = ?",
    )
    .bind(&source_script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut policy_groups: Vec<PolicyGroupTable> = sqlx::query_as(
        "SELECT id, script_id, order_index, `data` FROM policy_groups WHERE script_id = ?",
    )
    .bind(&source_script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut policy_sets: Vec<PolicySetTable> = sqlx::query_as(
        "SELECT id, script_id, order_index, `data` FROM policy_sets WHERE script_id = ?",
    )
    .bind(&source_script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut tasks: Vec<ScriptTaskTable> =
        sqlx::query_as("SELECT * FROM script_tasks WHERE script_id = ?")
            .bind(&source_script_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    let mut group_policies: Vec<GroupPolicyRelation> = sqlx::query_as("SELECT gp.group_id, gp.policy_id, gp.order_index FROM group_policies gp JOIN policy_groups g ON gp.group_id = g.id WHERE g.script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();
    let mut set_groups: Vec<SetGroupRelation> = sqlx::query_as("SELECT sg.set_id, sg.group_id, sg.order_index FROM set_groups sg JOIN policy_sets s ON sg.set_id = s.id WHERE s.script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();

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

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::error(Some(format!("开启事务失败: {}", e))),
    };

    if let Some(del_id) = target_delete_id {
        if let Err(error) = sqlx::query("DELETE FROM scripts WHERE id = ?")
            .bind(del_id.to_string())
            .execute(&mut *tx)
            .await
        {
            return ApiResponse::error(Some(format!("删除被覆盖脚本失败: {}", error)));
        }
    }

    if let Err(e) = crate::api::domain::script_batch_insert::batch_insert_script_related(
        &mut tx,
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

    if let Err(e) = tx.commit().await {
        return ApiResponse::error(Some(e.to_string()));
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
