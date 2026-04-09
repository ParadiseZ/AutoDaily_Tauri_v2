use crate::infrastructure::context::child_process_sec::get_ipc_client;
use crate::infrastructure::core::{ExecutionId, ScheduleId, ScriptId, SessionId, StepId, TaskId};
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, RuntimeEventMessage, RuntimeLifecycleEvent,
    RuntimeLifecyclePhase, RuntimeProgressEvent, RuntimeProgressPhase, RuntimeRecoveryEvent,
    RuntimeRecoveryPhase, RuntimeScheduleEvent, RuntimeScheduleStatus,
};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::scheduler::get_scheduler;
use crate::infrastructure::session::runtime_session::try_current_session_summary;

fn now_millis_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn current_session_id() -> Option<SessionId> {
    try_current_session_summary().map(|summary| summary.session_id)
}

fn current_script_id() -> Option<ScriptId> {
    get_scheduler().and_then(|scheduler| scheduler.current_script_snapshot())
}

fn emit_runtime_event(event: RuntimeEventMessage, log_label: &str) {
    if let Some(client) = get_ipc_client() {
        let log_label = log_label.to_string();
        tokio::spawn(async move {
            let msg = IpcMessage::new(
                *client.device_id,
                MessageType::Status,
                MessagePayload::RuntimeEvent(event),
            );
            if let Err(error) = client.send_ensure(msg).await {
                Log::warn(&format!("[ child ] 发送{}失败: {}", log_label, error));
            }
        });
    }
}

pub fn emit_lifecycle_event(
    phase: RuntimeLifecyclePhase,
    message: Option<String>,
) {
    emit_lifecycle_event_with(phase, current_session_id(), current_script_id(), message);
}

pub fn emit_lifecycle_event_with(
    phase: RuntimeLifecyclePhase,
    session_id: Option<SessionId>,
    current_script_id: Option<ScriptId>,
    message: Option<String>,
) {
    emit_runtime_event(
        RuntimeEventMessage::Lifecycle(RuntimeLifecycleEvent {
            session_id,
            phase,
            current_script_id,
            message,
            at: now_millis_string(),
        }),
        "生命周期事件",
    );
}

pub fn emit_progress_event(
    phase: RuntimeProgressPhase,
    assignment_id: Option<ScheduleId>,
    script_id: Option<ScriptId>,
    task_id: Option<TaskId>,
    step_id: Option<StepId>,
    message: Option<String>,
) {
    emit_runtime_event(
        RuntimeEventMessage::Progress(RuntimeProgressEvent {
            session_id: current_session_id(),
            assignment_id,
            script_id,
            task_id,
            step_id,
            phase,
            message,
            at: now_millis_string(),
        }),
        "进度事件",
    );
}

pub fn emit_schedule_event(
    status: RuntimeScheduleStatus,
    execution_id: Option<ExecutionId>,
    assignment_id: Option<ScheduleId>,
    script_id: Option<ScriptId>,
    task_id: Option<TaskId>,
    step_id: Option<StepId>,
    message: Option<String>,
) {
    emit_runtime_event(
        RuntimeEventMessage::Schedule(RuntimeScheduleEvent {
            session_id: current_session_id(),
            execution_id,
            assignment_id,
            script_id,
            task_id,
            step_id,
            status,
            message,
            at: now_millis_string(),
        }),
        "调度事件",
    );
}

pub fn emit_recovery_event(
    phase: RuntimeRecoveryPhase,
    execution_id: Option<ExecutionId>,
    assignment_id: Option<ScheduleId>,
    script_id: Option<ScriptId>,
    task_id: Option<TaskId>,
    step_id: Option<StepId>,
    checkpoint_updated_at: Option<String>,
    message: Option<String>,
) {
    emit_runtime_event(
        RuntimeEventMessage::Recovery(RuntimeRecoveryEvent {
            session_id: current_session_id(),
            execution_id,
            assignment_id,
            script_id,
            task_id,
            step_id,
            phase,
            checkpoint_updated_at,
            message,
            at: now_millis_string(),
        }),
        "恢复事件",
    );
}
