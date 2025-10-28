// 进程句柄管理模块
// 定义进程句柄、状态和优先级管理

use crate::infrastructure::core::{ProcessId, DeviceId, Error, Serialize, Deserialize};
use std::time::{SystemTime, Duration};
use std::process::{Child, Command};
use std::fmt;

/// 进程管理错误类型
#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("进程启动失败: {reason}")]
    StartupError { reason: String },
    
    #[error("进程未找到: {process_id}")]
    ProcessNotFound { process_id: ProcessId },
    
    #[error("进程已存在: {process_id}")]
    ProcessAlreadyExists { process_id: ProcessId },
    
    #[error("进程终止失败: {reason}")]
    TerminationError { reason: String },
    
    #[error("进程通信错误: {message}")]
    CommunicationError { message: String },
    
    #[error("权限不足: {operation}")]
    PermissionError { operation: String },
    
    #[error("资源不足: {resource}")]
    ResourceError { resource: String },
}

pub type ProcessResult<T> = Result<T, ProcessError>;

/// 进程状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Starting,       // 启动中
    Running,        // 运行中
    Paused,         // 暂停
    Stopping,       // 停止中
    Stopped,        // 已停止
    Failed,         // 失败
    Crashed,        // 崩溃
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessState::Starting => write!(f, "Starting"),
            ProcessState::Running => write!(f, "Running"),
            ProcessState::Paused => write!(f, "Paused"),
            ProcessState::Stopping => write!(f, "Stopping"),
            ProcessState::Stopped => write!(f, "Stopped"),
            ProcessState::Failed => write!(f, "Failed"),
            ProcessState::Crashed => write!(f, "Crashed"),
        }
    }
}

/// 进程优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProcessPriority {
    Low = 1,
    BelowNormal = 2,
    Normal = 3,
    AboveNormal = 4,
    High = 5,
    Realtime = 6,
}

impl Default for ProcessPriority {
    fn default() -> Self {
        ProcessPriority::Normal
    }
}

impl fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessPriority::Low => write!(f, "Low"),
            ProcessPriority::BelowNormal => write!(f, "BelowNormal"),
            ProcessPriority::Normal => write!(f, "Normal"),
            ProcessPriority::AboveNormal => write!(f, "AboveNormal"),
            ProcessPriority::High => write!(f, "High"),
            ProcessPriority::Realtime => write!(f, "Realtime"),
        }
    }
}

/// 进程类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessType {
    Device { device_id: DeviceId },     // 设备进程
    Node { script_id: String },         // Node.js脚本进程
}

impl fmt::Display for ProcessType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessType::Device { device_id } => write!(f, "Device({})", device_id),
            ProcessType::Node { script_id } => write!(f, "Node({})", script_id),
        }
    }
}

/// 进程句柄
pub struct ProcessHandle {
    // 基本信息
    pub process_id: ProcessId,
    pub process_type: ProcessType,
    pub priority: ProcessPriority,
    
    // 状态信息
    pub state: ProcessState,
    pub started_at: SystemTime,
    pub last_heartbeat: Option<SystemTime>,
    
    // 系统进程信息
    pub system_pid: Option<u32>,
    pub child_process: Option<Child>,
    
    // 资源使用
    pub allocated_cores: Vec<usize>,
    pub memory_limit_mb: Option<usize>,
    
    // 统计信息
    pub restart_count: u32,
    pub total_runtime: Duration,
    pub last_error: Option<String>,
}

impl ProcessHandle {
    /// 创建新的进程句柄
    pub fn new(
        process_id: ProcessId,
        process_type: ProcessType,
        priority: ProcessPriority,
    ) -> Self {
        Self {
            process_id,
            process_type,
            priority,
            state: ProcessState::Starting,
            started_at: SystemTime::now(),
            last_heartbeat: None,
            system_pid: None,
            child_process: None,
            allocated_cores: Vec::new(),
            memory_limit_mb: None,
            restart_count: 0,
            total_runtime: Duration::new(0, 0),
            last_error: None,
        }
    }
    
    /// 更新进程状态
    pub fn update_state(&mut self, new_state: ProcessState) {
        if self.state != new_state {
            tracing::info!(
                "进程 {} 状态变化: {} -> {}",
                self.process_id, self.state, new_state
            );
            self.state = new_state;
        }
    }
    
