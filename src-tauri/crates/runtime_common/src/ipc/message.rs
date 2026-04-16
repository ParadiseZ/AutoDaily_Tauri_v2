use bincode::{Decode, Encode};

use crate::core::{
    AccountId, Deserialize, DeviceId, ExecutionId, MessageId, PolicyGroupId, PolicyId,
    PolicySetId, ScheduleId, ScriptId, Serialize, SessionId, StepId, TaskId, TemplateId,
};
use crate::logging::LogLevel;

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct IpcMessage {
    pub id: MessageId,
    pub source_or_target: DeviceId,
    pub message_type: MessageType,
    pub payload: MessagePayload,
}

impl IpcMessage {
    pub fn new(
        source_or_target: DeviceId,
        message_type: MessageType,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: MessageId::new_v7(),
            source_or_target,
            message_type,
            payload,
        }
    }

    pub fn create_response(&self, source: DeviceId, response_payload: MessagePayload) -> Self {
        Self::new(source, MessageType::Response, response_payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Deserialize, Hash)]
pub enum MessageType {
    Command,
    Response,
    Heartbeat,
    Logger,
    Status,
    Error,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub enum MessagePayload {
    SocketRegistration(u32),
    ProcessControl(ProcessControlMessage),
    SessionControl(SessionControlMessage),
    RuntimeEvent(RuntimeEventMessage),
    ConfigUpdate(ConfigUpdateMessage),
    Logger(LogMessage),
    Heartbeat(HeartbeatMessage),
    Error(ErrorMessage),
    Empty,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ProcessControlMessage {
    pub action: ProcessAction,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ProcessAction {
    Start,
    Stop,
    Pause,
    Shutdown,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RunTarget {
    DeviceQueue,
    FullScript {
        script_id: ScriptId,
    },
    Task {
        script_id: ScriptId,
        task_id: TaskId,
    },
    PolicyGroup {
        script_id: ScriptId,
        policy_group_id: PolicyGroupId,
    },
    PolicySet {
        script_id: ScriptId,
        policy_set_id: PolicySetId,
    },
    Policy {
        script_id: ScriptId,
        policy_id: PolicyId,
    },
}

impl RunTarget {
    pub fn script_id(&self) -> Option<ScriptId> {
        match self {
            Self::DeviceQueue => None,
            Self::FullScript { script_id }
            | Self::Task { script_id, .. }
            | Self::PolicyGroup { script_id, .. }
            | Self::PolicySet { script_id, .. }
            | Self::Policy { script_id, .. } => Some(*script_id),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVisionTextCachePolicy {
    pub enabled: bool,
    pub dir: Option<String>,
    pub signature_grid_size: u16,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutAction {
    NotifyOnly,
    PauseExecution,
    StopExecution,
    RestartApp,
    RunRecoveryTask,
    SkipCurrentTask,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutNotifyChannel {
    SystemNotification,
    Email,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExecutionPolicy {
    pub ocr_text_cache: RuntimeVisionTextCachePolicy,
    pub action_wait_ms: u64,
    pub progress_timeout_enabled: bool,
    pub progress_timeout_ms: u64,
    pub timeout_action: TimeoutAction,
    pub timeout_notify_channels: Vec<TimeoutNotifyChannel>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeQueueItem {
    pub assignment_id: ScheduleId,
    pub script_id: ScriptId,
    pub time_template_id: Option<TemplateId>,
    pub account_id: Option<AccountId>,
    pub account_data_json: Option<String>,
    pub order_index: u32,
    pub template_values_json: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptBundleSnapshot {
    pub script_id: ScriptId,
    pub script_json: String,
    pub tasks_json: String,
    pub policies_json: String,
    pub policy_groups_json: String,
    pub policy_sets_json: String,
    pub group_policies_json: String,
    pub set_groups_json: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSessionSnapshot {
    pub session_id: SessionId,
    pub device_id: DeviceId,
    pub run_target: RunTarget,
    pub runtime_policy: RuntimeExecutionPolicy,
    pub queue: Vec<RuntimeQueueItem>,
    pub script_bundles: Vec<ScriptBundleSnapshot>,
    pub issued_at: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResumeMode {
    FromTaskStart,
    FromStepStart,
    FromNextStep,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResumeCheckpoint {
    pub execution_id: ExecutionId,
    pub source_session_id: SessionId,
    pub device_id: DeviceId,
    pub run_target: RunTarget,
    pub assignment_id: Option<ScheduleId>,
    pub script_id: ScriptId,
    pub time_template_id: Option<TemplateId>,
    pub account_id: Option<AccountId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub resume_mode: ResumeMode,
    pub definition_fingerprint: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionCheckpointReason {
    Restart,
    Shutdown,
    Manual,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeRecoveryPhase {
    CheckpointPreparing,
    CheckpointReady,
    RestartReady,
    CheckpointLoaded,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionControlMessage {
    LoadSession {
        session: RuntimeSessionSnapshot,
        checkpoint: Option<ResumeCheckpoint>,
    },
    ReloadSession {
        session: RuntimeSessionSnapshot,
        checkpoint: Option<ResumeCheckpoint>,
    },
    PrepareCheckpoint {
        reason: SessionCheckpointReason,
    },
    ClearSession,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeLifecyclePhase {
    Initializing,
    Loaded,
    Idle,
    Running,
    Paused,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLifecycleEvent {
    pub session_id: Option<SessionId>,
    pub phase: RuntimeLifecyclePhase,
    pub current_script_id: Option<ScriptId>,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeProgressPhase {
    Idle,
    Loading,
    Planning,
    Executing,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeProgressEvent {
    pub session_id: Option<SessionId>,
    pub assignment_id: Option<ScheduleId>,
    pub script_id: Option<ScriptId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub phase: RuntimeProgressPhase,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeScheduleStatus {
    Queued,
    Running,
    Success,
    Failed,
    Skipped,
    Cleared,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeScheduleEvent {
    pub session_id: Option<SessionId>,
    pub execution_id: Option<ExecutionId>,
    pub assignment_id: Option<ScheduleId>,
    pub script_id: Option<ScriptId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub status: RuntimeScheduleStatus,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRecoveryEvent {
    pub session_id: Option<SessionId>,
    pub execution_id: Option<ExecutionId>,
    pub assignment_id: Option<ScheduleId>,
    pub script_id: Option<ScriptId>,
    pub task_id: Option<TaskId>,
    pub step_id: Option<StepId>,
    pub phase: RuntimeRecoveryPhase,
    pub checkpoint_updated_at: Option<String>,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeEventMessage {
    Lifecycle(RuntimeLifecycleEvent),
    Progress(RuntimeProgressEvent),
    Schedule(RuntimeScheduleEvent),
    Recovery(RuntimeRecoveryEvent),
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ConfigUpdateMessage {
    pub update: ConfigUpdateType,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ConfigUpdateType {
    LogLevel(LogLevel),
    LogToFile(bool),
    AdbPath(Option<String>),
    AdbServerAddr(Option<String>),
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct HeartbeatMessage {
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ErrorMessage {
    pub code: u32,
    pub message: String,
    pub details: Option<String>,
}
