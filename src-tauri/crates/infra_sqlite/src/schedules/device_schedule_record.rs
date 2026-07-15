use ad_kernel::ids::{AssignmentId, DeviceId, ScriptId, TaskId, UuidV7};
use domain_schedule::{AssignmentProfile, AssignmentScheduleProfile, ExecutionScheduleProfile};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

fn id(value: String) -> Result<UuidV7, String> {
    Uuid::parse_str(&value)
        .map(Into::into)
        .map_err(|error| error.to_string())
}

#[derive(FromRow)]
struct AssignmentRow {
    id: String,
    device_id: String,
    script_id: String,
    time_template_id: Option<String>,
    account_data: Json<serde_json::Value>,
    index: i64,
}
#[derive(FromRow)]
struct ExecutionScheduleRow {
    id: String,
    device_id: String,
    execution_id: Option<String>,
    assignment_id: Option<String>,
    script_id: String,
    task_id: String,
    dedup_scope_hash: String,
    task_cycle: String,
    status: String,
    started_at: String,
    completed_at: Option<String>,
    message: Option<String>,
}
#[derive(FromRow)]
struct AssignmentScheduleRow {
    id: String,
    batch_id: String,
    device_id: String,
    assignment_id: Option<String>,
    script_id: Option<String>,
    time_template_id: Option<String>,
    window_start_at: Option<String>,
    scope_hash: String,
    dispatch_id: String,
    order_index: i64,
    created_at: String,
    run_target_json: Option<String>,
    status: String,
    trigger_source: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    message: Option<String>,
}
#[derive(FromRow)]
struct PlannerBatchRow {
    batch_id: String,
    _latest_created_at: String,
    live_count: i64,
}

impl TryFrom<AssignmentRow> for AssignmentProfile {
    type Error = String;
    fn try_from(row: AssignmentRow) -> Result<Self, String> {
        Ok(Self {
            id: id(row.id)?,
            device_id: id(row.device_id)?,
            script_id: id(row.script_id)?,
            time_template_id: row.time_template_id.map(id).transpose()?,
            account_data: row.account_data.0,
            index: row
                .index
                .try_into()
                .map_err(|_| "分配排序值无效".to_string())?,
        })
    }
}
impl TryFrom<ExecutionScheduleRow> for ExecutionScheduleProfile {
    type Error = String;
    fn try_from(row: ExecutionScheduleRow) -> Result<Self, String> {
        Ok(Self {
            id: id(row.id)?,
            device_id: id(row.device_id)?,
            execution_id: row.execution_id.map(id).transpose()?,
            assignment_id: row.assignment_id.map(id).transpose()?,
            script_id: id(row.script_id)?,
            task_id: id(row.task_id)?,
            dedup_scope_hash: row.dedup_scope_hash,
            task_cycle: row.task_cycle,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            message: row.message,
        })
    }
}
impl TryFrom<AssignmentScheduleRow> for AssignmentScheduleProfile {
    type Error = String;
    fn try_from(row: AssignmentScheduleRow) -> Result<Self, String> {
        Ok(Self {
            id: id(row.id)?,
            batch_id: id(row.batch_id)?,
            device_id: id(row.device_id)?,
            assignment_id: row.assignment_id.map(id).transpose()?,
            script_id: row.script_id.map(id).transpose()?,
            time_template_id: row.time_template_id.map(id).transpose()?,
            window_start_at: row.window_start_at,
            scope_hash: row.scope_hash,
            dispatch_id: id(row.dispatch_id)?,
            order_index: row
                .order_index
                .try_into()
                .map_err(|_| "调度排序值无效".to_string())?,
            created_at: row.created_at,
            run_target_json: row.run_target_json,
            status: row.status,
            trigger_source: row.trigger_source,
            started_at: row.started_at,
            completed_at: row.completed_at,
            message: row.message,
        })
    }
}

