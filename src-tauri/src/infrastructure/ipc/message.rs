// IPC消息定义模块
// 精简后的进程间通信消息类型和结构

use std::path::PathBuf;
use bincode::{Decode, Encode};
use crate::infrastructure::core::{Deserialize, DeviceId, MessageId, PolicyGroupId, PolicySetId, ScriptId, Serialize, TaskId};
use crate::infrastructure::logging::LogLevel;

/// IPC消息
#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
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
            id: MessageId::new_v7(),
            source_or_target,
            message_type,
            payload,
        }
    }

    /// 创建响应消息
    pub fn create_response(&self, source: DeviceId, response_payload: MessagePayload) -> Self {
        Self::new(source, MessageType::Response, response_payload)
    }
}

/// 消息类型
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Deserialize, Hash)]
pub enum MessageType {
    /// 命令（主进程 → 子进程）
    Command,
    /// 响应（子进程 → 主进程，回复 Command）
    Response,
    /// 心跳（双向）
    Heartbeat,
    /// 日志（子进程 → 主进程）
    Logger,
    /// 状态报告（子进程 → 主进程）
    Status,
    /// 错误报告（子进程 → 主进程）
    Error,
}

/// 消息载荷
#[derive(Debug, Clone, PartialEq, Encode, Decode, Deserialize)]
pub enum MessagePayload {
    /// Socket 注册（子进程连接时，携带 PID）
    SocketRegistration(u32),

    /// 进程控制
    ProcessControl(ProcessControlMessage),

    /// 脚本任务管理
    ScriptTask(ScriptTaskMessage),

    /// 配置更新
    ConfigUpdate(ConfigUpdateMessage),

    /// 状态报告
    StatusReport(StatusReportMessage),

    /// 日志
    Logger(LogMessage),

    /// 心跳
    Heartbeat(HeartbeatMessage),

    /// 错误
    Error(ErrorMessage),

    /// 空消息（ACK）
    Empty,
}

// ========== 进程控制 ==========

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ProcessControlMessage {
    pub action: ProcessAction,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ProcessAction {
    /// 开始执行调度（恢复运行）
    Start,
    /// 停止当前执行，进入 Idle
    Stop,
    /// 暂停调度（可恢复）
    Pause,
    /// 关闭子进程
    Shutdown,
}

// ========== 脚本任务管理 ==========

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ScriptTaskMessage {
    pub action: ScriptTaskAction,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ScriptTaskAction {
    /// 添加脚本到执行队列
    Add { script_id: ScriptId },
    /// 从执行队列移除脚本
    Remove { script_id: ScriptId },
    /// 执行指定目标（开发者调试用）
    Execute {
        script_id: ScriptId,
        target: ExecuteTarget,
    },
}

/// 执行目标（开发者调试）
#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ExecuteTarget {
    /// 执行整个脚本
    FullScript,
    /// 仅执行指定任务
    Task(TaskId),
    /// 仅执行指定策略组
    PolicyGroup(PolicyGroupId),
    /// 仅执行指定策略集
    PolicySet(PolicySetId),
}

// ========== 配置更新 ==========

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ConfigUpdateMessage {
    pub update: ConfigUpdateType,
}

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub enum ConfigUpdateType {
    /// 更新日志级别
    LogLevel(LogLevel),
    /// 更新日志写入文件开关
    LogToFile(bool),
}

// ========== 状态报告 ==========

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub struct StatusReportMessage {
    pub status: ChildProcessStatus,
    pub current_script: Option<ScriptId>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum ChildProcessStatus {
    /// 初始化中
    Initializing,
    /// 空闲等待任务
    Idle,
    /// 正在执行脚本
    Running,
    /// 已暂停
    Paused,
    /// 正在停止
    Stopping,
    /// 错误状态
    Error,
}

// ========== 日志 ==========

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
}

// ========== 心跳 ==========

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct HeartbeatMessage {
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

// ========== 错误 ==========

#[derive(Debug, Clone, Encode, Decode, Deserialize, PartialEq)]
pub struct ErrorMessage {
    pub code: u32,
    pub message: String,
    pub details: Option<String>,
}
