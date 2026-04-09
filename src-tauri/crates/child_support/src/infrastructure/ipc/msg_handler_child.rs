use crate::infrastructure::ipc::message::{
    ConfigUpdateMessage, ConfigUpdateType, IpcMessage, MessagePayload, MessageType, ProcessAction,
    ProcessControlMessage, RuntimeEventMessage, RuntimeLifecycleEvent, RuntimeLifecyclePhase,
    SessionControlMessage,
};
use crate::infrastructure::context::child_process_sec::{
    get_ipc_client, set_running_status, trigger_cancel, RunningStatus,
};
use crate::infrastructure::logging::log_trait::Log;
use std::sync::atomic::Ordering;

/// 子进程消息处理器
/// 处理来自主进程的命令消息
pub fn handle_main_message(msg: IpcMessage) {
    match msg.payload {
        MessagePayload::ProcessControl(ctrl) => {
            handle_process_control(ctrl);
        }
        MessagePayload::SessionControl(control) => {
            handle_session_control(control);
        }
        MessagePayload::ConfigUpdate(config) => {
            handle_config_update(config);
        }
        _ => {
            Log::warn(&format!("[ child ] 收到未处理的消息类型: {:?}", msg.message_type));
        }
    }
}

fn handle_process_control(ctrl: ProcessControlMessage) {
    match ctrl.action {
        ProcessAction::Start => {
            Log::info("[ child ] 收到启动命令");
            set_running_status(RunningStatus::Running);
            emit_lifecycle_event(RuntimeLifecyclePhase::Running, None, None);
            // TODO: 第二阶段后续 - 通知调度器开始执行
        }
        ProcessAction::Stop => {
            Log::info("[ child ] 收到停止命令，停止当前脚本执行");
            set_running_status(RunningStatus::Idle);
            emit_lifecycle_event(RuntimeLifecyclePhase::Idle, None, Some("收到停止命令".to_string()));
            // 停止当前脚本执行但不退出进程，回到 Idle 状态
            // TODO: 持久化运行时数据
        }
        ProcessAction::Pause => {
            Log::info("[ child ] 收到暂停命令");
            set_running_status(RunningStatus::Paused);
            emit_lifecycle_event(RuntimeLifecyclePhase::Paused, None, None);
        }
        ProcessAction::Shutdown => {
            Log::info("[ child ] 收到关闭命令，准备退出");
            set_running_status(RunningStatus::Stopping);
            emit_lifecycle_event(RuntimeLifecyclePhase::Stopping, None, None);
            trigger_cancel(); // 取消 CancellationToken，主循环立即退出
            // TODO: 持久化运行时数据
        }
    }
}

fn handle_session_control(control: SessionControlMessage) {
    use crate::infrastructure::scripts::scheduler::get_scheduler;

    match control {
        SessionControlMessage::LoadSession { session, checkpoint } => {
            let session_id = session.session_id;
            let queue_len = session.queue.len();
            Log::info(&format!(
                "[ child ] 加载 session[{}]，队列长度: {}，checkpoint: {}",
                session_id,
                queue_len,
                if checkpoint.is_some() { "yes" } else { "no" }
            ));
            if let Some(scheduler) = get_scheduler() {
                tokio::spawn(async move {
                    scheduler.load_session(session).await;
                });
            }
            set_running_status(RunningStatus::Idle);
            emit_lifecycle_event(
                RuntimeLifecyclePhase::Loaded,
                Some(session_id),
                Some("运行会话已加载".to_string()),
            );
        }
        SessionControlMessage::ReloadSession { session, checkpoint } => {
            let session_id = session.session_id;
            let queue_len = session.queue.len();
            Log::info(&format!(
                "[ child ] 热更新 session[{}]，队列长度: {}，checkpoint: {}",
                session_id,
                queue_len,
                if checkpoint.is_some() { "yes" } else { "no" }
            ));
            if let Some(scheduler) = get_scheduler() {
                tokio::spawn(async move {
                    scheduler.load_session(session).await;
                });
            }
            emit_lifecycle_event(
                RuntimeLifecyclePhase::Loaded,
                Some(session_id),
                Some("运行会话已热更新".to_string()),
            );
        }
        SessionControlMessage::PrepareCheckpoint { reason } => {
            Log::info(&format!("[ child ] 收到 checkpoint 准备命令: {:?}", reason));
            emit_lifecycle_event(
                RuntimeLifecyclePhase::Paused,
                None,
                Some("checkpoint 准备逻辑待接入".to_string()),
            );
        }
        SessionControlMessage::ClearSession => {
            Log::info("[ child ] 清空当前 session");
            if let Some(scheduler) = get_scheduler() {
                tokio::spawn(async move {
                    scheduler.clear_session().await;
                });
            }
            set_running_status(RunningStatus::Idle);
            emit_lifecycle_event(
                RuntimeLifecyclePhase::Idle,
                None,
                Some("运行会话已清空".to_string()),
            );
        }
    }
}

fn handle_config_update(config: ConfigUpdateMessage) {
    match config.update {
        ConfigUpdateType::LogLevel(level) => {
            if let Some(client) = get_ipc_client() {
                let level_u8 = level.clone() as u8;
                client.log_level.store(level_u8, Ordering::Relaxed);
                Log::info(&format!("[ child ] 日志级别已更新为: {}", level));
            }
        }
        ConfigUpdateType::LogToFile(enabled) => {
            Log::info(&format!("[ child ] 日志写入文件: {}", enabled));
            // log_to_file 由主进程的 ChildLogReceiver 控制，不需要子进程处理
        }
        ConfigUpdateType::AdbPath(path) => {
            Log::info(&format!("[ child ] ADB路径已更新: {:?}", path));
            tokio::spawn(async move {
                let adb_ctx = crate::infrastructure::adb_cli_local::adb_context::get_adb_ctx();
                let mut config = adb_ctx.adb_executor.adb_config.lock().await;
                config.update_adb_path(path);
            });
        }
        ConfigUpdateType::AdbServerAddr(addr) => {
            Log::info(&format!("[ child ] ADB服务地址已更新: {:?}", addr));
            tokio::spawn(async move {
                let adb_ctx = crate::infrastructure::adb_cli_local::adb_context::get_adb_ctx();
                let mut config = adb_ctx.adb_executor.adb_config.lock().await;
                config.update_server_addr(addr);
            });
        }
    }
}

fn emit_lifecycle_event(
    phase: RuntimeLifecyclePhase,
    session_id: Option<crate::infrastructure::core::SessionId>,
    message: Option<String>,
) {
    let at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string());

    let event = RuntimeEventMessage::Lifecycle(RuntimeLifecycleEvent {
        session_id,
        phase,
        current_script_id: current_script_for_event(),
        message,
        at,
    });

    if let Some(client) = get_ipc_client() {
        tokio::spawn(async move {
            let msg = IpcMessage::new(
                *client.device_id,
                MessageType::Status,
                MessagePayload::RuntimeEvent(event),
            );
            if let Err(error) = client.send_ensure(msg).await {
                Log::warn(&format!("[ child ] 发送生命周期事件失败: {}", error));
            }
        });
    }
}

fn current_script_for_event() -> Option<crate::infrastructure::core::ScriptId> {
    None
}

