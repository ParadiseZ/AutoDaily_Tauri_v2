use crate::constant::project::MAIN_WINDOW;
use crate::domain::devices::device_runtime_event::{
    DeviceAssignmentScheduleChangedEventPayload, DeviceConnectionEventPayload,
    DeviceProgressEventPayload, DeviceRuntimeProgressPhase, DeviceRuntimeReconcileAction,
    DeviceRuntimeReconcileEventPayload, DeviceRuntimeReconcileJobType, DeviceRuntimeReconcilePhase,
};
use crate::infrastructure::context::main_process::{MainProcessCtx, RuntimeReconcileJob};
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::message::ConnectionStatusKind;
use chrono::Local;
use tauri::{Emitter, Manager};

const DEVICE_RUNTIME_RECONCILE_EVENT: &str = "device-runtime-reconcile";
const ASSIGNMENT_SCHEDULE_CHANGED_EVENT: &str = "assignment-schedule-changed";

fn now_millis_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub(super) fn emit_device_connection_status(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    status: &ConnectionStatusKind,
    message: Option<&str>,
) {
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
