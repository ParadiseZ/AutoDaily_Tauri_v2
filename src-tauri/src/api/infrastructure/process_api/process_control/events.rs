use crate::constant::project::MAIN_WINDOW;
use crate::domain::devices::device_runtime_event::{
    DeviceAssignmentScheduleChangedEventPayload, DeviceConnectionEventPayload,
    DeviceLifecycleStatus, DeviceProgressEventPayload, DeviceRuntimeProgressPhase,
    DeviceRuntimeReconcileAction, DeviceRuntimeReconcileEventPayload,
    DeviceRuntimeReconcileJobType, DeviceRuntimeReconcilePhase, DeviceStatusEventPayload,
};
use crate::infrastructure::context::main_process::{MainProcessCtx, RuntimeReconcileJob};
use crate::infrastructure::core::{now_millis_string, DeviceId};
use crate::infrastructure::ipc::message::ConnectionStatusKind;
use crate::infrastructure::logging::log_trait::Log;
use chrono::Local;
use tauri::{Emitter, Manager};

const DEVICE_RUNTIME_RECONCILE_EVENT: &str = "device-runtime-reconcile";
const ASSIGNMENT_SCHEDULE_CHANGED_EVENT: &str = "assignment-schedule-changed";

pub(crate) fn device_log_label(app_handle: &tauri::AppHandle, device_id: DeviceId) -> String {
    app_handle
        .state::<MainProcessCtx>()
        .snapshot_device_runtime_state(device_id)
        .ok()
        .and_then(|state| {
            state
                .device_name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| device_id.to_string())
}

fn format_progress_log_detail(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Some(task_name) = trimmed.strip_prefix("开始执行任务:") {
        return format!(", 任务:{}", task_name.trim());
    }
    if let Some(step_name) = trimmed.strip_prefix("开始执行步骤:") {
        return format!(", 步骤:{}", step_name.trim());
    }
    if let Some(task_name) = trimmed.strip_prefix("任务执行完成:") {
        return format!(", 任务完成:{}", task_name.trim());
    }

    format!(", 消息:{}", trimmed)
}

pub(super) fn emit_device_connection_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    status: &ConnectionStatusKind,
    message: Option<&str>,
) {
    let device_label = device_log_label(app_handle, device_id);
    Log::info(&format!(
        "[ process ] 设备[{}]连接状态: {:?}{}",
        device_label,
        status,
        message
            .map(|value| format!("，{}", value))
            .unwrap_or_default()
    ));
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let payload = DeviceConnectionEventPayload {
            device_id,
            status: status.clone(),
            message: message.map(str::to_string),
            at: now_millis_string(),
        };
        let _ = main_window.emit("device-connection-status", payload);
    }
}

pub(super) fn emit_device_progress_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    phase: DeviceRuntimeProgressPhase,
    message: impl Into<String>,
) {
    let message = message.into();
    let at = Local::now().to_rfc3339();
    let device_label = device_log_label(app_handle, device_id);
    Log::info(&format!(
        "[ process ] 设备[{}]进度: {:?}{}",
        device_label,
        phase,
        format_progress_log_detail(&message)
    ));
    let _ = app_handle.state::<MainProcessCtx>().set_device_progress(
        device_id,
        serde_json::to_value(&phase)
            .ok()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_default(),
        message.clone(),
        Some(at.clone()),
    );
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let payload = DeviceProgressEventPayload {
            device_id,
            session_id: None,
            assignment_id: None,
            script_id: None,
            task_id: None,
            step_id: None,
            phase,
            message: Some(message),
            at,
        };
        let _ = main_window.emit("device-progress", payload);
    }
}

pub(super) fn emit_device_lifecycle_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    status: DeviceLifecycleStatus,
    message: impl Into<String>,
) {
    let message = message.into();
    let at = now_millis_string();
    let device_label = device_log_label(app_handle, device_id);
    Log::info(&format!(
        "[ process ] 设备[{}]生命周期: {:?}{}",
        device_label,
        status,
        if message.trim().is_empty() {
            String::new()
        } else {
            format!("，{}", message)
        }
    ));
    let _ = app_handle.state::<MainProcessCtx>().set_device_lifecycle(
        device_id,
        status.clone(),
        None,
        Some(message.clone()),
        Some(at.clone()),
    );
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let payload = DeviceStatusEventPayload {
            device_id,
            session_id: None,
            status,
            current_script_id: None,
            message: Some(message),
            at,
        };
        let _ = main_window.emit("device-status", payload);
    }
}

pub(crate) fn emit_assignment_schedule_changed(app_handle: &tauri::AppHandle, device_id: DeviceId) {
    let payload = DeviceAssignmentScheduleChangedEventPayload {
        device_id,
        at: Local::now().to_rfc3339(),
    };
    let _ = app_handle.emit(ASSIGNMENT_SCHEDULE_CHANGED_EVENT, payload);
}

pub(super) fn emit_runtime_reconcile_event(
    app_handle: &tauri::AppHandle,
    job: &RuntimeReconcileJob,
    phase: DeviceRuntimeReconcilePhase,
    action: Option<DeviceRuntimeReconcileAction>,
    message: Option<String>,
) {
    let payload = DeviceRuntimeReconcileEventPayload {
        job_id: job.job_id().to_string(),
        job_type: match job.job_type() {
            "deviceConfig" => DeviceRuntimeReconcileJobType::DeviceConfig,
            "deviceSessionRefresh" => DeviceRuntimeReconcileJobType::DeviceSessionRefresh,
            _ => DeviceRuntimeReconcileJobType::DeviceSessionRefresh,
        },
        device_id: job.device_id(),
        phase,
        action,
        message,
        at: Local::now().to_rfc3339(),
    };
    let _ = app_handle.emit(DEVICE_RUNTIME_RECONCILE_EVENT, payload);
}
