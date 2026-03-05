// 子进程管理 API — 供前端调用
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    ScriptTaskAction, ScriptTaskMessage,
};
use crate::infrastructure::core::ScriptId;
use tauri::command;

/// 向已运行的子进程发送 Start 命令（开始执行脚本队列）
#[command]
pub async fn cmd_device_start(device_id: DeviceId) -> Result<String, String> {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ProcessControl(ProcessControlMessage {
            action: ProcessAction::Start,
        }),
    );
    IpcServer::send_to_client(&device_id, msg).await;
    Ok(format!("已向设备[{}]发送启动命令", device_id))
}

/// 向子进程发送 Stop 命令（停止当前脚本但不退出）
#[command]
pub async fn cmd_device_stop(device_id: DeviceId) -> Result<String, String> {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ProcessControl(ProcessControlMessage {
            action: ProcessAction::Stop,
        }),
    );
    IpcServer::send_to_client(&device_id, msg).await;
    Ok(format!("已向设备[{}]发送停止命令", device_id))
}

/// 向子进程发送 Pause 命令
#[command]
pub async fn cmd_device_pause(device_id: DeviceId) -> Result<String, String> {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ProcessControl(ProcessControlMessage {
            action: ProcessAction::Pause,
        }),
    );
    IpcServer::send_to_client(&device_id, msg).await;
    Ok(format!("已向设备[{}]发送暂停命令", device_id))
}

/// 向子进程添加脚本到执行队列
#[command]
pub async fn cmd_add_script_to_device(
    device_id: DeviceId,
    script_id: ScriptId,
) -> Result<String, String> {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ScriptTask(ScriptTaskMessage {
            action: ScriptTaskAction::Add { script_id },
        }),
    );
    IpcServer::send_to_client(&device_id, msg).await;
    Ok(format!(
        "已向设备[{}]添加脚本[{}]",
        device_id, script_id
    ))
}

/// 从子进程执行队列中移除脚本
#[command]
pub async fn cmd_remove_script_from_device(
    device_id: DeviceId,
    script_id: ScriptId,
) -> Result<String, String> {
    let msg = IpcMessage::new(
        device_id,
        MessageType::Command,
        MessagePayload::ScriptTask(ScriptTaskMessage {
            action: ScriptTaskAction::Remove { script_id },
        }),
    );
    IpcServer::send_to_client(&device_id, msg).await;
    Ok(format!(
        "已从设备[{}]移除脚本[{}]",
        device_id, script_id
    ))
}

/// 关闭子进程
#[command]
pub async fn cmd_device_shutdown(device_id: DeviceId) -> Result<String, String> {
    if let Some(manager) = get_process_manager() {
        manager.stop_child(&device_id).await?;
        Ok(format!("设备[{}]子进程已关闭", device_id))
    } else {
        Err("进程管理器未初始化".to_string())
    }
}

/// 获取所有运行中的设备
#[command]
pub async fn cmd_get_running_devices() -> Result<Vec<String>, String> {
    if let Some(manager) = get_process_manager() {
        let ids = manager.get_running_device_ids().await;
        Ok(ids.iter().map(|id| id.to_string()).collect())
    } else {
        Err("进程管理器未初始化".to_string())
    }
}
