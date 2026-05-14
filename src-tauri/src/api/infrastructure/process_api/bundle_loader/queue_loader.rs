use super::LoadedScriptBundle;
use crate::constant::table_name::{
    ASSIGNMENT_TABLE, GROUP_POLICIES, POLICY_GROUP_TABLE, POLICY_SET_TABLE, POLICY_TABLE,
    SCRIPT_TABLE, SCRIPT_TASK_TABLE, SCRIPT_TIME_TEMPLATE_VALUES_TABLE, SET_GROUPS,
};
use crate::domain::devices::device_schedule::DeviceScriptAssignment;
use crate::domain::schedule::script_time_template_values::ScriptTimeTemplateValuesDto;
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::infrastructure::core::{AccountId, DeviceId, ScriptId, TemplateId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::ipc::message::{RunTarget, RuntimeQueueItem};
use serde::Serialize;
use std::collections::HashSet;

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
    let script_type = script.data.0.script_type.clone();
    let recovery_task_id = script.data.0.runtime_settings.recovery_task_id;
    let policy_ids = policies.iter().map(|policy| policy.id).collect();
    let policy_group_ids = policy_groups.iter().map(|group| group.id).collect();
    let policy_set_ids = policy_sets.iter().map(|set| set.id).collect();

    Ok(LoadedScriptBundle {
        script_id,
        script_name,
        script_type,
        recovery_task_id,
        runnable_task_ids,
        policy_ids,
        policy_group_ids,
        policy_set_ids,
        snapshot: crate::infrastructure::ipc::message::ScriptBundleSnapshot {
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

pub(super) async fn load_runtime_queue(device_id: DeviceId) -> Result<Vec<RuntimeQueueItem>, String> {
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
        queue.push(build_runtime_queue_item(device_id, assignment).await?);
    }

    Ok(queue)
}

async fn build_runtime_queue_item(
    device_id: DeviceId,
    assignment: DeviceScriptAssignment,
) -> Result<RuntimeQueueItem, String> {
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

    Ok(RuntimeQueueItem {
        assignment_id: assignment.id,
        script_id: assignment.script_id,
        time_template_id: assignment.time_template_id,
        account_id,
        account_data_json: Some(account_data_json),
        order_index: assignment.index,
        template_values_json,
    })
}

pub(super) async fn load_script_bundles(
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
