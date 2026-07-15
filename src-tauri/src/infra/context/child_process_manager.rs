//! Root-process child lifecycle management.
use crate::api::local::execution::{
    DeviceConnectionEventPayload, DeviceLifecycleStatus, DeviceStatusEventPayload,
};
use crate::app::constants::MAIN_WINDOW;
use crate::infra::app_handle::get_app_handle;
use crate::infra::context::main_process::{ChildRuntimeStatus, MainProcessCtx};
use crate::infra::ipc::channel_server::IpcServer;
use crate::infra::logging::LogLevel;
use crate::infra::logging::log_trait::Log;
use crate::infra::logging::main_process_log_handler::get_child_log_receiver;
use ad_kernel::ids::{DeviceId, now_millis_string};
use runner_protocol::ChildProcessInitData;
use runner_protocol::message::{
    ConnectionStatusKind, IpcMessage, LogMessage, MessagePayload, MessageType, ProcessAction,
    ProcessControlMessage,
};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::OnceLock;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{RwLock, mpsc, watch};

#[derive(Clone, Debug)]
struct ChildProcessExit {
    success: bool,
    message: String,
}

#[derive(Clone, Debug)]
enum ChildProcessCommand {
    ForceKill,
}

/// 子进程句柄
#[derive(Clone)]
pub struct ChildProcessHandle {
    pub device_name: String,
    pub pid: Option<u32>,
    control_tx: mpsc::UnboundedSender<ChildProcessCommand>,
    exit_rx: watch::Receiver<Option<ChildProcessExit>>,
}

/// 子进程管理器（主进程端）
/// 负责启动、停止、重启子进程
pub struct ChildProcessManager {
    /// 设备ID → 子进程句柄
    processes: RwLock<HashMap<DeviceId, ChildProcessHandle>>,
}

/// 全局子进程管理器
static PROCESS_MANAGER: std::sync::OnceLock<Arc<ChildProcessManager>> = std::sync::OnceLock::new();
type ChildProcessExitHandler = Arc<dyn Fn(DeviceId, bool, String) + Send + Sync>;
static CHILD_PROCESS_EXIT_HANDLER: OnceLock<ChildProcessExitHandler> = OnceLock::new();

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

pub fn set_child_process_exit_handler(handler: ChildProcessExitHandler) -> Result<(), String> {
    CHILD_PROCESS_EXIT_HANDLER
        .set(handler)
        .map_err(|_| "子进程退出处理器已注册".to_string())
}

fn spawn_child_stderr_forwarder(
    device_id: DeviceId,
    device_name: String,
    stderr: tokio::process::ChildStderr,
) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let message = line.trim();
                    if message.is_empty() {
                        continue;
                    }

                    Log::error(&format!(
                        "[ process ] 设备[{}]子进程 stderr: {}",
                        device_name, message
                    ));
                    write_child_log_line(device_id, LogLevel::Error, message).await;

                    if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
                        let emit_data = serde_json::json!({
                            "deviceId": device_id.to_string(),
                            "level": LogLevel::Error.to_string(),
                            "message": message,
                            "time": chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
                        });
                        let _ = main_window.emit("child-log", emit_data);
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    Log::warn(&format!(
                        "[ process ] 读取设备[{}]子进程 stderr 失败: {}",
                        device_name, error
                    ));
                    break;
                }
            }
        }
    });
}

async fn write_child_log_line(device_id: DeviceId, level: LogLevel, message: &str) {
    if let Some(receiver) = get_child_log_receiver() {
        receiver
            .handle_log(
                &device_id,
                &LogMessage {
                    level,
                    message: message.to_string(),
                    module: Some("child-process".to_string()),
                },
            )
            .await;
    }
}

fn emit_device_status_event(device_id: DeviceId, status: &str, message: &str) {
    let app_handle = get_app_handle();
    let lifecycle_status = match status {
        "Stopped" => DeviceLifecycleStatus::Stopped,
        "Error" => DeviceLifecycleStatus::Error,
        _ => DeviceLifecycleStatus::Error,
    };
    let _ = app_handle.state::<MainProcessCtx>().set_device_lifecycle(
        device_id,
        lifecycle_status.clone(),
        None,
        Some(message.to_string()),
        Some(now_millis_string()),
    );
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let payload = DeviceStatusEventPayload {
            device_id,
            session_id: None,
            status: lifecycle_status,
            current_script_id: None,
            message: Some(message.to_string()),
            at: now_millis_string(),
        };
        let _ = main_window.emit("device-status", payload);
    }
}

