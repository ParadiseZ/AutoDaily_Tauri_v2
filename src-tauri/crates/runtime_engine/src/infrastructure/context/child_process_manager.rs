use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
};
use crate::infrastructure::logging::log_trait::Log;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

/// 子进程句柄
pub struct ChildProcessHandle {
    pub device_id: DeviceId,
    pub device_name: String,
    pub process: Option<Child>,
    pub pid: Option<u32>,
}

/// 子进程管理器（主进程端）
/// 负责启动、停止、重启子进程
pub struct ChildProcessManager {
    /// 设备ID → 子进程句柄
    processes: RwLock<HashMap<DeviceId, ChildProcessHandle>>,
}

/// 全局子进程管理器
static PROCESS_MANAGER: std::sync::OnceLock<Arc<ChildProcessManager>> = std::sync::OnceLock::new();

pub fn init_process_manager() -> Arc<ChildProcessManager> {
    let manager = Arc::new(ChildProcessManager {
        processes: RwLock::new(HashMap::new()),
    });
    let _ = PROCESS_MANAGER.set(manager.clone());
    manager
}

pub fn get_process_manager() -> Option<Arc<ChildProcessManager>> {
    PROCESS_MANAGER.get().cloned()
}

impl ChildProcessManager {
    /// 启动一个子进程
    pub async fn spawn_child(&self, init_data: ChildProcessInitData) -> Result<(), String> {
        let device_id = init_data.device_id;
        let device_name = init_data.device_config.device_name.clone();

        // 检查是否已在运行
        {
            let processes = self.processes.read().await;
            if let Some(handle) = processes.get(&device_id) {
                if handle.process.is_some() {
                    return Err(format!("设备[{}]的子进程已在运行", device_name));
                }
            }
        }

        // 序列化初始化数据
        let init_json = serde_json::to_string(&init_data)
            .map_err(|e| format!("序列化初始化数据失败: {}", e))?;

        // 获取当前可执行文件路径（子进程使用同一个二进制，通过参数区分）
        let exe_path =
            std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;

        // 启动子进程
        let child = Command::new(&exe_path)
            .arg("--child")
            .env("CHILD_CONTEXT_DATA", &init_json)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("启动子进程失败: {}", e))?;

        let pid = child.id();
        Log::info(&format!(
            "[ process ] 启动设备[{}]子进程成功, PID: {:?}",
            device_name, pid
        ));

        let handle = ChildProcessHandle {
            device_id,
            device_name: device_name.clone(),
            process: Some(child),
            pid,
        };

        self.processes.write().await.insert(device_id, handle);
        Ok(())
    }

    /// 停止一个子进程
    pub async fn stop_child(&self, device_id: &DeviceId) -> Result<(), String> {
        let mut processes = self.processes.write().await;
        if let Some(handle) = processes.get_mut(device_id) {
            // 先通过 IPC 发送 Shutdown 命令
            let shutdown_msg = IpcMessage::new(
                *device_id,
                MessageType::Command,
                MessagePayload::ProcessControl(ProcessControlMessage {
                    action: ProcessAction::Shutdown,
                }),
            );
            IpcServer::send_to_client(device_id, shutdown_msg).await;

            // 等待一段时间后，如果子进程还在运行，则强制 kill
            if let Some(ref mut process) = handle.process {
                let timeout = tokio::time::Duration::from_secs(5);
                match tokio::time::timeout(timeout, process.wait()).await {
                    Ok(Ok(status)) => {
                        Log::info(&format!(
                            "[ process ] 设备[{}]子进程正常退出: {}",
                            handle.device_name, status
                        ));
                    }
                    _ => {
                        // 超时或错误，强制 kill
                        let _ = process.kill().await;
                        Log::warn(&format!(
                            "[ process ] 设备[{}]子进程强制终止",
                            handle.device_name
                        ));
                    }
                }
            }
            handle.process = None;
            handle.pid = None;
            Ok(())
        } else {
            Err(format!("设备[{}]没有运行中的子进程", device_id))
        }
    }

    /// 重启子进程
    pub async fn restart_child(
        &self,
        device_id: &DeviceId,
        init_data: ChildProcessInitData,
    ) -> Result<(), String> {
        self.stop_child(device_id).await.ok(); // 忽略停止错误
        self.spawn_child(init_data).await
    }

    /// 检查子进程是否在运行
    pub async fn is_running(&self, device_id: &DeviceId) -> bool {
        let processes = self.processes.read().await;
        processes
            .get(device_id)
            .map_or(false, |h| h.process.is_some())
    }

    /// 获取所有运行中的子进程设备ID
    pub async fn get_running_device_ids(&self) -> Vec<DeviceId> {
        let processes = self.processes.read().await;
        processes
            .iter()
            .filter(|(_, h)| h.process.is_some())
            .map(|(id, _)| *id)
            .collect()
    }

    /// 停止所有子进程
    pub async fn stop_all(&self) {
        let device_ids: Vec<DeviceId> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };
        for id in device_ids {
            if let Err(e) = self.stop_child(&id).await {
                Log::warn(&format!("[ process ] 停止子进程失败: {}", e));
            }
        }
    }
}
