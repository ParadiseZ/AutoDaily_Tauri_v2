#![allow(dead_code)]

use crate::constant::table_name::ASSIGNMENT_SCHEDULE_TABLE;
use crate::domain::devices::device_schedule::{
    AssignmentSchedule, AssignmentScheduleStatus, AssignmentTriggerSource,
};
use crate::infrastructure::core::{
    AssignmentId, AssignmentScheduleId, BatchId, DeviceId, DispatchId, HashMap, ScriptId,
    TemplateId,
};
use crate::infrastructure::db::get_pool;
use crate::infrastructure::ipc::message::RuntimeQueueItem;
use std::collections::HashSet;

pub fn assignment_schedule_status_value(status: &AssignmentScheduleStatus) -> &'static str {
    match status {
        AssignmentScheduleStatus::Planned => "planned",
        AssignmentScheduleStatus::Dispatched => "dispatched",
        AssignmentScheduleStatus::Running => "running",
        AssignmentScheduleStatus::Success => "success",
        AssignmentScheduleStatus::Failed => "failed",
        AssignmentScheduleStatus::Skipped => "skipped",
        AssignmentScheduleStatus::Cancelled => "cancelled",
        AssignmentScheduleStatus::Stopped => "stopped",
    }
}

pub fn assignment_trigger_source_value(source: &AssignmentTriggerSource) -> &'static str {
    match source {
        AssignmentTriggerSource::Planner => "planner",
        AssignmentTriggerSource::User => "user",
        AssignmentTriggerSource::Debug => "debug",
    }
}

fn assignment_schedule_select_sql() -> String {
    format!(
        "SELECT id, batch_id, device_id, assignment_id, script_id, time_template_id,
                window_start_at, scope_hash, dispatch_id, order_index, created_at,
                run_target_json, status, trigger_source, started_at, completed_at, message
         FROM {}",
        ASSIGNMENT_SCHEDULE_TABLE
    )
}

fn queue_item_scope_key(item: &RuntimeQueueItem) -> String {
    format!(
        "{}|{}|{}|{}",
        item.assignment_id,
        item.window_start_at.clone().unwrap_or_default(),
        item.dedup_scope_base_hash,
        item.order_index
    )
}

fn queue_item_carryover_scope_key(item: &RuntimeQueueItem) -> String {
    format!(
        "{}|{}|{}",
        item.assignment_id, item.dedup_scope_base_hash, item.order_index
    )
}

fn schedule_scope_key(record: &AssignmentSchedule) -> Option<String> {
    record.assignment_id.map(|assignment_id| {
        format!(
            "{}|{}|{}|{}",
            assignment_id,
            record.window_start_at.clone().unwrap_or_default(),
            record.scope_hash,
            record.order_index
        )
    })
}

fn schedule_carryover_scope_key(record: &AssignmentSchedule) -> Option<String> {
    record.assignment_id.map(|assignment_id| {
        format!(
            "{}|{}|{}",
            assignment_id, record.scope_hash, record.order_index
        )
    })
}

fn active_schedule_status(status: &str) -> bool {
    matches!(
        status,
        "planned" | "dispatched" | "running" | "success" | "failed" | "skipped" | "stopped"
    )
}