pub async fn list_assignments(device_id: DeviceId) -> Result<Vec<AssignmentProfile>, String> {
    sqlx::query_as::<_, AssignmentRow>("SELECT id, device_id, script_id, time_template_id, account_data, `index` FROM device_script_assignments WHERE device_id = ? ORDER BY `index` ASC, id ASC").bind(device_id.to_string()).fetch_all(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?.into_iter().map(TryInto::try_into).collect()
}
pub async fn list_execution_schedules(
    device_id: DeviceId,
) -> Result<Vec<ExecutionScheduleProfile>, String> {
    sqlx::query_as::<_, ExecutionScheduleRow>("SELECT id, device_id, execution_id, assignment_id, script_id, task_id, dedup_scope_hash, task_cycle, status, started_at, completed_at, message FROM device_script_schedules WHERE device_id = ? ORDER BY started_at DESC").bind(device_id.to_string()).fetch_all(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?.into_iter().map(TryInto::try_into).collect()
}

pub async fn insert_execution_schedule(profile: &ExecutionScheduleProfile) -> Result<(), String> {
    sqlx::query("INSERT INTO device_script_schedules (id, device_id, execution_id, assignment_id, script_id, task_id, dedup_scope_hash, task_cycle, status, started_at, completed_at, message) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(profile.id.to_string()).bind(profile.device_id.to_string()).bind(profile.execution_id.map(|id| id.to_string())).bind(profile.assignment_id.map(|id| id.to_string())).bind(profile.script_id.to_string()).bind(profile.task_id.to_string()).bind(&profile.dedup_scope_hash).bind(&profile.task_cycle).bind(&profile.status).bind(&profile.started_at).bind(&profile.completed_at).bind(&profile.message)
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn find_latest_success_schedule(
    assignment_id: AssignmentId,
    dedup_scope_hash: &str,
    task_id: TaskId,
) -> Result<Option<ExecutionScheduleProfile>, String> {
    sqlx::query_as::<_, ExecutionScheduleRow>("SELECT id, device_id, execution_id, assignment_id, script_id, task_id, dedup_scope_hash, task_cycle, status, started_at, completed_at, message FROM device_script_schedules WHERE assignment_id = ? AND dedup_scope_hash = ? AND task_id = ? AND status = ? ORDER BY COALESCE(completed_at, started_at) DESC, started_at DESC LIMIT 1")
        .bind(assignment_id.to_string()).bind(dedup_scope_hash).bind(task_id.to_string()).bind("Success")
        .fetch_optional(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?
        .map(TryInto::try_into).transpose()
}

pub(crate) async fn query_assignment_schedule_profiles(
    query: &str,
    params: &[Option<String>],
) -> Result<Vec<AssignmentScheduleProfile>, String> {
    let mut statement = sqlx::query_as::<_, AssignmentScheduleRow>(query);
    for param in params {
        statement = statement.bind(param);
    }
    statement
        .fetch_all(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

pub(crate) async fn query_assignment_schedule_profile(
    query: &str,
    params: &[Option<String>],
) -> Result<Option<AssignmentScheduleProfile>, String> {
    let mut statement = sqlx::query_as::<_, AssignmentScheduleRow>(query);
    for param in params {
        statement = statement.bind(param);
    }
    statement
        .fetch_optional(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .map(TryInto::try_into)
        .transpose()
}

pub(crate) async fn list_planner_batch_ids(
    device_id: DeviceId,
    day_prefix: &str,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query_as::<_, PlannerBatchRow>("SELECT batch_id, MAX(created_at) AS _latest_created_at, SUM(CASE WHEN status IN ('planned', 'dispatched', 'running', 'stopped') THEN 1 ELSE 0 END) AS live_count FROM assignment_schedules WHERE device_id = ? AND trigger_source = 'planner' AND created_at LIKE ? GROUP BY batch_id ORDER BY _latest_created_at DESC")
        .bind(device_id.to_string()).bind(format!("{}%", day_prefix)).fetch_all(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let active = rows
        .iter()
        .filter(|row| row.live_count > 0)
        .map(|row| row.batch_id.clone())
        .collect::<Vec<_>>();
    Ok(if active.is_empty() {
        vec![rows[0].batch_id.clone()]
    } else {
        active
    })
}

pub async fn has_active_assignment_schedules(device_id: DeviceId) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM assignment_schedules WHERE device_id = ? AND status IN ('planned', 'dispatched', 'running')").bind(device_id.to_string()).fetch_one(crate::bootstrap::get_pool()).await.map(|count| count > 0).map_err(|error| error.to_string())
}
pub async fn list_assigned_device_ids_by_script(
    script_id: ScriptId,
) -> Result<Vec<DeviceId>, String> {
    list_assigned_device_ids("script_id", script_id.to_string()).await
}

pub async fn list_assigned_device_ids_by_time_template(
    template_id: &str,
) -> Result<Vec<DeviceId>, String> {
    list_assigned_device_ids("time_template_id", template_id.to_string()).await
}

async fn list_assigned_device_ids(column: &str, value: String) -> Result<Vec<DeviceId>, String> {
    sqlx::query_scalar::<_, String>(&format!(
        "SELECT DISTINCT device_id FROM device_script_assignments WHERE {column} = ? ORDER BY device_id ASC"
    ))
    .bind(value)
    .fetch_all(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?
    .into_iter()
    .map(id)
    .collect()
}
pub async fn delete_assignment(assignment_id: AssignmentId) -> Result<Option<DeviceId>, String> {
    let device_id = sqlx::query_scalar::<_, String>(
        "SELECT device_id FROM device_script_assignments WHERE id = ?",
    )
    .bind(assignment_id.to_string())
    .fetch_optional(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM device_script_assignments WHERE id = ?")
        .bind(assignment_id.to_string())
        .execute(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?;
    device_id.map(id).transpose()
}
pub async fn clear_schedules_by_device(device_id: DeviceId) -> Result<(), String> {
    clear_schedules("device_id", device_id.to_string()).await
}
pub async fn clear_schedules_by_script(script_id: ScriptId) -> Result<(), String> {
    clear_schedules("script_id", script_id.to_string()).await
}
async fn clear_schedules(column: &str, value: String) -> Result<(), String> {
    for table in ["assignment_schedules", "device_script_schedules"] {
        sqlx::query(&format!("DELETE FROM {table} WHERE {column} = ?"))
            .bind(&value)
            .execute(crate::bootstrap::get_pool())
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
