// 子进程管理 API — 供前端调用
use crate::constant::table_name::DEVICE_TABLE;
use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::context::child_process_manager::get_process_manager;
use crate::infrastructure::core::{DeviceId, ScriptId};
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::message::{
    IpcMessage, MessagePayload, MessageType, ProcessAction, ProcessControlMessage,
    ScriptTaskAction, ScriptTaskMessage,
};
use tauri::command;
use tauri::Manager;

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

/// 启动设备的子进程
#[command]
pub async fn cmd_spawn_device(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
) -> Result<String, String> {
    // 1. 从数据库加载设备配置
    let device_table: DeviceTable = DbRepo::get_by_id(DEVICE_TABLE, &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备[{}]不存在", device_id))?;

    let device_config = device_table.data.0;

    // 2. 获取数据库路径
    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据库路径失败: {}", e))?;

    // 3. 构造初始化数据
    let init_data = ChildProcessInitData {
        device_id,
        device_config: device_config.clone(),
        shm_name: format!("autodaily_shm_{}", device_id),
        log_level: device_config.log_level.clone(),
        cpu_cores: device_config.cores.iter().map(|c| *c as usize).collect(),
        db_path,
    };

    // 4. 获取进程管理器并启动子进程
    let manager = get_process_manager()
        .ok_or_else(|| "进程管理器未初始化".to_string())?;

    manager.spawn_child(init_data).await?;

    Ok(format!(
        "设备[{}]({})子进程已启动",
        device_config.device_name, device_id
    ))
}

/// 检查设备子进程是否在运行
#[command]
pub async fn cmd_is_device_running(device_id: DeviceId) -> Result<bool, String> {
    if let Some(manager) = get_process_manager() {
        Ok(manager.is_running(&device_id).await)
    } else {
        Err("进程管理器未初始化".to_string())
    }
}