fn emit_device_connection_event(device_id: DeviceId, status: ConnectionStatusKind, message: &str) {
    if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
        let payload = DeviceConnectionEventPayload {
            device_id,
            status,
            message: Some(message.to_string()),
            at: now_millis_string(),
        };
        let _ = main_window.emit("device-connection-status", payload);
    }
}

async fn finalize_child_exit(
    device_id: DeviceId,
    device_name: String,
    pid: Option<u32>,
    exit: ChildProcessExit,
) {
    if exit.success {
        Log::info(&format!("[ process ] {}", exit.message));
    } else {
        Log::error(&format!("[ process ] {}", exit.message));
    }
    write_child_log_line(
        device_id,
        if exit.success {
            LogLevel::Info
        } else {
            LogLevel::Error
        },
        &exit.message,
    )
    .await;

    if let Some(manager) = get_process_manager() {
        let mut processes = manager.processes.write().await;
        if processes
            .get(&device_id)
            .is_some_and(|handle| handle.pid == pid)
        {
            processes.remove(&device_id);
        }
    }

    let app_handle = get_app_handle();
    if let Ok(mut guard) = app_handle.state::<MainProcessCtx>().ipc_servers.write() {
        guard.retain(|registered_device_id, _| **registered_device_id != device_id);
    }
    let runtime_state = app_handle.state::<MainProcessCtx>();
    let _ = runtime_state.set_child_runtime_status(
        device_id,
        if exit.success {
            ChildRuntimeStatus::Exited
        } else {
            ChildRuntimeStatus::Crashed
        },
    );
    let _ = runtime_state.set_device_connection_state(
        device_id,
        ConnectionStatusKind::DeviceDisconnected,
        Some(exit.message.clone()),
    );

    emit_device_connection_event(
        device_id,
        ConnectionStatusKind::DeviceDisconnected,
        &exit.message,
    );
    emit_device_status_event(
        device_id,
        if exit.success { "Stopped" } else { "Error" },
        &exit.message,
    );
    if let Some(handler) = CHILD_PROCESS_EXIT_HANDLER.get() {
        handler(device_id, exit.success, exit.message.clone());
    }

    if let Some(receiver) = get_child_log_receiver() {
        receiver.unregister_device(&device_id).await;
    }

    Log::info(&format!(
        "[ process ] 设备[{}]子进程退出清理完成, pid={:?}",
        device_name, pid
    ));
}

async fn watch_child_process(
    device_id: DeviceId,
    device_name: String,
    pid: Option<u32>,
    mut child: Child,
    mut control_rx: mpsc::UnboundedReceiver<ChildProcessCommand>,
    exit_tx: watch::Sender<Option<ChildProcessExit>>,
) {
    let exit = loop {
        let mut wait_result = Box::pin(child.wait());

        tokio::select! {
            result = &mut wait_result => {
                break match result {
                    Ok(status) => {
                        let success = status.success();
                        let code = status.code();
                        let message = if success {
                            format!("设备[{}]子进程已退出，code={:?}", device_name, code)
                        } else {
                            format!("设备[{}]子进程异常退出，code={:?}", device_name, code)
                        };
                        ChildProcessExit { success, message }
                    }
                    Err(error) => ChildProcessExit {
                        success: false,
                        message: format!("设备[{}]子进程等待退出失败: {}", device_name, error),
                    },
                };
            }
            command = control_rx.recv() => {
                match command {
                    Some(ChildProcessCommand::ForceKill) => {
                        drop(wait_result);
                        if let Err(error) = child.kill().await {
                            Log::warn(&format!(
                                "[ process ] 设备[{}]子进程强制终止失败: {}",
                                device_name, error
                            ));
                        }
                    }
                    None => {}
                }
            }
        }
    };

    let _ = exit_tx.send(Some(exit.clone()));
    finalize_child_exit(device_id, device_name, pid, exit).await;
}

async fn wait_for_child_exit(
    exit_rx: &mut watch::Receiver<Option<ChildProcessExit>>,
    timeout: std::time::Duration,
) -> Result<ChildProcessExit, String> {
    if let Some(exit) = exit_rx.borrow().clone() {
        return Ok(exit);
    }

    tokio::time::timeout(timeout, async {
        loop {
            exit_rx
                .changed()
                .await
                .map_err(|_| "子进程退出通知通道已关闭".to_string())?;
            if let Some(exit) = exit_rx.borrow().clone() {
                return Ok(exit);
            }
        }
    })
    .await
    .map_err(|_| "等待子进程退出超时".to_string())?
}

