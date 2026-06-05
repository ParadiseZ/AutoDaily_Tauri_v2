use crate::constant::project::MAIN_WINDOW;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::context::main_process::{
    DeviceCaptureResult, DeviceConnectionState, DeviceDispatchSignal, MainProcessCtx,
};
use crate::infrastructure::ipc::message::IpcMessage;
use crate::infrastructure::ipc::message::MessagePayload;
use crate::infrastructure::ipc::message::RuntimeEventMessage;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::logging::main_process_log_handler::get_child_log_receiver;
use crate::infrastructure::logging::LogLevel;
use crate::infrastructure::mail::{
    load_email_config, send_timeout_email_in_background, EmailMessagePayload,
};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

/// 主进程消息处理器
/// 处理来自子进程的消息
pub async fn handle_child_message(msg: IpcMessage) {
    let device_id = msg.source_or_target;

    match msg.payload {
        MessagePayload::Logger(ref log_msg) => {
            // 子进程日志：写入文件 + emit 前端
            if let Some(receiver) = get_child_log_receiver() {
                receiver.handle_log(&device_id, log_msg).await;
            }
            // emit 到前端
            if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "level": format!("{}", log_msg.level),
                    "message": log_msg.message,
                    "time": chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
                });
                let _ = main_window.emit("child-log", emit_data);
            }
        }
        MessagePayload::Heartbeat(_) => {
            // 心跳消息：更新最后心跳时间
            // TODO: 更新 IpcClientState.last_heartbeat
        }
        MessagePayload::RuntimeEvent(ref event) => {
            handle_runtime_event(device_id, event);
        }
        MessagePayload::Error(ref error) => {
            Log::error(&format!(
                "[ ipc ] 设备[{}]错误: [{}] {}",
                device_id, error.code, error.message
            ));
            if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "code": error.code,
                    "message": error.message,
                    "details": error.details,
                });
                let _ = main_window.emit("device-error", emit_data);
            }
        }
        _ => {}
    }
}

