use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::context::child_process_sec::{
    get_ipc_client, set_running_status, trigger_cancel, RunningStatus,
};
use crate::infrastructure::devices::device_ctx::get_device_ctx;
use crate::infrastructure::ipc::message::{
    CaptureControlMessage, ConfigUpdateMessage, ConfigUpdateType, ConnectionAction,
    ConnectionControlMessage, ConnectionStatusKind, IpcMessage, MessagePayload, ProcessAction,
    ProcessControlMessage, RuntimeLifecyclePhase, RuntimeProgressPhase, RuntimeScheduleStatus,
    SessionControlMessage,
};
use crate::infrastructure::ipc::runtime_reporter::{
    emit_capture_event, emit_connection_event, emit_lifecycle_event, emit_lifecycle_event_with,
    emit_progress_event, emit_schedule_event,
};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::session::runtime_session::{
    clear_runtime_session, replace_runtime_session,
};
use image::DynamicImage;
use runtime_engine::domain::devices::device_conf::DevicePlatform;
use runtime_engine::infrastructure::devices::device_launcher::{
    ensure_device_connection, probe_device_config_connection, wait_for_device_connection,
};
use std::sync::atomic::Ordering;
use vision_core::infrastructure::image::load_image::dynamic_image_to_base64;

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
        MessagePayload::ConnectionControl(control) => {
            handle_connection_control(control).await;
        }
        MessagePayload::CaptureControl(control) => {
            handle_capture_control(msg.id, control).await;
        }
        MessagePayload::ConfigUpdate(config) => {
            handle_config_update(config);
        }
        _ => {
            Log::warn(&format!(
                "[ child ] 收到未处理的消息类型: {:?}",
                msg.message_type
            ));
        }
    }
}

async fn handle_connection_control(control: ConnectionControlMessage) {
    match control.action {
        ConnectionAction::Probe => {
            Log::info("[ child ] 收到连接探测命令");
            emit_connection_event(
                ConnectionStatusKind::Checking,
                Some("正在检查设备连接".to_string()),
            );

            let device_config = get_device_ctx().device_config.read().await.clone();
            if matches!(device_config.platform, DevicePlatform::Desktop) {
                emit_connection_event(
                    ConnectionStatusKind::Connected,
                    Some("Desktop 平台无需 ADB 连接".to_string()),
                );
                return;
            }

            let result = probe_device_config_connection(&device_config);

            match result {
                Ok(runtime_connect) => {
                    ADBCtx::new(runtime_connect).await;
                    emit_connection_event(
                        ConnectionStatusKind::Connected,
                        Some("设备连接可用".to_string()),
                    );
                }
                Err(error) => {
                    Log::warn(&format!("[ child ] 设备连接探测失败: {}", error));
                    emit_connection_event(ConnectionStatusKind::Disconnected, Some(error));
                }
            }
        }
        ConnectionAction::EnsureReady => {
            Log::info("[ child ] 收到连接准备命令");
            emit_connection_event(
                ConnectionStatusKind::Checking,
                Some("正在准备设备连接".to_string()),
            );

            let device_config = get_device_ctx().device_config.read().await.clone();
            if matches!(device_config.platform, DevicePlatform::Desktop) {
                emit_connection_event(
                    ConnectionStatusKind::Connected,
                    Some("Desktop 平台无需 ADB 连接".to_string()),
                );
                return;
            }

            match ensure_device_connection(&device_config).await {
                Ok(runtime_connect) => {
                    ADBCtx::new(runtime_connect).await;
                    emit_connection_event(
                        ConnectionStatusKind::Connected,
                        Some("设备连接已就绪".to_string()),
                    );
                }
                Err(error) => {
                    Log::warn(&format!("[ child ] 设备连接准备失败: {}", error));
                    emit_connection_event(ConnectionStatusKind::Disconnected, Some(error));
                }
            }
        }
        ConnectionAction::EnsureReadyAfterLaunch => {
            Log::info("[ child ] 收到连接准备命令（主线程已启动模拟器）");
            emit_connection_event(
                ConnectionStatusKind::Checking,
                Some("正在等待设备连接就绪".to_string()),
            );

            let device_config = get_device_ctx().device_config.read().await.clone();
            if matches!(device_config.platform, DevicePlatform::Desktop) {
                emit_connection_event(
                    ConnectionStatusKind::Connected,
                    Some("Desktop 平台无需 ADB 连接".to_string()),
                );
                return;
            }

            match wait_for_device_connection(&device_config).await {
                Ok(runtime_connect) => {
                    ADBCtx::new(runtime_connect).await;
                    emit_connection_event(
                        ConnectionStatusKind::Connected,
                        Some("设备连接已就绪".to_string()),
                    );
                }
                Err(error) => {
                    Log::warn(&format!("[ child ] 设备连接准备失败: {}", error));
                    emit_connection_event(ConnectionStatusKind::Disconnected, Some(error));
                }
            }
        }
    }
}

async fn handle_capture_control(
    request_id: crate::infrastructure::core::MessageId,
    _control: CaptureControlMessage,
) {
    Log::info("[ child ] 收到设备截图命令");
    let device_ctx = get_device_ctx();

    if !device_ctx.valid_capture().await {
        emit_capture_event(
            request_id,
            None,
            Some("设备截图校验失败：请检查截图方式、窗口状态或设备连接".to_string()),
        );
        return;
    }

    match device_ctx.get_screenshot().await {
        Some(image) => match dynamic_image_to_base64(&DynamicImage::ImageRgba8(image)) {
            Ok(image_data) => emit_capture_event(request_id, Some(image_data), None),
            Err(error) => emit_capture_event(
                request_id,
                None,
                Some(format!("设备截图编码失败：{}", error)),
            ),
        },
        None => emit_capture_event(request_id, None, Some("设备截图失败".to_string())),
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
        SessionControlMessage::LoadSession { session } => {
            let summary = replace_runtime_session(session.clone()).await;
            Log::info(&format!(
                "[ child ] 加载 session[{}]，队列长度: {}",
                summary.session_id, summary.queue_len
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
        }
        SessionControlMessage::ReloadSession { session } => {
            let summary = replace_runtime_session(session.clone()).await;
            Log::info(&format!(
                "[ child ] 热更新 session[{}]，队列长度: {}",
                summary.session_id, summary.queue_len
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
