use crate::infrastructure::ipc::message::{
    ConfigUpdateMessage, ConfigUpdateType, IpcMessage, MessagePayload, ProcessAction,
    ProcessControlMessage, RuntimeLifecyclePhase, RuntimeProgressPhase, RuntimeRecoveryPhase,
    RuntimeScheduleStatus, SessionControlMessage,
};
use crate::infrastructure::context::child_process_sec::{
    get_ipc_client, set_running_status, trigger_cancel, RunningStatus,
};
use crate::infrastructure::ipc::runtime_reporter::{
    emit_lifecycle_event, emit_lifecycle_event_with, emit_progress_event, emit_recovery_event,
    emit_schedule_event,
};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::session::recovery_checkpoint_store::prepare_and_persist_checkpoint;
use crate::infrastructure::session::runtime_session::{
    clear_runtime_session, replace_runtime_session,
};
use std::sync::atomic::Ordering;

/// 子进程消息处理器
/// 处理来自主进程的命令消息
pub async fn handle_main_message(msg: IpcMessage) {
    match msg.payload {
        MessagePayload::ProcessControl(ctrl) => {
            handle_process_control(ctrl);
        }
        MessagePayload::SessionControl(control) => {
            handle_session_control(control).await;
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
            emit_lifecycle_event(RuntimeLifecyclePhase::Running, None);
            // TODO: 第二阶段后续 - 通知调度器开始执行
        }
        ProcessAction::Stop => {
            Log::info("[ child ] 收到停止命令，停止当前脚本执行");
            set_running_status(RunningStatus::Idle);
            emit_progress_event(
                RuntimeProgressPhase::Idle,
                None,
                None,
                None,
                None,
                Some("收到停止命令".to_string()),
            );
            emit_lifecycle_event(
                RuntimeLifecyclePhase::Idle,
                Some("收到停止命令".to_string()),
            );
            // 停止当前脚本执行但不退出进程，回到 Idle 状态
            // TODO: 持久化运行时数据
        }
        ProcessAction::Pause => {
            Log::info("[ child ] 收到暂停命令");
            set_running_status(RunningStatus::Paused);
            emit_progress_event(
                RuntimeProgressPhase::Paused,
                None,
                None,
                None,
                None,
                Some("执行已暂停".to_string()),
            );
            emit_lifecycle_event(RuntimeLifecyclePhase::Paused, None);
        }
        ProcessAction::Shutdown => {
            Log::info("[ child ] 收到关闭命令，准备退出");
            set_running_status(RunningStatus::Stopping);
            emit_lifecycle_event(RuntimeLifecyclePhase::Stopping, None);
            trigger_cancel(); // 取消 CancellationToken，主循环立即退出
            // TODO: 持久化运行时数据
        }
    }
}

