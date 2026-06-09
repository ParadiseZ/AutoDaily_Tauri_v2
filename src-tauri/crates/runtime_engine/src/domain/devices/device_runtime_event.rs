use crate::infrastructure::core::{
    AssignmentId, DeviceId, DispatchId, ExecutionId, ScriptId, SessionId, StepId, TaskId,
};
use runtime_common::ipc::message::{
    ConnectionStatusKind, RuntimeLifecyclePhase, RuntimeProgressPhase, RuntimeScheduleStatus,
    TimeoutAction,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DeviceLifecycleStatus {
    Initializing,
    Loaded,
    Idle,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

impl From<RuntimeLifecyclePhase> for DeviceLifecycleStatus {
    fn from(value: RuntimeLifecyclePhase) -> Self {
        match value {
            RuntimeLifecyclePhase::Initializing => Self::Initializing,
            RuntimeLifecyclePhase::Loaded => Self::Loaded,
            RuntimeLifecyclePhase::Idle => Self::Idle,
            RuntimeLifecyclePhase::Running => Self::Running,
            RuntimeLifecyclePhase::Paused => Self::Paused,
            RuntimeLifecyclePhase::Stopping => Self::Stopping,
            RuntimeLifecyclePhase::Error => Self::Error,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DeviceRuntimeProgressPhase {
    Idle,
    Loading,
    Planning,
    ChildRuntimeStarting,
    ChildIpcWaiting,
    ChildIpcReady,
    DeviceChecking,
    ShellProbeChecking,
    EmulatorStarting,
    EmulatorWaiting,
    DeviceConnected,
    DeviceDisconnected,
    Executing,
    Paused,
    Completed,
    Failed,
    ChildProcessExited,
    ChildProcessCrashed,
}

impl From<RuntimeProgressPhase> for DeviceRuntimeProgressPhase {
    fn from(value: RuntimeProgressPhase) -> Self {
        match value {
            RuntimeProgressPhase::Idle => Self::Idle,
            RuntimeProgressPhase::Loading => Self::Loading,
            RuntimeProgressPhase::Planning => Self::Planning,
            RuntimeProgressPhase::Executing => Self::Executing,
            RuntimeProgressPhase::Paused => Self::Paused,
            RuntimeProgressPhase::Completed => Self::Completed,
            RuntimeProgressPhase::Failed => Self::Failed,
        }
    }
}

impl From<ConnectionStatusKind> for DeviceRuntimeProgressPhase {
    fn from(value: ConnectionStatusKind) -> Self {
        match value {
            ConnectionStatusKind::DeviceUnknown => Self::Idle,
            ConnectionStatusKind::DeviceChecking => Self::DeviceChecking,
            ConnectionStatusKind::ShellProbeChecking => Self::ShellProbeChecking,
            ConnectionStatusKind::EmulatorStarting => Self::EmulatorStarting,
            ConnectionStatusKind::EmulatorWaiting => Self::EmulatorWaiting,
            ConnectionStatusKind::DeviceConnected => Self::DeviceConnected,
            ConnectionStatusKind::DeviceDisconnected => Self::DeviceDisconnected,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceStatusEventPayload {
    pub device_id: DeviceId,
    pub session_id: Option<SessionId>,
    pub status: DeviceLifecycleStatus,
    pub current_script_id: Option<ScriptId>,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceConnectionEventPayload {
    pub device_id: DeviceId,
    pub status: ConnectionStatusKind,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceProgressEventPayload {
    pub device_id: DeviceId,
    pub session_id: Option<SessionId>,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: Option<ScriptId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub phase: DeviceRuntimeProgressPhase,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceScheduleEventPayload {
    pub device_id: DeviceId,
    pub session_id: Option<SessionId>,
    pub execution_id: Option<ExecutionId>,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: Option<ScriptId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub status: RuntimeScheduleStatus,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceTimeoutEventPayload {
    pub device_id: DeviceId,
    pub session_id: Option<SessionId>,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: Option<ScriptId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub timeout_action: Option<TimeoutAction>,
    pub page_fingerprint: Option<String>,
    pub action_signature: Option<String>,
    pub detail: Option<String>,
    pub message: String,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceAssignmentScheduleChangedEventPayload {
    pub device_id: DeviceId,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DeviceRuntimeReconcileJobType {
    DeviceConfig,
    DeviceSessionRefresh,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DeviceRuntimeReconcilePhase {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DeviceRuntimeReconcileAction {
    Spawning,
    Starting,
    Pausing,
    Stopping,
    ShuttingDown,
    Restarting,
    Syncing,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceRuntimeReconcileEventPayload {
    pub job_id: String,
    pub job_type: DeviceRuntimeReconcileJobType,
    pub device_id: DeviceId,
    pub phase: DeviceRuntimeReconcilePhase,
    pub action: Option<DeviceRuntimeReconcileAction>,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeviceDispatchEventPayload {
    pub device_id: DeviceId,
    pub dispatch_id: Option<DispatchId>,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: Option<ScriptId>,
    pub at: String,
}
