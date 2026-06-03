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
use crate::infrastructure::ipc::message::{RuntimeQueueItem, RuntimeSessionSnapshot};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Clone, Debug, Default)]
pub struct DeviceDispatchState {
    pub active_dispatch: Option<DispatchId>,
    pub pending_dispatches: VecDeque<RuntimeQueueItem>,
    pub pending_debug_sessions: VecDeque<RuntimeSessionSnapshot>,
}

pub struct DispatchPlanner {
    device_states: Arc<RwLock<HashMap<DeviceId, DeviceDispatchState>>>,
}

static DISPATCH_PLANNER: OnceLock<Arc<DispatchPlanner>> = OnceLock::new();

impl DispatchPlanner {
    pub fn new() -> Self {
        Self {
            device_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn init() -> Arc<Self> {
        DISPATCH_PLANNER
            .get_or_init(|| Arc::new(Self::new()))
            .clone()
    }

    pub fn get() -> Option<Arc<Self>> {
        DISPATCH_PLANNER.get().cloned()
    }

    pub fn ensure_device_state(&self, device_id: DeviceId) -> Result<(), String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        guard.entry(device_id).or_default();
        Ok(())
    }

    pub fn replace_pending_dispatches(
        &self,
        device_id: DeviceId,
        queue: Vec<RuntimeQueueItem>,
    ) -> Result<(), String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        let state = guard.entry(device_id).or_default();
        state.pending_dispatches = queue.into_iter().collect();
        Ok(())
    }

    pub fn push_debug_session(
        &self,
        device_id: DeviceId,
        session: RuntimeSessionSnapshot,
    ) -> Result<(), String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        let state = guard.entry(device_id).or_default();
        state.pending_debug_sessions.push_back(session);
        Ok(())
    }

    pub fn pop_debug_session(
        &self,
        device_id: DeviceId,
    ) -> Result<Option<RuntimeSessionSnapshot>, String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        let state = guard.entry(device_id).or_default();
        Ok(state.pending_debug_sessions.pop_front())
    }

    pub fn mark_active_dispatch(
        &self,
        device_id: DeviceId,
        dispatch_id: Option<DispatchId>,
    ) -> Result<(), String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        let state = guard.entry(device_id).or_default();
        state.active_dispatch = dispatch_id;
        Ok(())
    }

    pub fn pop_next_dispatch(
        &self,
        device_id: DeviceId,
    ) -> Result<Option<RuntimeQueueItem>, String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        let state = guard.entry(device_id).or_default();
        Ok(state.pending_dispatches.pop_front())
    }

    pub fn snapshot_device_state(
        &self,
        device_id: DeviceId,
    ) -> Result<DeviceDispatchState, String> {
        let guard = self
            .device_states
            .read()
            .map_err(|_| "读取 dispatch planner 状态失败".to_string())?;
        Ok(guard.get(&device_id).cloned().unwrap_or_default())
    }

    pub fn clear_device_state(&self, device_id: DeviceId) -> Result<(), String> {
        let mut guard = self
            .device_states
            .write()
            .map_err(|_| "写入 dispatch planner 状态失败".to_string())?;
        guard.remove(&device_id);
        Ok(())
    }
}

pub fn assignment_schedule_status_value(status: &AssignmentScheduleStatus) -> &'static str {
    match status {
        AssignmentScheduleStatus::Planned => "planned",
        AssignmentScheduleStatus::Dispatched => "dispatched",
        AssignmentScheduleStatus::Running => "running",
        AssignmentScheduleStatus::Success => "success",
        AssignmentScheduleStatus::Failed => "failed",
        AssignmentScheduleStatus::Skipped => "skipped",
        AssignmentScheduleStatus::Cancelled => "cancelled",
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

fn active_schedule_status(status: &str) -> bool {
    matches!(
        status,
        "planned" | "dispatched" | "running" | "success" | "skipped"
    )
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
           AND status IN ('planned', 'dispatched', 'running', 'success', 'skipped')
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
           AND status IN ('planned', 'dispatched', 'running', 'success', 'skipped')",
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

    Ok(batches.values().any(|batch| *batch == expected))
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
) -> Result<Vec<AssignmentSchedule>, String> {
    let batch_id = BatchId::new_v7();
    let created_at = chrono::Local::now().to_rfc3339();
    let mut records = Vec::with_capacity(items.len());
    for item in items {
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
                AssignmentScheduleStatus::Planned,
                trigger_source.clone(),
                message.clone(),
            )
            .await?,
        );
    }
    Ok(records)
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
         WHERE id = ?",
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
         WHERE dispatch_id = ?",
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
