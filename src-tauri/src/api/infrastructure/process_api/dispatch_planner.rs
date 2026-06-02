#![allow(dead_code)]

use crate::constant::table_name::ASSIGNMENT_SCHEDULE_TABLE;
use crate::domain::devices::device_schedule::{
    AssignmentSchedule, AssignmentScheduleStatus, AssignmentTriggerSource,
};
use crate::infrastructure::core::{
    AssignmentId, AssignmentScheduleId, DeviceId, DispatchId, HashMap, TemplateId,
};
use crate::infrastructure::db::get_pool;
use crate::infrastructure::ipc::message::RuntimeQueueItem;
use std::collections::VecDeque;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Clone, Debug, Default)]
pub struct DeviceDispatchState {
    pub active_dispatch: Option<DispatchId>,
    pub pending_dispatches: VecDeque<RuntimeQueueItem>,
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

    pub fn pop_next_dispatch(&self, device_id: DeviceId) -> Result<Option<RuntimeQueueItem>, String> {
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

fn assignment_schedule_status_value(status: &AssignmentScheduleStatus) -> &'static str {
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

fn assignment_trigger_source_value(source: &AssignmentTriggerSource) -> &'static str {
    match source {
        AssignmentTriggerSource::Planner => "planner",
        AssignmentTriggerSource::User => "user",
        AssignmentTriggerSource::Debug => "debug",
    }
}

pub async fn load_assignment_schedules_by_device(
    device_id: DeviceId,
) -> Result<Vec<AssignmentSchedule>, String> {
    let query = format!(
        "SELECT id, device_id, assignment_id, time_template_id, window_start_at, dispatch_id, status, trigger_source, started_at, completed_at, message
         FROM {}
         WHERE device_id = ?
         ORDER BY COALESCE(completed_at, started_at, window_start_at) DESC",
        ASSIGNMENT_SCHEDULE_TABLE
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
) -> Result<Option<AssignmentSchedule>, String> {
    let query = format!(
        "SELECT id, device_id, assignment_id, time_template_id, window_start_at, dispatch_id, status, trigger_source, started_at, completed_at, message
         FROM {}
         WHERE assignment_id = ?
           AND ((window_start_at IS NULL AND ?2 IS NULL) OR window_start_at = ?2)
           AND trigger_source = ?
         LIMIT 1",
        ASSIGNMENT_SCHEDULE_TABLE
    );
    sqlx::query_as::<_, AssignmentSchedule>(&query)
        .bind(assignment_id.to_string())
        .bind(window_start_at)
        .bind(assignment_trigger_source_value(&trigger_source))
        .fetch_optional(get_pool())
        .await
        .map_err(|error| error.to_string())
}

pub async fn insert_assignment_schedule(
    device_id: DeviceId,
    assignment_id: AssignmentId,
    time_template_id: Option<TemplateId>,
    window_start_at: Option<String>,
    dispatch_id: DispatchId,
    status: AssignmentScheduleStatus,
    trigger_source: AssignmentTriggerSource,
    message: Option<String>,
) -> Result<AssignmentSchedule, String> {
    let record = AssignmentSchedule {
        id: AssignmentScheduleId::new_v7(),
        device_id,
        assignment_id,
        time_template_id,
        window_start_at,
        dispatch_id,
        status: assignment_schedule_status_value(&status).to_string(),
        trigger_source: assignment_trigger_source_value(&trigger_source).to_string(),
        started_at: None,
        completed_at: None,
        message,
    };

    sqlx::query(&format!(
        "INSERT INTO {} (id, device_id, assignment_id, time_template_id, window_start_at, dispatch_id, status, trigger_source, started_at, completed_at, message)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(record.id.to_string())
    .bind(record.device_id.to_string())
    .bind(record.assignment_id.to_string())
    .bind(record.time_template_id.map(|value| value.to_string()))
    .bind(record.window_start_at.clone())
    .bind(record.dispatch_id.to_string())
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
