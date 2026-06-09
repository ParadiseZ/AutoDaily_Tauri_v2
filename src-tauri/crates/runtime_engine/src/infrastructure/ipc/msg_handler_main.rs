use crate::constant::project::MAIN_WINDOW;
use crate::domain::devices::device_runtime_event::{
    DeviceConnectionEventPayload, DeviceLifecycleStatus, DeviceProgressEventPayload,
    DeviceRuntimeProgressPhase, DeviceScheduleEventPayload, DeviceStatusEventPayload,
    DeviceTimeoutEventPayload,
};
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::context::main_process::{
    DeviceCaptureResult, DeviceDispatchSignal, MainProcessCtx,
};
use crate::infrastructure::ipc::message::IpcMessage;
use crate::infrastructure::ipc::message::MessagePayload;
use crate::infrastructure::ipc::message::{RuntimeEventMessage, TimeoutAction};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::logging::main_process_log_handler::get_child_log_receiver;
use crate::infrastructure::logging::LogLevel;
use crate::infrastructure::mail::{
    load_email_config, send_timeout_email_in_background, EmailMessagePayload,
};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

fn normalize_timeout_meta_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "<none>" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_timeout_message(
    message: &str,
) -> (
    Option<TimeoutAction>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let segments = message
        .split(';')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return (None, None, None, Some(message.trim().to_string()));
    }

    let mut timeout_action = None;
    let mut page_fingerprint = None;
    let mut action_signature = None;
    for segment in &segments {
        let Some((key, value)) = segment.split_once('=') else {
            continue;
        };
        match key.trim() {
            "action" => {
                timeout_action = match value.trim() {
                    "StopExecution" | "stopExecution" => Some(TimeoutAction::StopExecution),
                    "RunRecoveryTask" | "runRecoveryTask" => Some(TimeoutAction::RunRecoveryTask),
                    "SkipCurrentTask" | "skipCurrentTask" => Some(TimeoutAction::SkipCurrentTask),
                    _ => None,
                };
            }
            "page" => page_fingerprint = normalize_timeout_meta_value(value),
            "signature" => action_signature = normalize_timeout_meta_value(value),
            _ => {}
        }
    }

    let detail = if segments.len() > 3 {
        Some(segments[3..].join("; "))
    } else {
        None
    };
    (timeout_action, page_fingerprint, action_signature, detail)
}

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
                let emit_data = DeviceStatusEventPayload {
                    device_id,
                    session_id: lifecycle.session_id,
                    status: DeviceLifecycleStatus::from(lifecycle.phase.clone()),
                    current_script_id: lifecycle.current_script_id,
                    message: lifecycle.message.clone(),
                    at: lifecycle.at.clone(),
                };
                let _ = main_window.emit("device-status", emit_data);
            }
            RuntimeEventMessage::Progress(progress) => {
                let _ = get_app_handle()
                    .state::<MainProcessCtx>()
                    .set_device_progress(
                        device_id,
                        serde_json::to_value(&DeviceRuntimeProgressPhase::from(
                            progress.phase.clone(),
                        ))
                        .ok()
                        .and_then(|value| value.as_str().map(str::to_string))
                        .unwrap_or_default(),
                        progress.message.clone().unwrap_or_default(),
                        Some(progress.at.clone()),
                    );
                let emit_data = DeviceProgressEventPayload {
                    device_id,
                    session_id: progress.session_id,
                    assignment_id: progress.assignment_id,
                    script_id: progress.script_id,
                    task_id: progress.task_id,
                    step_id: progress.step_id,
                    phase: progress.phase.clone().into(),
                    message: progress.message.clone(),
                    at: progress.at.clone(),
                };
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
                        let (timeout_action, page_fingerprint, action_signature, detail) =
                            parse_timeout_message(body.trim());
                        let emit_data = DeviceTimeoutEventPayload {
                            device_id,
                            session_id: progress.session_id,
                            assignment_id: progress.assignment_id,
                            script_id: progress.script_id,
                            task_id: progress.task_id,
                            step_id: progress.step_id,
                            timeout_action,
                            page_fingerprint,
                            action_signature,
                            detail,
                            message: body.trim().to_string(),
                            at: progress.at.clone(),
                        };
                        let _ = main_window.emit("device-timeout", emit_data);
                    }
                }
            }
            RuntimeEventMessage::Schedule(schedule) => {
                let emit_data = DeviceScheduleEventPayload {
                    device_id,
                    session_id: schedule.session_id,
                    execution_id: schedule.execution_id,
                    assignment_id: schedule.assignment_id,
                    script_id: schedule.script_id,
                    task_id: schedule.task_id,
                    step_id: schedule.step_id,
                    status: schedule.status.clone(),
                    message: schedule.message.clone(),
                    at: schedule.at.clone(),
                };
                let _ = main_window.emit("device-schedule", emit_data);
            }
            RuntimeEventMessage::Connection(connection) => {
                let _ = get_app_handle()
                    .state::<MainProcessCtx>()
                    .set_device_connection_state(
                        device_id,
                        connection.status.clone(),
                        connection.message.clone(),
                    );
                let _ = get_app_handle()
                    .state::<MainProcessCtx>()
                    .set_device_progress(
                        device_id,
                        serde_json::to_value(&DeviceRuntimeProgressPhase::from(
                            connection.status.clone(),
                        ))
                        .ok()
                        .and_then(|value| value.as_str().map(str::to_string))
                        .unwrap_or_default(),
                        connection.message.clone().unwrap_or_default(),
                        Some(connection.at.clone()),
                    );

                if let (Some(message), Some(receiver)) =
                    (connection.message.as_ref(), get_child_log_receiver())
                {
                    let level = match connection.status {
                        crate::infrastructure::ipc::message::ConnectionStatusKind::DeviceConnected => {
                            LogLevel::Info
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::DeviceChecking => {
                            LogLevel::Info
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::ShellProbeChecking
                        | crate::infrastructure::ipc::message::ConnectionStatusKind::EmulatorStarting
                        | crate::infrastructure::ipc::message::ConnectionStatusKind::EmulatorWaiting => {
                            LogLevel::Info
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::DeviceDisconnected => {
                            LogLevel::Warn
                        }
                        crate::infrastructure::ipc::message::ConnectionStatusKind::DeviceUnknown => {
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

                let emit_data = DeviceConnectionEventPayload {
                    device_id,
                    status: connection.status.clone(),
                    message: connection.message.clone(),
                    at: connection.at.clone(),
                };
                let _ = main_window.emit("device-connection-status", emit_data);
                let progress_data = DeviceProgressEventPayload {
                    device_id,
                    session_id: None,
                    assignment_id: None,
                    script_id: None,
                    task_id: None,
                    step_id: None,
                    phase: connection.status.clone().into(),
                    message: connection.message.clone(),
                    at: connection.at.clone(),
                };
                let _ = main_window.emit("device-progress", progress_data);
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
