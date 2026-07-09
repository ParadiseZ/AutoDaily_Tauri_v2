use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::context::child_process_sec::{
    clear_stop_request, get_ipc_client, request_stop_execution, set_running_status, trigger_cancel,
    RunningStatus,
};
use crate::infrastructure::devices::device_ctx::try_get_device_ctx;
use crate::infrastructure::ipc::message::{
    CaptureControlMessage, ConfigUpdateMessage, ConnectionAction, ConnectionControlMessage,
    ConnectionStatusKind, IpcMessage, MessagePayload, ProcessAction, ProcessControlMessage,
    RuntimeLifecyclePhase, RuntimeProgressPhase, RuntimeScheduleStatus, SessionControlMessage,
};
use crate::infrastructure::ipc::runtime_reporter::{
    emit_capture_event, emit_connection_event, emit_lifecycle_event_now,
    emit_lifecycle_event_with_now, emit_progress_event, emit_schedule_event,
};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::session::runtime_session::{
    clear_runtime_session, replace_runtime_session,
};
use image::DynamicImage;
use runtime_engine::domain::devices::device_conf::{DeviceConfig, DevicePlatform};
use runtime_engine::infrastructure::devices::device_launcher::{
    ensure_device_connection_with_progress, probe_device_config_connection_with_timeout,
    resolve_runtime_connect_config,
};
use std::sync::atomic::Ordering;
use vision_core::infrastructure::image::load_image::dynamic_image_to_base64;

