use crate::infrastructure::core::{
    AccountId, DeviceId, ExecutionId, ScheduleId, ScriptId, SessionId, StepId, TaskId, TemplateId,
};
use crate::infrastructure::ipc::message::{ResumeCheckpoint, ResumeMode, RunTarget};
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct RecoveryCheckpointRow {
    pub execution_id: ExecutionId,
    pub source_session_id: SessionId,
    pub device_id: DeviceId,
    pub run_target_json: Json<RunTarget>,
    pub assignment_id: Option<ScheduleId>,
    pub script_id: ScriptId,
    pub time_template_id: Option<TemplateId>,
    pub account_id: Option<AccountId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub resume_mode: Json<ResumeMode>,
    pub definition_fingerprint: String,
    pub updated_at: String,
}

impl RecoveryCheckpointRow {
    pub fn into_checkpoint(self) -> ResumeCheckpoint {
        ResumeCheckpoint {
            execution_id: self.execution_id,
            source_session_id: self.source_session_id,
            device_id: self.device_id,
            run_target: self.run_target_json.0,
            assignment_id: self.assignment_id,
            script_id: self.script_id,
            time_template_id: self.time_template_id,
            account_id: self.account_id,
            task_id: self.task_id,
            step_id: self.step_id,
            resume_mode: self.resume_mode.0,
            definition_fingerprint: self.definition_fingerprint,
            updated_at: self.updated_at,
        }
    }
}

impl From<ResumeCheckpoint> for RecoveryCheckpointRow {
    fn from(checkpoint: ResumeCheckpoint) -> Self {
        Self {
            execution_id: checkpoint.execution_id,
            source_session_id: checkpoint.source_session_id,
            device_id: checkpoint.device_id,
            run_target_json: Json(checkpoint.run_target),
            assignment_id: checkpoint.assignment_id,
            script_id: checkpoint.script_id,
            time_template_id: checkpoint.time_template_id,
            account_id: checkpoint.account_id,
            task_id: checkpoint.task_id,
            step_id: checkpoint.step_id,
            resume_mode: Json(checkpoint.resume_mode),
            definition_fingerprint: checkpoint.definition_fingerprint,
            updated_at: checkpoint.updated_at,
        }
    }
}
