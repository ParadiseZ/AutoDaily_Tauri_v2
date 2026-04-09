use crate::constant::table_name::SCHEDULE_TABLE;
use crate::domain::devices::device_schedule::DeviceScriptSchedule;
use crate::domain::devices::device_schedule::RunStatus;
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::infrastructure::core::{DeviceId, ExecutionId, ScheduleId, ScriptId};
use crate::infrastructure::db::get_pool;

pub struct ScheduleJournal;

impl ScheduleJournal {
    fn task_cycle_value(task: &ScriptTaskTable) -> Result<String, String> {
        let json =
            serde_json::to_value(&task.default_task_cycle.0).map_err(|error| error.to_string())?;
        Ok(match json {
            serde_json::Value::String(value) => value,
            value => value.to_string(),
        })
    }

    pub async fn append_task_record(
        device_id: DeviceId,
        execution_id: ExecutionId,
        assignment_id: ScheduleId,
        script_id: ScriptId,
        task: &ScriptTaskTable,
        status: RunStatus,
        started_at: String,
        completed_at: Option<String>,
        message: Option<String>,
    ) -> Result<DeviceScriptSchedule, String> {
        let record = DeviceScriptSchedule {
            id: ScheduleId::new_v7(),
            device_id,
            execution_id: Some(execution_id),
            assignment_id: Some(assignment_id),
            script_id,
            task_id: task.id,
            task_cycle: Self::task_cycle_value(task)?,
            status: format!("{:?}", status),
            started_at,
            completed_at,
            message,
        };

        sqlx::query(&format!(
            "INSERT INTO {} (id, device_id, execution_id, assignment_id, script_id, task_id, task_cycle, status, started_at, completed_at, message)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            SCHEDULE_TABLE
        ))
        .bind(record.id.to_string())
        .bind(record.device_id.to_string())
        .bind(record.execution_id.map(|id| id.to_string()))
        .bind(record.assignment_id.map(|id| id.to_string()))
        .bind(record.script_id.to_string())
        .bind(record.task_id.to_string())
        .bind(&record.task_cycle)
        .bind(&record.status)
        .bind(&record.started_at)
        .bind(&record.completed_at)
        .bind(&record.message)
        .execute(get_pool())
        .await
        .map_err(|error| error.to_string())?;

        Ok(record)
    }
}