async fn load_sync_target_planner_batch_ids(device_id: DeviceId) -> Result<Vec<String>, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let rows = sqlx::query_as::<_, (String, String, i64)>(&format!(
        "SELECT batch_id,
                MAX(created_at) AS latest_created_at,
                SUM(CASE WHEN status IN ('planned', 'dispatched', 'running', 'stopped') THEN 1 ELSE 0 END) AS live_count
         FROM {}
         WHERE device_id = ?
           AND trigger_source = 'planner'
           AND created_at LIKE ?
         GROUP BY batch_id
         ORDER BY latest_created_at DESC",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(device_id.to_string())
    .bind(format!("{}%", today))
    .fetch_all(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let active_batches = rows
        .iter()
        .filter(|(_, _, live_count)| *live_count > 0)
        .map(|(batch_id, _, _)| batch_id.clone())
        .collect::<Vec<_>>();
    if !active_batches.is_empty() {
        return Ok(active_batches);
    }

    Ok(vec![rows[0].0.clone()])
}

async fn load_latest_stopped_carryover_scopes(
    device_id: DeviceId,
    trigger_source: AssignmentTriggerSource,
) -> Result<HashSet<String>, String> {
    let query = format!(
        "{}
         WHERE device_id = ?
           AND trigger_source = ?
           AND assignment_id IS NOT NULL
         ORDER BY created_at DESC, order_index ASC",
        assignment_schedule_select_sql()
    );
    let rows = sqlx::query_as::<_, AssignmentSchedule>(&query)
        .bind(device_id.to_string())
        .bind(assignment_trigger_source_value(&trigger_source))
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

    let mut seen = HashSet::new();
    let mut stopped = HashSet::new();
    for row in rows {
        let Some(key) = schedule_carryover_scope_key(&row) else {
            continue;
        };
        if !seen.insert(key.clone()) {
            continue;
        }
        if row.status == "stopped" {
            stopped.insert(key);
        }
    }

    Ok(stopped)
}

pub async fn load_assignment_schedules_by_device(
    device_id: DeviceId,
) -> Result<Vec<AssignmentSchedule>, String> {
    let query = format!(
        "{}
         WHERE device_id = ?
         ORDER BY created_at DESC, order_index ASC",
        assignment_schedule_select_sql()
    );
    sqlx::query_as::<_, AssignmentSchedule>(&query)
        .bind(device_id.to_string())
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())
}

pub async fn find_assignment_schedule_scope(
    assignment_id: AssignmentId,
    window_start_at: Option<&str>,
    trigger_source: AssignmentTriggerSource,
    scope_hash: &str,
) -> Result<Option<AssignmentSchedule>, String> {
    let query = format!(
        "{}
         WHERE assignment_id = ?
           AND ((window_start_at IS NULL AND ?2 IS NULL) OR window_start_at = ?2)
           AND trigger_source = ?
           AND scope_hash = ?
           AND status IN ('planned', 'dispatched', 'running', 'success', 'failed', 'skipped', 'stopped')
         LIMIT 1",
        assignment_schedule_select_sql()
    );
    sqlx::query_as::<_, AssignmentSchedule>(&query)
        .bind(assignment_id.to_string())
        .bind(window_start_at)
        .bind(assignment_trigger_source_value(&trigger_source))
        .bind(scope_hash)
        .fetch_optional(get_pool())
        .await
        .map_err(|error| error.to_string())
}

pub async fn has_complete_assignment_schedule_batch(
    device_id: DeviceId,
    trigger_source: AssignmentTriggerSource,
    items: &[RuntimeQueueItem],
) -> Result<bool, String> {
    if items.is_empty() {
        return Ok(true);
    }

    let expected = items
        .iter()
        .map(queue_item_scope_key)
        .collect::<HashSet<_>>();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let query = format!(
        "{}
         WHERE device_id = ?
           AND trigger_source = ?
           AND status IN ('planned', 'dispatched', 'running', 'success', 'failed', 'skipped', 'stopped')",
        assignment_schedule_select_sql()
    );
    let rows = sqlx::query_as::<_, AssignmentSchedule>(&query)
        .bind(device_id.to_string())
        .bind(assignment_trigger_source_value(&trigger_source))
        .fetch_all(get_pool())
        .await
        .map_err(|error| error.to_string())?;

    let mut batches: HashMap<BatchId, HashSet<String>> = HashMap::new();
    for row in rows {
        if !row.created_at.starts_with(&today) || !active_schedule_status(&row.status) {
            continue;
        }
        let Some(key) = schedule_scope_key(&row) else {
            continue;
        };
        batches.entry(row.batch_id).or_default().insert(key);
    }

    Ok(batches.values().any(|batch| expected.is_subset(batch)))
}

pub async fn insert_assignment_schedule(
    batch_id: BatchId,
    device_id: DeviceId,
    assignment_id: Option<AssignmentId>,
    script_id: Option<ScriptId>,
    time_template_id: Option<TemplateId>,
    window_start_at: Option<String>,
    scope_hash: String,
    dispatch_id: DispatchId,
    order_index: u32,
    created_at: String,
    run_target_json: Option<String>,
    status: AssignmentScheduleStatus,
    trigger_source: AssignmentTriggerSource,
    message: Option<String>,
) -> Result<AssignmentSchedule, String> {
    let record = AssignmentSchedule {
        id: AssignmentScheduleId::new_v7(),
        batch_id,
        device_id,
        assignment_id,
        script_id,
        time_template_id,
        window_start_at,
        scope_hash,
        dispatch_id,
        order_index,
        created_at,
        run_target_json,
        status: assignment_schedule_status_value(&status).to_string(),
        trigger_source: assignment_trigger_source_value(&trigger_source).to_string(),
        started_at: None,
        completed_at: None,
        message,
    };

    sqlx::query(&format!(
        "INSERT INTO {} (
            id, batch_id, device_id, assignment_id, script_id, time_template_id,
            window_start_at, scope_hash, dispatch_id, order_index, created_at,
            run_target_json, status, trigger_source, started_at, completed_at, message
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(record.id.to_string())
    .bind(record.batch_id.to_string())
    .bind(record.device_id.to_string())
    .bind(record.assignment_id.map(|value| value.to_string()))
    .bind(record.script_id.map(|value| value.to_string()))
    .bind(record.time_template_id.map(|value| value.to_string()))
    .bind(record.window_start_at.clone())
    .bind(&record.scope_hash)
    .bind(record.dispatch_id.to_string())
    .bind(record.order_index)
    .bind(&record.created_at)
    .bind(&record.run_target_json)
    .bind(&record.status)
    .bind(&record.trigger_source)
    .bind(&record.started_at)
    .bind(&record.completed_at)
    .bind(&record.message)
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(record)
}

pub async fn insert_assignment_schedule_batch(
    device_id: DeviceId,
    trigger_source: AssignmentTriggerSource,
    items: &[RuntimeQueueItem],
    message: Option<String>,
    preserve_stopped: bool,
) -> Result<Vec<AssignmentSchedule>, String> {
    let batch_id = BatchId::new_v7();
    let created_at = chrono::Local::now().to_rfc3339();
    let stopped_scopes = if preserve_stopped {
        load_latest_stopped_carryover_scopes(device_id, trigger_source.clone()).await?
    } else {
        HashSet::new()
    };
    let mut records = Vec::with_capacity(items.len());
    for item in items {
        let preserve_stopped_record =
            stopped_scopes.contains(&queue_item_carryover_scope_key(item));
        let status = if preserve_stopped_record {
            AssignmentScheduleStatus::Stopped
        } else {
            AssignmentScheduleStatus::Planned
        };
        let record_message = if preserve_stopped_record {
            Some("延续上次停止状态".to_string())
        } else {
            message.clone()
        };
        records.push(
            insert_assignment_schedule(
                batch_id,
                device_id,
                Some(item.assignment_id),
                Some(item.script_id),
                item.time_template_id,
                item.window_start_at.clone(),
                item.dedup_scope_base_hash.clone(),
                item.dispatch_id,
                item.order_index,
                created_at.clone(),
                None,
                status,
                trigger_source.clone(),
                record_message,
            )
            .await?,
        );
    }
    Ok(records)
}

pub async fn reactivate_retryable_planner_schedules_for_device(
    device_id: DeviceId,
    day_prefix: String,
    message: String,
) -> Result<u64, String> {
    let result = sqlx::query(&format!(
        "UPDATE {}
         SET status = ?, started_at = NULL, completed_at = NULL, message = ?
         WHERE device_id = ?
           AND trigger_source = 'planner'
           AND status IN ('stopped', 'failed')
           AND created_at LIKE ?",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(assignment_schedule_status_value(
        &AssignmentScheduleStatus::Planned,
    ))
    .bind(message)
    .bind(device_id.to_string())
    .bind(format!("{}%", day_prefix))
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(result.rows_affected())
}

pub async fn load_next_planned_assignment_schedule(
    device_id: DeviceId,
) -> Result<Option<AssignmentSchedule>, String> {
    let query = format!(
        "{}
         WHERE device_id = ?
           AND status = 'planned'
           AND trigger_source IN ('user', 'planner')
         ORDER BY
           CASE trigger_source WHEN 'user' THEN 0 WHEN 'planner' THEN 1 ELSE 2 END ASC,
           created_at ASC,
           order_index ASC
         LIMIT 1",
        assignment_schedule_select_sql()
    );
    sqlx::query_as::<_, AssignmentSchedule>(&query)
        .bind(device_id.to_string())
        .fetch_optional(get_pool())
        .await
        .map_err(|error| error.to_string())
}

pub async fn sync_active_planner_schedule_order_indices(
    device_id: DeviceId,
    assignment_ids: &[AssignmentId],
) -> Result<u64, String> {
    if assignment_ids.is_empty() {
        return Ok(0);
    }

    let batch_ids = load_sync_target_planner_batch_ids(device_id).await?;

    if batch_ids.is_empty() {
        return Ok(0);
    }

    let desired_order = assignment_ids
        .iter()
        .enumerate()
        .map(|(index, assignment_id)| (*assignment_id, index))
        .collect::<HashMap<_, _>>();
    let mut updated = 0u64;

    for batch_id in batch_ids {
        let query = format!(
            "{}
             WHERE device_id = ?
               AND trigger_source = 'planner'
               AND batch_id = ?
             ORDER BY order_index ASC, created_at ASC",
            assignment_schedule_select_sql()
        );
        let rows = sqlx::query_as::<_, AssignmentSchedule>(&query)
            .bind(device_id.to_string())
            .bind(batch_id)
            .fetch_all(get_pool())
            .await
            .map_err(|error| error.to_string())?;
        if rows.is_empty() {
            continue;
        }

        let mut ordered_rows = rows
            .into_iter()
            .enumerate()
            .map(|(stable_index, row)| {
                let desired_rank = row
                    .assignment_id
                    .and_then(|assignment_id| desired_order.get(&assignment_id).copied());
                (desired_rank, row.order_index, stable_index, row)
            })
            .collect::<Vec<_>>();
        ordered_rows.sort_by_key(|(desired_rank, old_order, stable_index, _)| {
            (
                desired_rank.is_none(),
                desired_rank.unwrap_or(usize::MAX),
                *old_order,
                *stable_index,
            )
        });

        for (new_index, (_, _, _, row)) in ordered_rows.into_iter().enumerate() {
            let new_index = new_index as u32;
            if row.order_index == new_index {
                continue;
            }
            sqlx::query(&format!(
                "UPDATE {}
                 SET order_index = ?
                 WHERE id = ?",
                ASSIGNMENT_SCHEDULE_TABLE
            ))
            .bind(new_index)
            .bind(row.id.to_string())
            .execute(get_pool())
            .await
            .map_err(|error| error.to_string())?;
            updated += 1;
        }
    }

    Ok(updated)
}

fn planner_batch_mutable_status(status: &str) -> bool {
    matches!(status, "planned" | "stopped" | "cancelled")
}

pub async fn sync_active_planner_schedules_from_queue(
    device_id: DeviceId,
    items: &[RuntimeQueueItem],
    reason: &str,
) -> Result<u64, String> {
    let batch_ids = load_sync_target_planner_batch_ids(device_id).await?;

    if batch_ids.is_empty() {
        return Ok(0);
    }

    let now = chrono::Local::now().to_rfc3339();
    let queue_by_assignment = items
        .iter()
        .map(|item| (item.assignment_id, item))
        .collect::<HashMap<_, _>>();
    let assignment_ids = items
        .iter()
        .map(|item| item.assignment_id)
        .collect::<Vec<_>>();
    let mut updated = 0u64;

    for batch_id in batch_ids {
        let query = format!(
            "{}
             WHERE device_id = ?
               AND trigger_source = 'planner'
               AND batch_id = ?
             ORDER BY order_index ASC, created_at ASC",
            assignment_schedule_select_sql()
        );
        let rows = sqlx::query_as::<_, AssignmentSchedule>(&query)
            .bind(device_id.to_string())
            .bind(&batch_id)
            .fetch_all(get_pool())
            .await
            .map_err(|error| error.to_string())?;
        if rows.is_empty() {
            continue;
        }

        let batch_id_value = rows[0].batch_id;
        let batch_created_at = rows[0].created_at.clone();
        let mut seen_assignments = HashSet::new();

        for row in &rows {
            let Some(assignment_id) = row.assignment_id else {
                continue;
            };
            seen_assignments.insert(assignment_id);
            let Some(item) = queue_by_assignment.get(&assignment_id) else {
                if !planner_batch_mutable_status(&row.status) {
                    continue;
                }
                let result = sqlx::query(&format!(
                    "UPDATE {}
                     SET status = ?, completed_at = COALESCE(completed_at, ?), message = ?
                     WHERE id = ?",
                    ASSIGNMENT_SCHEDULE_TABLE
                ))
                .bind(assignment_schedule_status_value(
                    &AssignmentScheduleStatus::Cancelled,
                ))
                .bind(&now)
                .bind("队列定义已删除或已移出当前窗口，取消当前批次未执行记录")
                .bind(row.id.to_string())
                .execute(get_pool())
                .await
                .map_err(|error| error.to_string())?;
                updated += result.rows_affected();
                continue;
            };

            if !planner_batch_mutable_status(&row.status) {
                continue;
            }

            let next_status = if row.status == "stopped" {
                AssignmentScheduleStatus::Stopped
            } else {
                AssignmentScheduleStatus::Planned
            };
            let next_message = if row.status == "stopped" {
                "队列定义变更，当前批次保持停止状态"
            } else {
                reason
            };
            let result = sqlx::query(&format!(
                "UPDATE {}
                 SET script_id = ?,
                     time_template_id = ?,
                     window_start_at = ?,
                     scope_hash = ?,
                     run_target_json = NULL,
                     status = ?,
                     completed_at = CASE WHEN ? = 'planned' THEN NULL ELSE completed_at END,
                     message = ?
                 WHERE id = ?",
                ASSIGNMENT_SCHEDULE_TABLE
            ))
            .bind(item.script_id.to_string())
            .bind(item.time_template_id.map(|value| value.to_string()))
            .bind(item.window_start_at.clone())
            .bind(&item.dedup_scope_base_hash)
            .bind(assignment_schedule_status_value(&next_status))
            .bind(assignment_schedule_status_value(&next_status))
            .bind(next_message)
            .bind(row.id.to_string())
            .execute(get_pool())
            .await
            .map_err(|error| error.to_string())?;
            updated += result.rows_affected();
        }

        for item in items {
            if seen_assignments.contains(&item.assignment_id) {
                continue;
            }
            insert_assignment_schedule(
                batch_id_value,
                device_id,
                Some(item.assignment_id),
                Some(item.script_id),
                item.time_template_id,
                item.window_start_at.clone(),
                item.dedup_scope_base_hash.clone(),
                item.dispatch_id,
                item.order_index,
                batch_created_at.clone(),
                None,
                AssignmentScheduleStatus::Planned,
                AssignmentTriggerSource::Planner,
                Some(reason.to_string()),
            )
            .await?;
            updated += 1;
        }
    }

    updated +=
        sync_active_planner_schedule_order_indices(device_id, assignment_ids.as_slice()).await?;
    Ok(updated)
}

pub async fn update_assignment_schedule_status(
    schedule_id: AssignmentScheduleId,
    status: AssignmentScheduleStatus,
    started_at: Option<String>,
    completed_at: Option<String>,
    message: Option<String>,
) -> Result<(), String> {
    sqlx::query(&format!(
        "UPDATE {}
         SET status = ?, started_at = COALESCE(?, started_at), completed_at = COALESCE(?, completed_at), message = ?
         WHERE id = ? AND status NOT IN ('cancelled', 'stopped')",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(assignment_schedule_status_value(&status))
    .bind(started_at)
    .bind(completed_at)
    .bind(message)
    .bind(schedule_id.to_string())
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

pub async fn update_assignment_schedule_status_by_dispatch_id(
    dispatch_id: DispatchId,
    status: AssignmentScheduleStatus,
    started_at: Option<String>,
    completed_at: Option<String>,
    message: Option<String>,
) -> Result<(), String> {
    sqlx::query(&format!(
        "UPDATE {}
         SET status = ?, started_at = COALESCE(?, started_at), completed_at = COALESCE(?, completed_at), message = ?
         WHERE dispatch_id = ? AND status NOT IN ('cancelled', 'stopped')",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(assignment_schedule_status_value(&status))
    .bind(started_at)
    .bind(completed_at)
    .bind(message)
    .bind(dispatch_id.to_string())
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

pub async fn stop_active_assignment_schedules_by_device(
    device_id: DeviceId,
    completed_at: String,
    message: String,
) -> Result<u64, String> {
    let result = sqlx::query(&format!(
        "UPDATE {}
         SET status = ?, completed_at = COALESCE(completed_at, ?), message = ?
         WHERE device_id = ?
           AND status IN ('planned', 'dispatched', 'running')",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(assignment_schedule_status_value(
        &AssignmentScheduleStatus::Stopped,
    ))
    .bind(completed_at)
    .bind(message)
    .bind(device_id.to_string())
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(result.rows_affected())
}

pub async fn fail_active_assignment_schedules_by_device(
    device_id: DeviceId,
    completed_at: String,
    message: String,
) -> Result<u64, String> {
    let result = sqlx::query(&format!(
        "UPDATE {}
         SET status = ?, completed_at = COALESCE(completed_at, ?), message = ?
         WHERE device_id = ?
           AND status IN ('dispatched', 'running')",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(assignment_schedule_status_value(
        &AssignmentScheduleStatus::Failed,
    ))
    .bind(completed_at)
    .bind(message)
    .bind(device_id.to_string())
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(result.rows_affected())
}

pub async fn stop_planned_planner_schedules_by_device(
    device_id: DeviceId,
    completed_at: String,
    message: String,
) -> Result<u64, String> {
    let result = sqlx::query(&format!(
        "UPDATE {}
         SET status = ?, completed_at = COALESCE(completed_at, ?), message = ?
         WHERE device_id = ?
           AND trigger_source = 'planner'
           AND status = 'planned'",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(assignment_schedule_status_value(
        &AssignmentScheduleStatus::Stopped,
    ))
    .bind(completed_at)
    .bind(message)
    .bind(device_id.to_string())
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(result.rows_affected())
}
