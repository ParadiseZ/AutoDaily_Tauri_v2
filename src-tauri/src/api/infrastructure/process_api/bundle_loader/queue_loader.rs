use super::LoadedScriptBundle;
use crate::constant::table_name::{
    ASSIGNMENT_TABLE, GROUP_POLICIES, POLICY_GROUP_TABLE, POLICY_SET_TABLE, POLICY_TABLE,
    SCRIPT_TABLE, SCRIPT_TASK_TABLE, SCRIPT_TIME_TEMPLATE_VALUES_TABLE, SET_GROUPS,
    TIME_TEMPLATE_TABLE,
};
use crate::domain::devices::device_schedule::DeviceScriptAssignment;
use crate::domain::schedule::script_time_template_values::ScriptTimeTemplateValuesDto;
use crate::domain::schedule::time_template::TimeTemplate;
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::infrastructure::core::{AccountId, DeviceId, DispatchId, ScriptId, TemplateId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::ipc::message::{
    DispatchKind, DispatchSource, RunTarget, RuntimeQueueItem,
};
use chrono::TimeZone;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

fn parse_hhmm(value: &str) -> Result<chrono::NaiveTime, String> {
    chrono::NaiveTime::parse_from_str(value, "%H:%M")
        .map_err(|error| format!("解析时间模板时间失败[{}]: {}", value, error))
}

fn compute_window_start_at(
    template: &TimeTemplate,
    now: chrono::DateTime<chrono::Local>,
) -> Result<Option<String>, String> {
    let start = template.start_time.as_deref().map(parse_hhmm).transpose()?;
    let end = template.end_time.as_deref().map(parse_hhmm).transpose()?;
    let today = now.date_naive();
    let now_time = now.time();

    let window_start = match (start, end) {
        (None, None) => return Ok(None),
        (Some(start), None) => {
            if now_time < start {
                return Ok(None);
            }
            today.and_time(start)
        }
        (None, Some(end)) => {
            if now_time > end {
                return Ok(None);
            }
            today
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| "构造时间窗口起点失败".to_string())?
        }
        (Some(start), Some(end)) if start <= end => {
            if now_time < start || now_time > end {
                return Ok(None);
            }
            today.and_time(start)
        }
        (Some(start), Some(end)) => {
            if now_time >= start {
                today.and_time(start)
            } else if now_time <= end {
                (today - chrono::Days::new(1)).and_time(start)
            } else {
                return Ok(None);
            }
        }
    };

    Ok(Some(
        chrono::Local
            .from_local_datetime(&window_start)
            .single()
            .ok_or_else(|| "构造时间窗口实例失败".to_string())?
            .to_rfc3339(),
    ))
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

async fn load_time_template(time_template_id: TemplateId) -> Result<Option<TimeTemplate>, String> {
    let query = format!(
        "SELECT id, name, start_time, end_time FROM {} WHERE id = ?",
        TIME_TEMPLATE_TABLE
    );
    sqlx::query_as::<_, TimeTemplate>(&query)
        .bind(time_template_id.to_string())
        .fetch_optional(get_pool())
        .await
        .map_err(|e| e.to_string())
}

fn extract_task_settings_scope(template_values_json: Option<&str>) -> Result<Value, String> {
    let Some(content) = template_values_json else {
        return Ok(Value::Object(Map::new()));
    };
    let root: Value =
        serde_json::from_str(content).map_err(|error| format!("解析模板覆盖值失败: {}", error))?;
    let Some(task_settings) = root.get("taskSettings").and_then(Value::as_object) else {
        return Ok(Value::Object(Map::new()));
    };

    let mut ordered = BTreeMap::new();
    for (task_id, setting) in task_settings {
        let Some(setting_object) = setting.as_object() else {
            continue;
        };
        let mut scoped_setting = Map::new();
        if let Some(enabled) = setting_object
            .get("enabled")
            .filter(|value| !value.is_null())
        {
            scoped_setting.insert("enabled".to_string(), enabled.clone());
        }
        if let Some(task_cycle) = setting_object
            .get("taskCycle")
            .filter(|value| !value.is_null())
        {
            scoped_setting.insert("taskCycle".to_string(), task_cycle.clone());
        }
        if !scoped_setting.is_empty() {
            ordered.insert(task_id.clone(), Value::Object(scoped_setting));
        }
    }

    let mut task_settings_scope = Map::new();
    for (task_id, setting) in ordered {
        task_settings_scope.insert(task_id, setting);
    }
    Ok(Value::Object(task_settings_scope))
}

#[derive(Serialize)]
struct RuntimeDedupScope {
    device_id: String,
    script_id: String,
    time_template_id: Option<String>,
    account_id: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    task_settings: Value,
}

fn compute_dedup_scope_base_hash(
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: Option<TemplateId>,
    account_id: Option<&str>,
    template: Option<&TimeTemplate>,
    template_values_json: Option<&str>,
) -> Result<String, String> {
    let scope = RuntimeDedupScope {
        device_id: device_id.to_string(),
        script_id: script_id.to_string(),
        time_template_id: time_template_id.map(|id| id.to_string()),
        account_id: account_id.map(str::to_string),
        start_time: template.and_then(|item| item.start_time.clone()),
        end_time: template.and_then(|item| item.end_time.clone()),
        task_settings: extract_task_settings_scope(template_values_json)?,
    };
    let json = serde_json::to_vec(&scope).map_err(|error| error.to_string())?;
    let digest = Sha256::digest(json);
    Ok(format!("{:x}", digest))
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

pub(super) async fn load_runtime_queue(
    device_id: DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
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
        queue.push(build_runtime_queue_item(device_id, assignment, None).await?);
    }

    Ok(queue)
}

pub(super) async fn load_runtime_queue_for_current_window(
    device_id: DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    let query = format!(
        "SELECT id, device_id, script_id, time_template_id, account_data, `index` FROM {} WHERE device_id = ? ORDER BY `index` ASC",
        ASSIGNMENT_TABLE
    );
    let assignments = sqlx::query_as::<_, DeviceScriptAssignment>(&query)
        .bind(device_id.to_string())
        .fetch_all(get_pool())
        .await
        .map_err(|e| e.to_string())?;

    let now = chrono::Local::now();
    let mut queue = Vec::new();
    for assignment in assignments {
        let window_start_at = match assignment.time_template_id {
            Some(time_template_id) => {
                let Some(template) = load_time_template(time_template_id).await? else {
                    continue;
                };
                match compute_window_start_at(&template, now)? {
                    Some(window_start_at) => Some(window_start_at),
                    None => continue,
                }
            }
            None => None,
        };
        queue.push(build_runtime_queue_item(device_id, assignment, window_start_at).await?);
    }

    Ok(queue)
}

async fn build_runtime_queue_item(
    device_id: DeviceId,
    assignment: DeviceScriptAssignment,
    window_start_at: Option<String>,
) -> Result<RuntimeQueueItem, String> {
    let account_data_json =
        serde_json::to_string(&assignment.account_data.0).map_err(|e| e.to_string())?;
    let account_id = None;
    let time_template = match assignment.time_template_id {
        Some(time_template_id) => load_time_template(time_template_id).await?,
        None => None,
    };
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
    let dedup_scope_base_hash = compute_dedup_scope_base_hash(
        device_id,
        assignment.script_id,
        assignment.time_template_id,
        account_id.as_deref(),
        time_template.as_ref(),
        template_values_json.as_deref(),
    )?;

    Ok(RuntimeQueueItem {
        dispatch_id: DispatchId::new_v7(),
        dispatch_kind: DispatchKind::QueueAssignment,
        dispatch_source: DispatchSource::User,
        assignment_id: assignment.id,
        script_id: assignment.script_id,
        time_template_id: assignment.time_template_id,
        account_id,
        account_data_json: Some(account_data_json),
        order_index: assignment.index,
        window_start_at,
        template_values_json,
        dedup_scope_base_hash,
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

#[cfg(test)]
mod tests {
    use super::compute_dedup_scope_base_hash;
    use crate::domain::schedule::time_template::TimeTemplate;
    use crate::infrastructure::core::UuidV7;

    fn sample_template(start_time: Option<&str>, end_time: Option<&str>) -> TimeTemplate {
        TimeTemplate {
            id: UuidV7(3),
            name: "sample".to_string(),
            start_time: start_time.map(str::to_string),
            end_time: end_time.map(str::to_string),
        }
    }

    #[test]
    fn dedup_scope_base_hash_ignores_unrelated_template_values() {
        let hash_a = compute_dedup_scope_base_hash(
            UuidV7(1),
            UuidV7(2),
            Some(UuidV7(3)),
            None,
            Some(&sample_template(Some("09:00"), Some("18:00"))),
            Some(
                r#"{"taskSettings":{"task-b":{"taskCycle":"daily"},"task-a":{"enabled":true}},"variables":{"foo":"bar"}}"#,
            ),
        )
        .expect("hash a");
        let hash_b = compute_dedup_scope_base_hash(
            UuidV7(1),
            UuidV7(2),
            Some(UuidV7(3)),
            None,
            Some(&sample_template(Some("09:00"), Some("18:00"))),
            Some(
                r#"{"variables":{"foo":"baz"},"taskSettings":{"task-a":{"enabled":true,"label":"ignored"},"task-b":{"taskCycle":"daily"}}}"#,
            ),
        )
        .expect("hash b");

        assert_eq!(hash_a, hash_b);
    }

    #[test]
    fn dedup_scope_base_hash_changes_with_time_window_or_task_settings() {
        let base = compute_dedup_scope_base_hash(
            UuidV7(1),
            UuidV7(2),
            Some(UuidV7(3)),
            None,
            Some(&sample_template(Some("09:00"), Some("18:00"))),
            Some(r#"{"taskSettings":{"task-a":{"enabled":true}}}"#),
        )
        .expect("base hash");
        let changed_window = compute_dedup_scope_base_hash(
            UuidV7(1),
            UuidV7(2),
            Some(UuidV7(3)),
            None,
            Some(&sample_template(Some("10:00"), Some("18:00"))),
            Some(r#"{"taskSettings":{"task-a":{"enabled":true}}}"#),
        )
        .expect("window hash");
        let changed_task_setting = compute_dedup_scope_base_hash(
            UuidV7(1),
            UuidV7(2),
            Some(UuidV7(3)),
            None,
            Some(&sample_template(Some("09:00"), Some("18:00"))),
            Some(r#"{"taskSettings":{"task-a":{"enabled":false}}}"#),
        )
        .expect("task-setting hash");

        assert_ne!(base, changed_window);
        assert_ne!(base, changed_task_setting);
    }
}