fn handle_runtime_event(
    device_id: crate::infrastructure::core::DeviceId,
    event: &RuntimeEventMessage,
) {
    if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
        match event {
            RuntimeEventMessage::Lifecycle(lifecycle) => {
                Log::info(&format!(
                    "[ ipc ] 设备[{}]生命周期: {:?}",
                    device_id, lifecycle.phase
                ));
                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "sessionId": lifecycle.session_id.map(|id| id.to_string()),
                    "status": format!("{:?}", lifecycle.phase),
                    "currentScript": lifecycle.current_script_id.map(|id| id.to_string()),
                    "message": lifecycle.message,
                    "at": lifecycle.at,
                });
                let _ = main_window.emit("device-status", emit_data);
            }
            RuntimeEventMessage::Progress(progress) => {
                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "sessionId": progress.session_id.map(|id| id.to_string()),
                    "assignmentId": progress.assignment_id.map(|id| id.to_string()),
                    "scriptId": progress.script_id.map(|id| id.to_string()),
                    "taskId": progress.task_id.map(|id| id.to_string()),
                    "stepId": progress.step_id.map(|id| id.to_string()),
                    "phase": format!("{:?}", progress.phase),
                    "message": progress.message,
                    "at": progress.at,
                });
                let _ = main_window.emit("device-progress", emit_data);

                if let Some(message) = progress.message.as_deref() {
                    if let Some(body) = message.strip_prefix("[timeout_notify]") {
                        let app_handle = get_app_handle();
                        let desktop_notice_enabled = load_email_config(&app_handle)
                            .map(|config| config.desktop_notice)
                            .unwrap_or(true);

                        if desktop_notice_enabled {
                            let _ = app_handle
                                .notification()
                                .builder()
                                .title("脚本执行超时")
                                .body(body.trim())
                                .show();
                        }
                    }

                    if let Some(body) = message.strip_prefix("[timeout_email]") {
                        let email_body = format!(
                            "设备: {}\n时间: {}\n\n{}",
                            device_id,
                            progress.at,
                            body.trim()
                        );
                        send_timeout_email_in_background(
                            get_app_handle().clone(),
                            EmailMessagePayload {
                                subject: format!("AutoDaily 执行超时通知 - 设备 {}", device_id),
                                body: email_body,
                            },
                        );
                    }

                    if let Some(body) = message.strip_prefix("[timeout]") {
                        let emit_data = serde_json::json!({
                            "deviceId": device_id.to_string(),
                            "sessionId": progress.session_id.map(|id| id.to_string()),
                            "assignmentId": progress.assignment_id.map(|id| id.to_string()),
                            "scriptId": progress.script_id.map(|id| id.to_string()),
                            "taskId": progress.task_id.map(|id| id.to_string()),
                            "stepId": progress.step_id.map(|id| id.to_string()),
                            "message": body.trim(),
                            "at": progress.at,
                        });
                        let _ = main_window.emit("device-timeout", emit_data);
                    }
                }
            }
            RuntimeEventMessage::Schedule(schedule) => {
                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "sessionId": schedule.session_id.map(|id| id.to_string()),
                    "executionId": schedule.execution_id.map(|id| id.to_string()),
                    "assignmentId": schedule.assignment_id.map(|id| id.to_string()),
                    "scriptId": schedule.script_id.map(|id| id.to_string()),
                    "taskId": schedule.task_id.map(|id| id.to_string()),
                    "stepId": schedule.step_id.map(|id| id.to_string()),
                    "status": format!("{:?}", schedule.status),
                    "message": schedule.message,
                    "at": schedule.at,
                });
                let _ = main_window.emit("device-schedule", emit_data);
            }
            RuntimeEventMessage::Connection(connection) => {
                if let Ok(mut guard) = get_app_handle()
                    .state::<MainProcessCtx>()
                    .device_connections
                    .write()
                {
                    guard.insert(
                        device_id,
                        DeviceConnectionState {
                            status: connection.status.clone(),
                            message: connection.message.clone(),
                        },
                    );
                }

                if let (Some(message), Some(receiver)) =
                    (connection.message.as_ref(), get_child_log_receiver())
                {
                    let level = match connection.status {
                        crate::infrastructure::ipc::message::ConnectionStatusKind::Connected => {
                            LogLevel::Info
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::Checking => {
                            LogLevel::Info
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::Disconnected => {
                            LogLevel::Warn
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::Unknown => {
                            LogLevel::Debug
                        }
                    };
                    let message = format!("[connection] {}", message);
                    tauri::async_runtime::spawn(async move {
                        receiver
                            .handle_log(
                                &device_id,
                                &crate::infrastructure::ipc::message::LogMessage {
                                    level,
                                    message,
                                    module: Some("connection".to_string()),
                                },
                            )
                            .await;
                    });
                }

                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "status": format!("{:?}", connection.status),
                    "message": connection.message,
                    "at": connection.at,
                });
                let _ = main_window.emit("device-connection-status", emit_data);
            }
            RuntimeEventMessage::Capture(capture) => {
                if let Ok(mut guard) = get_app_handle()
                    .state::<MainProcessCtx>()
                    .device_capture_results
                    .write()
                {
                    guard.insert(
                        capture.request_id,
                        DeviceCaptureResult {
                            device_id,
                            image_data: capture.image_data.clone(),
                            message: capture.message.clone(),
                        },
                    );
                }
            }
            RuntimeEventMessage::Dispatch(dispatch) => {
                let _ = get_app_handle()
                    .state::<MainProcessCtx>()
                    .dispatch_signal_tx
                    .send(DeviceDispatchSignal {
                        device_id,
                        dispatch_id: dispatch.dispatch_id,
                        assignment_id: dispatch.assignment_id,
                        script_id: dispatch.script_id,
                        phase: dispatch.phase.clone(),
                        message: dispatch.message.clone(),
                        at: dispatch.at.clone(),
                    });

                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "dispatchId": dispatch.dispatch_id.map(|id| id.to_string()),
                    "assignmentId": dispatch.assignment_id.map(|id| id.to_string()),
                    "scriptId": dispatch.script_id.map(|id| id.to_string()),
                    "phase": format!("{:?}", dispatch.phase),
                    "message": dispatch.message,
                    "at": dispatch.at,
                });
                let _ = main_window.emit("device-dispatch", emit_data);
            }
        }
    }
}
