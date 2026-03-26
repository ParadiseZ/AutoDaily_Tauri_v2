use crate::api::api_response::ApiResponse;
use crate::constant::table_name::{SCRIPT_TABLE, SCRIPT_TASK_TABLE};
use crate::domain::scripts::policy::*;
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::infrastructure::core::ScriptId;
use crate::infrastructure::db::{get_pool, DbRepo};
use tauri::command;

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
pub async fn save_script_cmd(script: ScriptTable) -> Result<(), String> {
    DbRepo::upsert_id_data(SCRIPT_TABLE, &script.id.to_string(), &script.data).await
}

/// 删除脚本配置
#[command]
pub async fn delete_script_cmd(script_id: ScriptId) -> Result<(), String> {
    DbRepo::delete(SCRIPT_TABLE, &script_id.to_string()).await
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

/// 批量保存脚本任务逻辑
/// 这里采用简单策略：先删除该脚本的所有任务，再重新插入
#[command]
pub async fn save_script_tasks_cmd(script_id: ScriptId, tasks: Vec<ScriptTaskTable>) -> Result<(), String> {
    let pool = get_pool();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 删除旧任务
    let delete_query = format!("DELETE FROM {} WHERE script_id = ?", SCRIPT_TASK_TABLE);
    sqlx::query(&delete_query)
        .bind(script_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 插入新任务
    for task in tasks {
        let insert_query = format!(
            "INSERT INTO {} (id, script_id, name, is_hidden, task_type, nodes, edges, `data`, created_at, updated_at, deleted_at, is_deleted, `index`) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            SCRIPT_TASK_TABLE
        );
        sqlx::query(&insert_query)
            .bind(task.id.to_string())
            .bind(script_id.to_string())
            .bind(&task.name)
            .bind(task.is_hidden)
            .bind(task.task_type)
            .bind("[]")
            .bind("[]")
            .bind(&task.data)
            .bind(task.created_at)
            .bind(task.updated_at)
            .bind(task.deleted_at)
            .bind(task.is_deleted)
            .bind(task.index as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取 YOLO 标签文件
#[command]
pub async fn get_yolo_labels_cmd(path: String) -> Result<std::collections::HashMap<u16, String>, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let values: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
    
    if let Some(names) = values.get("names") {
        let labels: std::collections::HashMap<u16, String> = serde_yaml::from_value(names.clone()).map_err(|e| e.to_string())?;
        Ok(labels)
    } else {
        Err("Yolo标签文件格式错误：未找到 names 属性".to_string())
    }
}

/// 克隆本地脚本字典逻辑
/// 1. 权限控制: 如果 allow_clone 为 false 且不属于当前用户，拒绝
/// 2. 克隆 published -> Dev: 根据入参 `overwrite_cloud_id` 决定是覆盖还是新建
/// 3. 克隆 Dev -> Dev: 始终作为独立新副本
#[command]
pub async fn clone_local_script_cmd(
    source_script_id: String,
    current_user_id: Option<String>,
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

    let user_id = current_user_id.unwrap_or_else(|| "".to_string());
    
    // 2. Permission Check
    if !script.data.allow_clone && script.data.user_id.to_string() != user_id {
        return ApiResponse::error(Some("该脚本作者未开放克隆权限".to_string()));
    }

    // 3. Setup cloning IDs mappings
    let new_script_id = ScriptId::new_v7();
    let mut policy_map: std::collections::HashMap<PolicyId, PolicyId> = std::collections::HashMap::new();
    let mut group_map: std::collections::HashMap<PolicyGroupId, PolicyGroupId> = std::collections::HashMap::new();
    let mut set_map: std::collections::HashMap<PolicySetId, PolicySetId> = std::collections::HashMap::new();

    // 4. Handle cloud_id and script_type updates
    let is_published = script.data.script_type == ScriptType::Published;
    
    script.data.name = format!("{} (Clone)", script.data.name);
    script.data.script_type = ScriptType::Dev;
    
    if let Ok(uuid) = uuid::Uuid::parse_str(&user_id) {
        script.data.user_id = UserId::from(uuid);
    }

    let mut target_delete_id: Option<String> = None;

    if is_published {
        if overwrite_cloud_id {
            // Check if there is an existing Dev version with this cloud_id
            let existing_dev: Option<ScriptTable> = sqlx::query_as(
                "SELECT id, `data` FROM scripts WHERE json_extract(data, '$.scriptType') = 'Dev' AND json_extract(data, '$.cloudId') = ?"
            ).bind(&script.data.cloud_id.unwrap_or_else(|| script.id).to_string()).fetch_optional(pool).await.unwrap_or(None);

            if let Some(existing) = existing_dev {
                // If the user wants to overwrite their local Dev clone of this Published cloud_id
                target_delete_id = Some(existing.id.to_string());
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
    let mut policies: Vec<PolicyTable> = sqlx::query_as("SELECT id, script_id, order_index, `data` FROM policies WHERE script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();
    let mut policy_groups: Vec<PolicyGroupTable> = sqlx::query_as("SELECT id, script_id, order_index, `data` FROM policy_groups WHERE script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();
    let mut policy_sets: Vec<PolicySetTable> = sqlx::query_as("SELECT id, script_id, order_index, `data` FROM policy_sets WHERE script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();
    let mut tasks: Vec<ScriptTaskTable> = sqlx::query_as("SELECT * FROM script_tasks WHERE script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();
    let mut group_policies: Vec<GroupPolicyRelation> = sqlx::query_as("SELECT gp.group_id, gp.policy_id, gp.order_index FROM group_policies gp JOIN policy_groups g ON gp.group_id = g.id WHERE g.script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();
    let mut set_groups: Vec<SetGroupRelation> = sqlx::query_as("SELECT sg.set_id, sg.group_id, sg.order_index FROM set_groups sg JOIN policy_sets s ON sg.set_id = s.id WHERE s.script_id = ?").bind(&source_script_id).fetch_all(pool).await.unwrap_or_default();

    // 6. Rewrite UUIDs
    for p in policies.iter_mut() { let n = PolicyId::new_v7(); policy_map.insert(p.id.clone(), n.clone()); p.id = n; p.script_id = new_script_id.clone(); }
    for g in policy_groups.iter_mut() { let n = PolicyGroupId::new_v7(); group_map.insert(g.id.clone(), n.clone()); g.id = n; g.script_id = new_script_id.clone(); }
    for s in policy_sets.iter_mut() { let n = PolicySetId::new_v7(); set_map.insert(s.id.clone(), n.clone()); s.id = n; s.script_id = new_script_id.clone(); }
    for t in tasks.iter_mut() { t.id = TaskId::new_v7(); t.script_id = new_script_id.clone(); }
    for gp in group_policies.iter_mut() {
        if let Some(n) = group_map.get(&gp.group_id) { gp.group_id = n.clone(); }
        if let Some(n) = policy_map.get(&gp.policy_id) { gp.policy_id = n.clone(); }
    }
    for sg in set_groups.iter_mut() {
        if let Some(n) = set_map.get(&sg.set_id) { sg.set_id = n.clone(); }
        if let Some(n) = group_map.get(&sg.group_id) { sg.group_id = n.clone(); }
    }

    // 7. Push to Transaction
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::error(Some(format!("开启事务失败: {}", e))),
    };

    if let Some(del_id) = target_delete_id {
        let _ = sqlx::query("DELETE FROM scripts WHERE id = ?").bind(del_id).execute(&mut *tx).await;
        // Policies, Tasks, Groups cascading delete should trigger automatically IF foreign keys delete logic set. Otherwise:
        let _ = sqlx::query("DELETE FROM policies WHERE script_id = ?").bind(&source_script_id).execute(&mut *tx).await;
        let _ = sqlx::query("DELETE FROM script_tasks WHERE script_id = ?").bind(&source_script_id).execute(&mut *tx).await;
        let _ = sqlx::query("DELETE FROM policy_groups WHERE script_id = ?").bind(&source_script_id).execute(&mut *tx).await;
        let _ = sqlx::query("DELETE FROM policy_sets WHERE script_id = ?").bind(&source_script_id).execute(&mut *tx).await;
    }

    if let Err(e) = crate::api::domain::script_batch_insert::batch_insert_script_related(
        &mut tx, &script, &policies, &policy_groups, &policy_sets, &group_policies, &set_groups, &tasks,
    ).await { return ApiResponse::error(Some(e)); }

    if let Err(e) = tx.commit().await { return ApiResponse::error(Some(e.to_string())); }
    
    ApiResponse::success(Some(new_script_id.to_string()), Some("复制成功".to_string()))
}