/// 子进程消息处理器
/// 处理来自主进程的命令消息
pub async fn handle_main_message(msg: IpcMessage) {
    match msg.payload {
        MessagePayload::ProcessControl(ctrl) => {
            handle_process_control(ctrl).await;
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
            handle_config_update(config).await;
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
    let Some(device_ctx) = try_get_device_ctx() else {
        let message = "[ child ] 设备上下文尚未初始化，请稍后重试".to_string();
        Log::warn(&message);
        emit_connection_event(ConnectionStatusKind::DeviceDisconnected, Some(message));
        return;
    };

    match control.action {
        ConnectionAction::Probe => {
            Log::info("[ child ] 收到连接探测命令");
            emit_connection_event(
                ConnectionStatusKind::DeviceChecking,
                Some("正在预探测设备连接".to_string()),
            );

            let device_config = device_ctx.device_config.read().await.clone();
            if matches!(device_config.platform, DevicePlatform::Desktop) {
                emit_connection_event(
                    ConnectionStatusKind::DeviceConnected,
                    Some("Desktop 平台无需 ADB 连接".to_string()),
                );
                return;
            }

            let result = probe_device_config_connection_with_timeout(
                &device_config,
                std::time::Duration::from_secs(3),
            )
            .await;

            match result {
                Ok(runtime_connect) => match ADBCtx::new(runtime_connect).await {
                    Ok(()) => emit_connection_event(
                        ConnectionStatusKind::DeviceConnected,
                        Some("[ child ] 设备已连接".to_string()),
                    ),
                    Err(error) => {
                        Log::warn(&format!("[ child ] ADB上下文连接初始化失败: {}", error));
                        emit_connection_event(
                            ConnectionStatusKind::DeviceDisconnected,
                            Some(error),
                        );
                    }
                },
                Err(error) => {
                    Log::warn(&format!("[ child ] 设备连接探测失败: {}", error));
                    emit_connection_event(ConnectionStatusKind::DeviceDisconnected, Some(error));
                }
            }
        }
        ConnectionAction::EnsureReady => {
            Log::info("[ child ] 收到连接准备命令");
            emit_connection_event(
                ConnectionStatusKind::DeviceChecking,
                Some("正在准备设备连接（如需会启动模拟器）".to_string()),
            );

            let device_config = device_ctx.device_config.read().await.clone();
            if matches!(device_config.platform, DevicePlatform::Desktop) {
                emit_connection_event(
                    ConnectionStatusKind::DeviceConnected,
                    Some("Desktop 平台无需 ADB 连接".to_string()),
                );
                return;
            }

            match ensure_device_connection_with_progress(
                &device_config,
                emit_device_connection_status,
            )
            .await
            {
                Ok(runtime_connect) => match ADBCtx::new(runtime_connect).await {
                    Ok(()) => emit_connection_event(
                        ConnectionStatusKind::DeviceConnected,
                        Some("[ child ] 模拟器启动后再次尝试连接成功".to_string()),
                    ),
                    Err(error) => {
                        Log::warn(&format!("[ child ] ADB上下文连接初始化失败: {}", error));
                        emit_connection_event(
                            ConnectionStatusKind::DeviceDisconnected,
                            Some(error),
                        );
                    }
                },
                Err(error) => {
                    Log::warn(&format!("[ child ] 设备连接准备失败: {}", error));
                    emit_connection_event(ConnectionStatusKind::DeviceDisconnected, Some(error));
                }
            }
        }
    }
}

fn emit_device_connection_status(status: ConnectionStatusKind, message: String) {
    emit_connection_event(status, Some(message));
}

async fn handle_capture_control(
    request_id: crate::infrastructure::core::MessageId,
    _control: CaptureControlMessage,
) {
    Log::info("[ child ] 收到设备截图命令");
    let Some(device_ctx) = try_get_device_ctx() else {
        emit_capture_event(
            request_id,
            None,
            Some("[ child ] 设备上下文尚未初始化，请稍后重试".to_string()),
        );
        return;
    };

    if !device_ctx.valid_capture().await {
        emit_capture_event(
            request_id,
            None,
            Some("[ child ] 设备截图校验失败：请检查截图方式、窗口状态或设备连接".to_string()),
        );
        return;
    }

    match device_ctx.get_screenshot_result().await {
        Ok(image) => match dynamic_image_to_base64(&DynamicImage::ImageRgba8(image)) {
            Ok(image_data) => emit_capture_event(request_id, Some(image_data), None),
            Err(error) => emit_capture_event(
                request_id,
                None,
                Some(format!("[ child ] 设备截图编码失败：{}", error)),
            ),
        },
        Err(error) => emit_capture_event(request_id, None, Some(error)),
    }
}

async fn handle_process_control(ctrl: ProcessControlMessage) {
    match ctrl.action {
        ProcessAction::Start => {
            Log::info("[ child ] 收到启动命令");
            clear_stop_request();
            set_running_status(RunningStatus::Running);
            let _ = emit_lifecycle_event_now(RuntimeLifecyclePhase::Running, None).await;
            // TODO: 第二阶段后续 - 通知调度器开始执行
        }
        ProcessAction::Stop => {
            Log::info("[ child ] 收到停止命令，停止当前脚本执行");
            request_stop_execution();
            set_running_status(RunningStatus::Stopping);
            emit_progress_event(
                RuntimeProgressPhase::Stopping,
                None,
                None,
                None,
                None,
                Some("收到停止命令，正在停止当前执行".to_string()),
            );
            let _ = emit_lifecycle_event_now(
                RuntimeLifecyclePhase::Stopping,
                Some("收到停止命令，正在停止当前执行".to_string()),
            )
            .await;
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
                Some("[ child ] 执行已暂停".to_string()),
            );
            let _ = emit_lifecycle_event_now(RuntimeLifecyclePhase::Paused, None).await;
        }
        ProcessAction::Shutdown => {
            Log::info("[ child ] 收到关闭命令，准备退出");
            set_running_status(RunningStatus::Stopping);
            let _ = emit_lifecycle_event_now(RuntimeLifecyclePhase::Stopping, None).await;
            trigger_cancel(); // 取消 CancellationToken，主循环立即退出
                              // TODO: 持久化运行时数据
        }
    }
}

async fn handle_session_control(control: SessionControlMessage) {
    use crate::infrastructure::scripts::scheduler::get_scheduler;

    match control {
        SessionControlMessage::LoadSession { session } => {
            clear_stop_request();
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
            let _ = emit_lifecycle_event_with_now(
                RuntimeLifecyclePhase::Loaded,
                Some(summary.session_id),
                None,
                Some("运行会话已加载".to_string()),
            )
            .await;
            let _ = emit_lifecycle_event_with_now(
                RuntimeLifecyclePhase::Idle,
                Some(summary.session_id),
                None,
                Some("设备待命，等待执行命令".to_string()),
            )
            .await;
        }
        SessionControlMessage::ReloadSession { session } => {
            clear_stop_request();
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
            let _ = emit_lifecycle_event_with_now(
                RuntimeLifecyclePhase::Loaded,
                Some(summary.session_id),
                None,
                Some("运行会话已热更新".to_string()),
            )
            .await;
        }
        SessionControlMessage::ClearSession => {
            clear_stop_request();
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
            let _ = emit_lifecycle_event_with_now(
                RuntimeLifecyclePhase::Idle,
                cleared.map(|summary| summary.session_id),
                None,
                Some("运行会话已清空".to_string()),
            )
            .await;
        }
    }
}

async fn handle_config_update(config: ConfigUpdateMessage) {
    let next_config = match serde_json::from_str::<DeviceConfig>(&config.device_config_json) {
        Ok(config) => config,
        Err(error) => {
            Log::error(&format!("[ child ] 设备配置热更新反序列化失败: {}", error));
            return;
        }
    };

    if let Some(client) = get_ipc_client() {
        client
            .log_level
            .store(next_config.log_level.clone() as u8, Ordering::Relaxed);
    }

    let Some(device_ctx) = try_get_device_ctx() else {
        Log::warn("[ child ] 设备上下文尚未初始化，忽略本次设备配置热更新");
        return;
    };

    device_ctx.apply_device_config(next_config.clone()).await;

    if matches!(next_config.platform, DevicePlatform::Android) {
        match resolve_runtime_connect_config(&next_config) {
            Ok(runtime_connect) => {
                if let Err(error) = ADBCtx::new(runtime_connect).await {
                    let message = format!(
                        "[ child ] 设备配置已更新，但 ADB 连接配置生效失败: {}",
                        error
                    );
                    Log::warn(&message);
                    emit_connection_event(ConnectionStatusKind::DeviceDisconnected, Some(message));
                }
            }
            Err(error) => {
                Log::warn(&format!(
                    "[ child ] 设备配置已更新，但 ADB 连接配置未生效: {}",
                    error
                ));
            }
        }
    }

    Log::info(&format!("[ child ] 设备配置已热更新: {:?}", next_config));
}
