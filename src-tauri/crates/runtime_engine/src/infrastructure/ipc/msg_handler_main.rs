use crate::constant::project::MAIN_WINDOW;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::ipc::message::{IpcMessage, MessagePayload};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::logging::main_process_log_handler::get_child_log_receiver;
use tauri::{Emitter, Manager};

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
        MessagePayload::StatusReport(ref report) => {
            // 状态报告：转发到前端
            Log::info(&format!(
                "[ ipc ] 设备[{}]状态: {:?}",
                device_id, report.status
            ));
            if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
                let emit_data = serde_json::json!({
                    "deviceId": device_id.to_string(),
                    "status": format!("{:?}", report.status),
                    "currentScript": report.current_script.map(|s| s.to_string()),
                    "message": report.message,
                });
                let _ = main_window.emit("device-status", emit_data);
            }
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