async fn handle_session_control(control: SessionControlMessage) {
    use crate::infrastructure::scripts::scheduler::get_scheduler;

    match control {
        SessionControlMessage::LoadSession { session, checkpoint } => {
            let summary = replace_runtime_session(session.clone(), checkpoint.clone()).await;
            Log::info(&format!(
                "[ child ] 加载 session[{}]，队列长度: {}，checkpoint: {}",
                summary.session_id,
                summary.queue_len,
                if summary.has_checkpoint { "yes" } else { "no" }
            ));
            if let Some(scheduler) = get_scheduler() {
                scheduler.load_session(session).await;
            }
            set_running_status(RunningStatus::Idle);
            emit_progress_event(
                RuntimeProgressPhase::Loading,
                None,
                None,
                None,
                None,
                Some(format!("运行会话已加载，队列 {} 项", summary.queue_len)),
            );
            emit_lifecycle_event_with(
                RuntimeLifecyclePhase::Loaded,
                Some(summary.session_id),
                None,
                Some("运行会话已加载".to_string()),
            );
            emit_lifecycle_event_with(
                RuntimeLifecyclePhase::Idle,
                Some(summary.session_id),
                None,
                Some("设备待命，等待执行命令".to_string()),
            );
            if let Some(checkpoint) = checkpoint {
                emit_recovery_event(
                    RuntimeRecoveryPhase::CheckpointLoaded,
                    Some(checkpoint.execution_id),
                    checkpoint.assignment_id,
                    Some(checkpoint.script_id),
                    checkpoint.task_id,
                    checkpoint.step_id,
                    Some(checkpoint.updated_at),
                    Some("已加载可恢复检查点".to_string()),
                );
            }
        }
        SessionControlMessage::ReloadSession { session, checkpoint } => {
            let summary = replace_runtime_session(session.clone(), checkpoint.clone()).await;
            Log::info(&format!(
                "[ child ] 热更新 session[{}]，队列长度: {}，checkpoint: {}",
                summary.session_id,
                summary.queue_len,
                if summary.has_checkpoint { "yes" } else { "no" }
            ));
            if let Some(scheduler) = get_scheduler() {
                scheduler.load_session(session).await;
            }
            emit_progress_event(
                RuntimeProgressPhase::Loading,
                None,
                None,
                None,
                None,
                Some(format!("运行会话已热更新，队列 {} 项", summary.queue_len)),
            );
            emit_lifecycle_event_with(
                RuntimeLifecyclePhase::Loaded,
                Some(summary.session_id),
                None,
                Some("运行会话已热更新".to_string()),
            );
            if let Some(checkpoint) = checkpoint {
                emit_recovery_event(
                    RuntimeRecoveryPhase::CheckpointLoaded,
                    Some(checkpoint.execution_id),
                    checkpoint.assignment_id,
                    Some(checkpoint.script_id),
                    checkpoint.task_id,
                    checkpoint.step_id,
                    Some(checkpoint.updated_at),
                    Some("已热加载可恢复检查点".to_string()),
                );
            }
        }
        SessionControlMessage::PrepareCheckpoint { reason } => {
            Log::info(&format!("[ child ] 收到 checkpoint 准备命令: {:?}", reason));
            emit_recovery_event(
                RuntimeRecoveryPhase::CheckpointPreparing,
                None,
                None,
                None,
                None,
                None,
                None,
                Some("正在准备恢复检查点".to_string()),
            );
            match prepare_and_persist_checkpoint(reason).await {
                Ok(Some(checkpoint)) => {
                    emit_recovery_event(
                        RuntimeRecoveryPhase::CheckpointReady,
                        Some(checkpoint.execution_id),
                        checkpoint.assignment_id,
                        Some(checkpoint.script_id),
                        checkpoint.task_id,
                        checkpoint.step_id,
                        Some(checkpoint.updated_at),
                        Some("恢复检查点已保存".to_string()),
                    );
                    emit_progress_event(
                        RuntimeProgressPhase::Paused,
                        checkpoint.assignment_id,
                        Some(checkpoint.script_id),
                        checkpoint.task_id,
                        checkpoint.step_id,
                        Some("恢复检查点已保存，等待主进程下一步动作".to_string()),
                    );
                }
                Ok(None) => {
                    emit_recovery_event(
                        RuntimeRecoveryPhase::CheckpointReady,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some("当前没有活动 execution，无需保存检查点".to_string()),
                    );
                }
                Err(error) => {
                    Log::error(&format!("[ child ] 保存 checkpoint 失败: {}", error));
                    emit_progress_event(
                        RuntimeProgressPhase::Failed,
                        None,
                        None,
                        None,
                        None,
                        Some(format!("保存 checkpoint 失败: {}", error)),
                    );
                }
            }
        }
        SessionControlMessage::ClearSession => {
            let cleared = clear_runtime_session().await;
            Log::info("[ child ] 清空当前 session");
            if let Some(scheduler) = get_scheduler() {
                scheduler.clear_session().await;
            }
            set_running_status(RunningStatus::Idle);
            emit_schedule_event(
                RuntimeScheduleStatus::Cleared,
                None,
                None,
                None,
                None,
                None,
                Some("运行会话已清空".to_string()),
            );
            emit_progress_event(
                RuntimeProgressPhase::Idle,
                None,
                None,
                None,
                None,
                Some("运行会话已清空".to_string()),
            );
            emit_lifecycle_event_with(
                RuntimeLifecyclePhase::Idle,
                cleared.map(|summary| summary.session_id),
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