    /// 更新心跳
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Some(SystemTime::now());
    }
    
    /// 检查是否超时（基于心跳）
    pub fn is_timeout(&self, timeout_duration: Duration) -> bool {
        if let Some(last_heartbeat) = self.last_heartbeat {
            if let Ok(elapsed) = last_heartbeat.elapsed() {
                return elapsed > timeout_duration;
            }
        }
        // 如果没有心跳记录，检查启动时间
        if let Ok(elapsed) = self.started_at.elapsed() {
            elapsed > timeout_duration
        } else {
            false
        }
    }
    
    /// 获取运行时间
    pub fn get_runtime(&self) -> Duration {
        self.started_at.elapsed().unwrap_or(Duration::new(0, 0))
    }
    
    /// 设置分配的CPU核心
    pub fn set_allocated_cores(&mut self, cores: Vec<usize>) {
        self.allocated_cores = cores;
    }
    
    /// 设置内存限制
    pub fn set_memory_limit(&mut self, limit_mb: usize) {
        self.memory_limit_mb = Some(limit_mb);
    }
    
    /// 记录错误
    pub fn record_error(&mut self, error: String) {
        self.last_error = Some(error);
        tracing::error!("进程 {} 错误: {}", self.process_id, self.last_error.as_ref().unwrap());
    }
    
    /// 增加重启计数
    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
        tracing::warn!("进程 {} 重启计数: {}", self.process_id, self.restart_count);
    }
    
    /// 检查进程是否健康
    pub fn is_healthy(&self) -> bool {
        match self.state {
            ProcessState::Running => {
                // 检查心跳超时
                !self.is_timeout(Duration::from_secs(30))
            },
            ProcessState::Starting => {
                // 启动中，检查是否启动时间过长
                self.get_runtime() < Duration::from_secs(60)
            },
            _ => false,
        }
    }
    
    /// 获取进程描述
    pub fn get_description(&self) -> String {
        format!(
            "Process[{}]: {} - {} (Priority: {}, Cores: {:?})",
            self.process_id,
            self.process_type,
            self.state,
            self.priority,
            self.allocated_cores
        )
    }
}

impl fmt::Display for ProcessHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}

impl fmt::Debug for ProcessHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessHandle")
            .field("process_id", &self.process_id)
            .field("process_type", &self.process_type)
            .field("state", &self.state)
            .field("priority", &self.priority)
            .field("system_pid", &self.system_pid)
            .field("allocated_cores", &self.allocated_cores)
            .field("restart_count", &self.restart_count)
            .field("runtime", &self.get_runtime())
            .field("is_healthy", &self.is_healthy())
            .finish()
    }
}

/// 设备进程句柄
pub struct DeviceProcessHandle {
    pub base: ProcessHandle,
    pub device_id: DeviceId,
    pub device_name: String,
    pub automation_script: Option<String>,
}

impl DeviceProcessHandle {
    pub fn new(
        process_id: ProcessId,
        device_id: DeviceId,
        device_name: String,
        priority: ProcessPriority,
    ) -> Self {
        Self {
            base: ProcessHandle::new(
                process_id,
                ProcessType::Device { device_id },
                priority,
            ),
            device_id,
            device_name,
            automation_script: None,
        }
    }
    
    pub fn set_automation_script(&mut self, script: String) {
        self.automation_script = Some(script);
    }
}

impl std::ops::Deref for DeviceProcessHandle {
    type Target = ProcessHandle;
    
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DeviceProcessHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// Node.js进程句柄
pub struct NodeProcessHandle {
    pub base: ProcessHandle,
    pub script_path: std::path::PathBuf,
    pub node_version: Option<String>,
    pub environment_vars: std::collections::HashMap<String, String>,
}

impl NodeProcessHandle {
    pub fn new(
        process_id: ProcessId,
        script_path: std::path::PathBuf,
        priority: ProcessPriority,
    ) -> Self {
        let script_id = script_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        Self {
            base: ProcessHandle::new(
                process_id,
                ProcessType::Node { script_id },
                priority,
            ),
            script_path,
            node_version: None,
            environment_vars: std::collections::HashMap::new(),
        }
    }
    
    pub fn set_environment_var(&mut self, key: String, value: String) {
        self.environment_vars.insert(key, value);
    }
}

impl std::ops::Deref for NodeProcessHandle {
    type Target = ProcessHandle;
    
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NodeProcessHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_process_handle_creation() {
        let process_id = "test_process".to_string();
        let process_type = ProcessType::Device { 
            device_id: Uuid::new_v4() 
        };
        let handle = ProcessHandle::new(process_id.clone(), process_type, ProcessPriority::Normal);
        
        assert_eq!(handle.process_id, process_id);
        assert_eq!(handle.state, ProcessState::Starting);
        assert!(handle.is_healthy());
    }

    #[test]
    fn test_state_transitions() {
        let mut handle = ProcessHandle::new(
            "test".to_string(),
            ProcessType::Device { device_id: Uuid::now_v7() },
            ProcessPriority::Normal,
        );
        
        handle.update_state(ProcessState::Running);
        assert_eq!(handle.state, ProcessState::Running);
        
        handle.update_heartbeat();
        assert!(handle.last_heartbeat.is_some());
        assert!(handle.is_healthy());
    }

    #[test]
    fn test_timeout_detection() {
        let mut handle = ProcessHandle::new(
            "test".to_string(),
            ProcessType::Node { script_id: "".to_string() },
            ProcessPriority::Normal,
        );
        
        // 模拟旧的心跳
        handle.last_heartbeat = Some(SystemTime::now() - Duration::from_secs(60));
        
        // 应该检测到超时
        assert!(handle.is_timeout(Duration::from_secs(30)));
    }
}