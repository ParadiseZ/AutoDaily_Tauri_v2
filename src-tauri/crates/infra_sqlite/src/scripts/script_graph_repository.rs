use super::policy_record::{GroupPolicyRow, PolicyGroupRow, PolicyRow, PolicySetRow, SetGroupRow};
use super::script_task_record::ScriptTaskRow;
use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, ScriptId};
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptTaskProfile,
};
use sqlx::types::Json;

async fn rows<T, U>(query: &str, script_id: ScriptId) -> Result<Vec<U>, String>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    U: TryFrom<T, Error = String>,
{
    sqlx::query_as::<_, T>(query)
        .bind(script_id.to_string())
        .fetch_all(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

pub async fn list_policies(script_id: ScriptId) -> Result<Vec<PolicyProfile>, String> {
    rows::<PolicyRow, _>("SELECT id, script_id, order_index, `data` FROM policies WHERE script_id = ? ORDER BY order_index", script_id).await
}

pub async fn list_policy_groups(script_id: ScriptId) -> Result<Vec<PolicyGroupProfile>, String> {
    rows::<PolicyGroupRow, _>("SELECT id, script_id, order_index, `data` FROM policy_groups WHERE script_id = ? ORDER BY order_index", script_id).await
}

pub async fn list_policy_sets(script_id: ScriptId) -> Result<Vec<PolicySetProfile>, String> {
    rows::<PolicySetRow, _>("SELECT id, script_id, order_index, `data` FROM policy_sets WHERE script_id = ? ORDER BY order_index", script_id).await
}

pub async fn list_script_tasks(script_id: ScriptId) -> Result<Vec<ScriptTaskProfile>, String> {
    rows::<ScriptTaskRow, _>(
        "SELECT * FROM script_tasks WHERE script_id = ? ORDER BY `index` ASC, created_at ASC",
        script_id,
    )
    .await
}

pub async fn list_group_policy_links(
    script_id: ScriptId,
) -> Result<Vec<PolicyGroupPolicyLink>, String> {
    rows::<GroupPolicyRow, _>("SELECT gp.group_id, gp.policy_id, gp.order_index FROM group_policies gp JOIN policy_groups g ON gp.group_id = g.id WHERE g.script_id = ? ORDER BY g.order_index, gp.order_index", script_id).await
}

pub async fn list_policy_ids_in_group(group_id: PolicyGroupId) -> Result<Vec<PolicyId>, String> {
    sqlx::query_as::<_, GroupPolicyRow>("SELECT group_id, policy_id, order_index FROM group_policies WHERE group_id = ? ORDER BY order_index")
        .bind(group_id.to_string()).fetch_all(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?
        .into_iter().map(TryInto::try_into).collect::<Result<Vec<PolicyGroupPolicyLink>, String>>()
        .map(|links| links.into_iter().map(|link| link.policy_id).collect())
}

pub async fn list_set_group_links(script_id: ScriptId) -> Result<Vec<PolicySetGroupLink>, String> {
    rows::<SetGroupRow, _>("SELECT sg.set_id, sg.group_id, sg.order_index FROM set_groups sg JOIN policy_sets s ON sg.set_id = s.id WHERE s.script_id = ? ORDER BY s.order_index, sg.order_index", script_id).await
}

pub async fn list_group_ids_in_set(set_id: PolicySetId) -> Result<Vec<PolicyGroupId>, String> {
    sqlx::query_as::<_, SetGroupRow>("SELECT set_id, group_id, order_index FROM set_groups WHERE set_id = ? ORDER BY order_index")
        .bind(set_id.to_string()).fetch_all(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?
        .into_iter().map(TryInto::try_into).collect::<Result<Vec<PolicySetGroupLink>, String>>()
        .map(|links| links.into_iter().map(|link| link.group_id).collect())
}

pub async fn save_policy(profile: &PolicyProfile) -> Result<(), String> {
    sqlx::query("INSERT INTO policies (id, script_id, order_index, `data`) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET script_id = excluded.script_id, order_index = excluded.order_index, `data` = excluded.`data`")
        .bind(profile.id.to_string()).bind(profile.script_id.to_string()).bind(profile.order_index).bind(Json(&profile.info))
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn save_policy_group(profile: &PolicyGroupProfile) -> Result<(), String> {
    sqlx::query("INSERT INTO policy_groups (id, script_id, order_index, `data`) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET script_id = excluded.script_id, order_index = excluded.order_index, `data` = excluded.`data`")
        .bind(profile.id.to_string()).bind(profile.script_id.to_string()).bind(profile.order_index).bind(Json(&profile.info))
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn save_policy_set(profile: &PolicySetProfile) -> Result<(), String> {
    sqlx::query("INSERT INTO policy_sets (id, script_id, order_index, `data`) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET script_id = excluded.script_id, order_index = excluded.order_index, `data` = excluded.`data`")
        .bind(profile.id.to_string()).bind(profile.script_id.to_string()).bind(profile.order_index).bind(Json(&profile.info))
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn delete_policy(id: PolicyId) -> Result<(), String> {
    delete("policies", id.to_string()).await
}
pub async fn delete_policy_group(id: PolicyGroupId) -> Result<(), String> {
    delete("policy_groups", id.to_string()).await
}
pub async fn delete_policy_set(id: PolicySetId) -> Result<(), String> {
    delete("policy_sets", id.to_string()).await
}

async fn delete(table: &str, id: String) -> Result<(), String> {
    sqlx::query(&format!("DELETE FROM {table} WHERE id = ?"))
        .bind(id)
        .execute(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn replace_group_policy_links(
    group_id: PolicyGroupId,
    policy_ids: Vec<PolicyId>,
) -> Result<(), String> {
    let mut tx = crate::bootstrap::get_pool()
        .begin()
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM group_policies WHERE group_id = ?")
        .bind(group_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    for (order_index, policy_id) in policy_ids.into_iter().enumerate() {
        sqlx::query(
            "INSERT INTO group_policies (group_id, policy_id, order_index) VALUES (?, ?, ?)",
        )
        .bind(group_id.to_string())
        .bind(policy_id.to_string())
        .bind(order_index as i32)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    }
    tx.commit().await.map_err(|error| error.to_string())
}

pub async fn replace_set_group_links(
    set_id: PolicySetId,
    group_ids: Vec<PolicyGroupId>,
) -> Result<(), String> {
    let mut tx = crate::bootstrap::get_pool()
        .begin()
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM set_groups WHERE set_id = ?")
        .bind(set_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    for (order_index, group_id) in group_ids.into_iter().enumerate() {
        sqlx::query("INSERT INTO set_groups (set_id, group_id, order_index) VALUES (?, ?, ?)")
            .bind(set_id.to_string())
            .bind(group_id.to_string())
            .bind(order_index as i32)
            .execute(&mut *tx)
            .await
            .map_err(|error| error.to_string())?;
    }
    tx.commit().await.map_err(|error| error.to_string())
}
