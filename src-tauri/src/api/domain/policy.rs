use crate::constant::table_name::{POLICY_GROUP_TABLE, POLICY_SET_TABLE, POLICY_TABLE};
use crate::infrastructure::core::{PolicyGroupId, PolicyId, PolicySetId};
use crate::infrastructure::db::{DbRepo, get_pool};
use tauri::command;
use crate::domain::scripts::policy::{PolicyTable, PolicyGroupTable, PolicySetTable, GroupPolicyRelation, SetGroupRelation};
use sqlx::types::Json;

// --- Policy Commands ---

#[command]
pub async fn get_all_policies_cmd() -> Result<Vec<PolicyTable>, String> {
    DbRepo::get_all::<PolicyTable>(POLICY_TABLE).await
}

#[command]
pub async fn save_policy_cmd(policy: PolicyTable) -> Result<(), String> {
    DbRepo::upsert_id_data(POLICY_TABLE, &policy.id.to_string(), &policy.data).await
}

#[command]
pub async fn delete_policy_cmd(id: PolicyId) -> Result<(), String> {
    DbRepo::delete(POLICY_TABLE, &id.to_string()).await
}

// --- Policy Group Commands ---

#[command]
pub async fn get_all_policy_groups_cmd() -> Result<Vec<PolicyGroupTable>, String> {
    DbRepo::get_all::<PolicyGroupTable>(POLICY_GROUP_TABLE).await
}

#[command]
pub async fn save_policy_group_cmd(group: PolicyGroupTable) -> Result<(), String> {
    DbRepo::upsert_id_data(POLICY_GROUP_TABLE, &group.id.to_string(), &group.data).await
}

#[command]
pub async fn delete_policy_group_cmd(id: PolicyGroupId) -> Result<(), String> {
    DbRepo::delete(POLICY_GROUP_TABLE, &id.to_string()).await
}

#[command]
pub async fn get_group_policies_cmd(group_id: PolicyGroupId) -> Result<Vec<PolicyId>, String> {
    let pool = get_pool();
    let rows = sqlx::query_as::<_, GroupPolicyRelation>(
        "SELECT group_id, policy_id, order_index FROM group_policies WHERE group_id = ? ORDER BY order_index"
    )
    .bind(group_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(rows.into_iter().map(|r| r.policy_id).collect())
}

#[command]
pub async fn update_group_policies_cmd(group_id: PolicyGroupId, policy_ids: Vec<PolicyId>) -> Result<(), String> {
    let pool = get_pool();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    
    sqlx::query("DELETE FROM group_policies WHERE group_id = ?")
        .bind(group_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        
    for (idx, policy_id) in policy_ids.into_iter().enumerate() {
        sqlx::query("INSERT INTO group_policies (group_id, policy_id, order_index) VALUES (?, ?, ?)")
            .bind(group_id.to_string())
            .bind(policy_id.to_string())
            .bind(idx as i32)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// --- Policy Set Commands ---

#[command]
pub async fn get_all_policy_sets_cmd() -> Result<Vec<PolicySetTable>, String> {
    DbRepo::get_all::<PolicySetTable>(POLICY_SET_TABLE).await
}

#[command]
pub async fn save_policy_set_cmd(set: PolicySetTable) -> Result<(), String> {
    DbRepo::upsert_id_data(POLICY_SET_TABLE, &set.id.to_string(), &set.data).await
}

#[command]
pub async fn delete_policy_set_cmd(id: PolicySetId) -> Result<(), String> {
    DbRepo::delete(POLICY_SET_TABLE, &id.to_string()).await
}

#[command]
pub async fn get_set_groups_cmd(set_id: PolicySetId) -> Result<Vec<PolicyGroupId>, String> {
    let pool = get_pool();
    let rows = sqlx::query_as::<_, SetGroupRelation>(
        "SELECT set_id, group_id, order_index FROM set_groups WHERE set_id = ? ORDER BY order_index"
    )
    .bind(set_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(rows.into_iter().map(|r| r.group_id).collect())
}

#[command]
pub async fn update_set_groups_cmd(set_id: PolicySetId, group_ids: Vec<PolicyGroupId>) -> Result<(), String> {
    let pool = get_pool();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    
    sqlx::query("DELETE FROM set_groups WHERE set_id = ?")
        .bind(set_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        
    for (idx, group_id) in group_ids.into_iter().enumerate() {
        sqlx::query("INSERT INTO set_groups (set_id, group_id, order_index) VALUES (?, ?, ?)")
            .bind(set_id.to_string())
            .bind(group_id.to_string())
            .bind(idx as i32)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