impl ChildProcessManager {
    /// 启动一个子进程
    pub async fn spawn_child(&self, init_data: ChildProcessInitData) -> Result<(), String> {
        let device_id = init_data.device_id;
        let device_name = init_data.device_config.device_name.clone();

        // 检查是否已在运行
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&device_id) {
                return Err(format!("设备[{}]的子进程已在运行", device_name));
            }
        }

        // 序列化初始化数据
        let init_json = serde_json::to_string(&init_data)
            .map_err(|e| format!("序列化初始化数据失败: {}", e))?;

        // 获取当前可执行文件路径（子进程使用同一个二进制，通过参数区分）
        let exe_path =
            std::env::current_exe().map_err(|e| format!("获取可执行文件路径失败: {}", e))?;

        if let Some(receiver) = get_child_log_receiver() {
            receiver
                .register_device(
                    device_id,
                    device_name.clone(),
                    init_data.device_config.log_to_file,
                )
                .await;
        }

        // 启动子进程
        let spawn_result = Command::new(&exe_path)
            .arg("--child")
            .env("CHILD_CONTEXT_DATA", &init_json)
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn();
        let mut child = match spawn_result {
            Ok(child) => child,
            Err(error) => {
                if let Some(receiver) = get_child_log_receiver() {
                    receiver.unregister_device(&device_id).await;
                }
                return Err(format!("启动子进程失败: {}", error));
            }
        };

        if let Some(stderr) = child.stderr.take() {
            spawn_child_stderr_forwarder(device_id, device_name.clone(), stderr);
        }

        let pid = child.id();
        let (control_tx, control_rx) = mpsc::unbounded_channel();
        let (exit_tx, exit_rx) = watch::channel(None);
        tokio::spawn(watch_child_process(
            device_id,
            device_name.clone(),
            pid,
            child,
            control_rx,
            exit_tx,
        ));

        Log::info(&format!(
            "[ process ] 启动设备[{}]子进程成功, PID: {:?}",
            device_name, pid
        ));
        write_child_log_line(
            device_id,
            LogLevel::Info,
            &format!("子进程已启动，PID={:?}", pid),
        )
        .await;

        let handle = ChildProcessHandle {
            device_name: device_name.clone(),
            pid,
            control_tx,
            exit_rx,
        };

        self.processes.write().await.insert(device_id, handle);
        Ok(())
    }

    /// 停止一个子进程
    pub async fn stop_child(&self, device_id: &DeviceId) -> Result<(), String> {
        let Some(handle) = self.processes.read().await.get(device_id).cloned() else {
            return Err("目标设备没有运行中的子进程".to_string());
        };

        let shutdown_msg = IpcMessage::new(
            *device_id,
            MessageType::Command,
            MessagePayload::ProcessControl(ProcessControlMessage {
                action: ProcessAction::Shutdown,
            }),
        );
        IpcServer::send_to_client(device_id, shutdown_msg).await;

        let mut exit_rx = handle.exit_rx.clone();
        match wait_for_child_exit(&mut exit_rx, tokio::time::Duration::from_secs(5)).await {
            Ok(_) => Ok(()),
            Err(_) => {
                Log::warn(&format!(
                    "[ process ] 设备[{}]子进程未在超时内退出，准备强制终止",
                    handle.device_name
                ));
                handle
                    .control_tx
                    .send(ChildProcessCommand::ForceKill)
                    .map_err(|_| {
                        format!("设备[{}]子进程强制终止命令发送失败", handle.device_name)
                    })?;
                wait_for_child_exit(&mut exit_rx, tokio::time::Duration::from_secs(5)).await?;
                Ok(())
            }
        }
    }

    /// 检查子进程是否在运行
    pub async fn is_running(&self, device_id: &DeviceId) -> bool {
        let processes = self.processes.read().await;
        processes.contains_key(device_id)
    }

    /// 获取所有运行中的子进程设备ID
    pub async fn get_running_device_ids(&self) -> Vec<DeviceId> {
        let processes = self.processes.read().await;
        processes.keys().cloned().collect()
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
