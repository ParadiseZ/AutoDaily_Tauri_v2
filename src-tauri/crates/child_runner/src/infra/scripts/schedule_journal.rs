use ad_kernel::ids::{AssignmentId, DeviceId, ExecutionId, ScheduleId, ScriptId, TaskId};
use domain_schedule::{ExecutionScheduleProfile, TaskRunStatus};
use domain_script::{ScriptTaskProfile, TaskCycle};
use infra_sqlite::{find_latest_success_schedule, insert_execution_schedule};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) struct ScheduleJournal;
#[derive(Serialize)]
struct TaskDedupScope<'a> {
    base_scope_hash: &'a str,
    task_id: String,
}

impl ScheduleJournal {
    pub(crate) fn compute_dedup_scope_hash(
        base_scope_hash: &str,
        task_id: TaskId,
    ) -> Result<String, String> {
        let json = serde_json::to_vec(&TaskDedupScope {
            base_scope_hash,
            task_id: task_id.to_string(),
        })
        .map_err(|error| error.to_string())?;
        Ok(format!("{:x}", Sha256::digest(json)))
    }
    fn task_cycle_value(task_cycle: &TaskCycle) -> Result<String, String> {
        serde_json::to_value(task_cycle)
            .map_err(|error| error.to_string())
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string())
            })
    }
    pub(crate) async fn append_task_record(
        device_id: DeviceId,
        execution_id: ExecutionId,
        assignment_id: AssignmentId,
        script_id: ScriptId,
        task: &ScriptTaskProfile,
        dedup_scope_hash: &str,
        task_cycle: &TaskCycle,
        status: TaskRunStatus,
        started_at: String,
        completed_at: Option<String>,
        message: Option<String>,
    ) -> Result<ExecutionScheduleProfile, String> {
        let record = ExecutionScheduleProfile {
            id: ScheduleId::new_v7(),
            device_id,
            execution_id: Some(execution_id),
            assignment_id: Some(assignment_id),
            script_id,
            task_id: task.id,
            dedup_scope_hash: dedup_scope_hash.to_string(),
            task_cycle: Self::task_cycle_value(task_cycle)?,
            status: format!("{:?}", status),
            started_at,
            completed_at,
            message,
        };
        insert_execution_schedule(&record).await?;
        Ok(record)
    }
    pub(crate) async fn load_latest_success_record(
        assignment_id: AssignmentId,
        dedup_scope_hash: &str,
        task_id: TaskId,
    ) -> Result<Option<ExecutionScheduleProfile>, String> {
        find_latest_success_schedule(assignment_id, dedup_scope_hash, task_id).await
    }
}
