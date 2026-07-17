use super::LoadedScriptBundle;
use ad_kernel::ids::{AccountId, DeviceId, DispatchId, ScriptId, TemplateId};
use chrono::{Days, TimeZone, Timelike};
use domain_schedule::{AssignmentProfile, TemplateValueProfile, TimeTemplateProfile};
use domain_schedule::{TimeOfDay, TimeWindow};
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptTaskProfile, TaskRowType,
};
use infra_sqlite::{
    find_template_value, get_script, get_time_template, list_assignments, list_group_policy_links,
    list_policies, list_policy_groups, list_policy_sets, list_script_tasks, list_set_group_links,
};
use runner_protocol::message::{DispatchKind, DispatchSource, RunTarget, RuntimeQueueItem};
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

fn compute_window_start_at(
    template: &TimeTemplateProfile,
    now: chrono::DateTime<chrono::Local>,
) -> Result<Option<String>, String> {
    let window = TimeWindow::parse(template.start_time.as_deref(), template.end_time.as_deref())
        .map_err(|error| error.to_string())?;
    if window.is_unbounded() {
        return Ok(None);
    }

    let today = now.date_naive();
    let now_time = TimeOfDay::from_hour_minute(now.hour() as u8, now.minute() as u8)
        .ok_or_else(|| "构造当前时间失败".to_string())?;
    if !window.contains(now_time) {
        return Ok(None);
    }

    let window_start = match window.start() {
        None => today
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "构造时间窗口起点失败".to_string())?,
        Some(start) => {
            let date = if window.starts_previous_day(now_time) {
                today - Days::new(1)
            } else {
                today
            };
            date.and_hms_opt(start.hour() as u32, start.minute() as u32, 0)
                .ok_or_else(|| "构造时间窗口起点失败".to_string())?
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

async fn load_time_template(
    time_template_id: TemplateId,
) -> Result<Option<TimeTemplateProfile>, String> {
    get_time_template(time_template_id).await
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
    template: Option<&TimeTemplateProfile>,
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
) -> Result<Option<TemplateValueProfile>, String> {
    let account_id = normalize_account_id(account_id);
    find_template_value(script_id, time_template_id, Some(device_id), account_id).await
}

async fn load_script_bundle(script_id: ScriptId) -> Result<LoadedScriptBundle, String> {
    let script = get_script(script_id)
        .await?
        .ok_or_else(|| format!("脚本[{}]不存在", script_id))?;

    let tasks: Vec<ScriptTaskProfile> = list_script_tasks(script_id).await?;
    let policies: Vec<PolicyProfile> = list_policies(script_id).await?;
    let policy_groups: Vec<PolicyGroupProfile> = list_policy_groups(script_id).await?;
    let policy_sets: Vec<PolicySetProfile> = list_policy_sets(script_id).await?;
    let group_policies: Vec<PolicyGroupPolicyLink> = list_group_policy_links(script_id).await?;
    let set_groups: Vec<PolicySetGroupLink> = list_set_group_links(script_id).await?;

    let runnable_task_ids = tasks
        .iter()
        .filter(|task| task.row_type == TaskRowType::Task && !task.is_deleted)
        .map(|task| task.id)
        .collect();

    let script_name = script.info.name.clone();
    let script_type = script.info.script_type.clone();
    let policy_ids = policies.iter().map(|policy| policy.id).collect();
    let policy_group_ids = policy_groups.iter().map(|group| group.id).collect();
    let policy_set_ids = policy_sets.iter().map(|set| set.id).collect();

    Ok(LoadedScriptBundle {
        script_id,
        script_name,
        script_type,
        runnable_task_ids,
        policy_ids,
        policy_group_ids,
        policy_set_ids,
        snapshot: runner_protocol::message::ScriptBundleSnapshot {
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
    let assignments = list_assignments(device_id).await?;

    let mut queue = Vec::with_capacity(assignments.len());
    for assignment in assignments {
        queue.push(build_runtime_queue_item(device_id, assignment, None).await?);
    }

    Ok(queue)
}

pub(super) async fn load_runtime_queue_for_current_window(
    device_id: DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    let assignments = list_assignments(device_id).await?;

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
    assignment: AssignmentProfile,
    window_start_at: Option<String>,
) -> Result<RuntimeQueueItem, String> {
    let account_data_json =
        serde_json::to_string(&assignment.account_data).map_err(|e| e.to_string())?;
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
        .map(|record| serde_json::to_string(&record.values).map_err(|e| e.to_string()))
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
    use ad_kernel::ids::UuidV7;
    use domain_schedule::TimeTemplateProfile;

    fn sample_template(start_time: Option<&str>, end_time: Option<&str>) -> TimeTemplateProfile {
        TimeTemplateProfile {
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
