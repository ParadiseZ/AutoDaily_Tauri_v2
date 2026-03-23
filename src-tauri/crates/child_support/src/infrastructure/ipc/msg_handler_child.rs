use crate::infrastructure::ipc::message::{
    ConfigUpdateMessage, ConfigUpdateType, IpcMessage, MessagePayload, ProcessAction,
    ProcessControlMessage, ScriptTaskAction, ScriptTaskMessage,
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
        MessagePayload::ScriptTask(task) => {
            handle_script_task(task);
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
            // TODO: 第二阶段后续 - 通知调度器开始执行
        }
        ProcessAction::Stop => {
            Log::info("[ child ] 收到停止命令，停止当前脚本执行");
            set_running_status(RunningStatus::Idle);
            // 停止当前脚本执行但不退出进程，回到 Idle 状态
            // TODO: 持久化运行时数据
        }
        ProcessAction::Pause => {
            Log::info("[ child ] 收到暂停命令");
            set_running_status(RunningStatus::Paused);
        }
        ProcessAction::Shutdown => {
            Log::info("[ child ] 收到关闭命令，准备退出");
            set_running_status(RunningStatus::Stopping);
            trigger_cancel(); // 取消 CancellationToken，主循环立即退出
            // TODO: 持久化运行时数据
        }
    }
}

fn handle_script_task(task: ScriptTaskMessage) {
    use crate::infrastructure::scripts::scheduler::get_scheduler;

    match task.action {
        ScriptTaskAction::Add { script_id } => {
            Log::info(&format!("[ child ] 添加脚本到队列: {}", script_id));
            if let Some(scheduler) = get_scheduler() {
                tokio::spawn(async move {
                    scheduler.add_script(script_id).await;
                });
            }
        }
        ScriptTaskAction::Remove { script_id } => {
            Log::info(&format!("[ child ] 从队列移除脚本: {}", script_id));
            if let Some(scheduler) = get_scheduler() {
                tokio::spawn(async move {
                    scheduler.remove_script(&script_id).await;
                });
            }
        }
        ScriptTaskAction::Execute { script_id, target } => {
            Log::info(&format!("[ child ] 调试执行脚本: {} target: {:?}", script_id, target));
            if let Some(scheduler) = get_scheduler() {
                tokio::spawn(async move {
                    if let Err(e) = scheduler.debug_execute(script_id, target).await {
                        Log::error(&format!("[ child ] 调试执行失败: {}", e));
                    }
                });
            }
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

