// 子进程启动器
// 负责启动和管理子进程

use crate::infrastructure::core::{DeviceId, ProcessId};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use tokio::process::Child;

/// 子进程初始化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProcessInitData {
    pub device_id: DeviceId,
    pub process_id: ProcessId,
    pub ipc_endpoint: String,
    pub log_level: String,
    pub shared_memory_keys: Vec<String>, // 共享内存标识符
    pub config_data: Option<serde_json::Value>,
}

/// 子进程启动器
pub struct ChildProcessLauncher {
    executable_path: PathBuf,
}

impl ChildProcessLauncher {
    pub fn new(executable_path: PathBuf) -> Self {
        Self { executable_path }
    }

    /// 启动子进程
    pub async fn launch_child_process(
        &self,
        init_data: ChildProcessInitData,
    ) -> Result<Child, Box<dyn std::error::Error>> {
        // 1. 序列化初始化数据
        let serialized_data = serde_json::to_string(&init_data)?;
        
        // 2. 创建子进程命令
        let mut cmd = Command::new(&self.executable_path);
        
        // 3. 设置环境变量
        cmd.env("CHILD_CONTEXT_DATA", serialized_data);
        cmd.env("RUST_LOG", &init_data.log_level);
        
        // 4. 设置标准输入输出
        cmd.stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // 5. 启动进程
        let child = tokio::process::Command::from(cmd)
            .spawn()
            .map_err(|e| format!("启动子进程失败: {}", e))?;

        tracing::info!(
            "子进程已启动: device_id={}, process_id={}, pid={}",
            init_data.device_id,
            init_data.process_id,
            child.id().unwrap_or(0)
        );

        Ok(child)
    }

    /// 创建子进程初始化数据
    pub fn create_init_data(
        device_id: DeviceId,
        process_id: ProcessId,
        ipc_endpoint: String,
    ) -> ChildProcessInitData {
        ChildProcessInitData {
            device_id,
            process_id,
            ipc_endpoint,
            log_level: "info".to_string(),
            shared_memory_keys: Vec::new(),
            config_data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_data_serialization() {
        let init_data = ChildProcessInitData {
            device_id: 1,
            process_id: "test-child".to_string(),
            ipc_endpoint: "pipe://test".to_string(),
            log_level: "debug".to_string(),
            shared_memory_keys: vec!["mem1".to_string(), "mem2".to_string()],
            config_data: Some(serde_json::json!({"test": true})),
        };

        let serialized = serde_json::to_string(&init_data).unwrap();
        let deserialized: ChildProcessInitData = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(init_data.device_id, deserialized.device_id);
        assert_eq!(init_data.process_id, deserialized.process_id);
    }
}