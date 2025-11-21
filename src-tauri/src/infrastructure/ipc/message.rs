// IPC消息定义模块
// 定义进程间通信的消息类型和结构

use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::core::{DeviceId, HashMap, ScriptId};
use crate::infrastructure::logging::LogLevel;
use bincode::{Decode, Encode};
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

/// 消息唯一标识符（使用UUID v7，便于时间排序和调试）
pub type MessageId = Uuid;

/// IPC消息枚举
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct IpcMessage {
    pub id: MessageId,
    pub source_or_target: DeviceId,
    pub message_type: MessageType,
    pub payload: MessagePayload,
}

impl IpcMessage {
    /// 创建新消息
    pub fn new(
        source_or_target: DeviceId,
        message_type: MessageType,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            source_or_target,
            message_type,
            payload,
        }
    }

    /// 创建响应消息
    pub fn create_response(&self, source: DeviceId, response_payload: MessagePayload) -> Self {
        Self::new(source, MessageType::Response, response_payload)
    }

    pub fn set_heart_payload(mut self, msg_payload: MessagePayload) {
        self.payload = msg_payload;
    }
}

/// 消息类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Hash)]
pub enum MessageType {
    // 控制消息
    Request,  // 请求
    Response, // 响应
    Command,  // 命令
    Event,    // 事件

    // 系统消息
    Heartbeat, // 心跳
    Status,    // 状态报告
    Error,     // 错误报告

    // 数据消息
    Data,   // 数据传输
    Stream, // 流数据

    // 通知消息
    Notification, // 通知
    Broadcast,    // 广播

    // 日志信息
    Logger,
}

/// 消息载荷枚举
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum MessagePayload {
    // socket注册
    SocketRegistration(u32),

    // 进程控制消息
    ProcessControl(ProcessControlMessage),

    // 脚本管理消息
    ScriptManagement(ScriptManagementMessage),

    // 状态同步消息
    StateSync(StateSyncMessage),

    // 日志消息
    Logger(LogMessage),

    // 心跳消息
    Heartbeat(HeartbeatMessage),

    // 性能监控消息
    Performance(PerformanceMessage),

    // 资源管理消息
    Resource(ResourceMessage),

    // 配置更新消息
    Config(ConfigMessage),

    // 视觉处理消息
    Vision(VisionMessage),

    // 错误消息
    Error(ErrorMessage),

    // 空消息（用于ACK等）
    Empty,
}
/// 进程控制消息
#[derive(Debug, Encode, Clone, Decode, PartialEq)]
pub struct ProcessControlMessage {
    pub action: ProcessAction,
    pub process_id: DeviceId,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Encode, Clone, Decode, PartialEq)]
pub enum ProcessAction {
    Start,
    Stop,
    Restart,
    Pause,
    Resume,
    Kill,
    GetStatus,
    UpdateConfig,
}

/// 脚本管理消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct ScriptManagementMessage {
    pub action: ScriptAction,
    pub script_id: ScriptId,
    pub script_content: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum ScriptAction {
    Load(PathBuf),
    Unload,
    Execute,
    Stop,
    GetStatus,
    UpdateParameters,
}

/// 状态同步消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct StateSyncMessage {
    pub state_type: StateType,
    pub data: HashMap<String, serde_json::Value>,
    pub version: u64,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum StateType {
    ProcessState,
    DeviceState,
    SystemState,
    ConfigState,
    ModelState,
    SocketState,
}
/// 日志消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
}

/// 心跳消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct HeartbeatMessage {
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

/// 性能监控消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct PerformanceMessage {
    pub metrics_type: MetricsType,
    pub data: HashMap<String, f64>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum MetricsType {
    CpuMetrics,
    MemoryMetrics,
    InferenceMetrics,
    NetworkMetrics,
}

/// 资源管理消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct ResourceMessage {
    pub resource_type: ResourceType,
    pub action: ResourceAction,
    //pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum ResourceType {
    CpuCores,
    Memory,
    ThreadPool,
    Model,
    File,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum ResourceAction {
    Allocate,
    Deallocate,
    Update,
    Query,
    Lock,
    Unlock,
}

/// 配置更新消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct ConfigMessage {
    pub config_type: String,
    pub action: ConfigAction,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum ConfigAction {
    Get,
    Set,
    Update,
    Reset,
    Validate,
}

/// 视觉处理消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct VisionMessage {
    pub vision_type: VisionType,
    pub processing_time_ms: f64,
}
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum VisionType {
    ObjectDetection(Vec<DetResult>),
    TextDetection(Vec<DetResult>),
    TextRecognition(Vec<OcrResult>),
    TemplateMatching,
    ColorDetection,
    Combined,
}

/// 错误消息
#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct ErrorMessage {
    pub error_type: ErrorType,
    pub code: u32,
    pub message: String,
    pub details: Option<String>,
    pub stack_trace: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum ErrorType {
    ProcessError,
    ConfigError,
    ResourceError,
    CommunicationError,
    VisionError,
    SystemError,
    Unknown,
}
