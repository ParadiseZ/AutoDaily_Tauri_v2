use bincode::{Decode, Encode};

use crate::core::{
    Deserialize, DeviceId, MessageId, PolicyGroupId, PolicySetId, ScriptId, Serialize, TaskId,
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
    ScriptTask(ScriptTaskMessage),
    ConfigUpdate(ConfigUpdateMessage),
    StatusReport(StatusReportMessage),
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

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ScriptTaskMessage {
    pub action: ScriptTaskAction,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ScriptTaskAction {
    Add { script_id: ScriptId },
    Remove { script_id: ScriptId },
    Execute { script_id: ScriptId, target: ExecuteTarget },
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ExecuteTarget {
    FullScript,
    Task(TaskId),
    PolicyGroup(PolicyGroupId),
    PolicySet(PolicySetId),
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

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub struct StatusReportMessage {
    pub status: ChildProcessStatus,
    pub current_script: Option<ScriptId>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum ChildProcessStatus {
    Initializing,
    Idle,
    Running,
    Paused,
    Stopping,
    Error,
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
