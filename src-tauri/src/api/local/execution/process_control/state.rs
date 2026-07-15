use crate::infra::context::main_process::{DeviceDispatchState, MainProcessCtx};
use ad_kernel::ids::{DeviceId, DispatchId};
use runner_protocol::message::RuntimeSessionSnapshot;
use std::sync::{Arc, OnceLock};
use tauri::{AppHandle, Manager};

static AUTO_DISPATCH_NOTIFY: OnceLock<Arc<tokio::sync::Notify>> = OnceLock::new();
static AUTO_DISPATCH_RESCHEDULE_NOTIFY: OnceLock<Arc<tokio::sync::Notify>> = OnceLock::new();

pub(super) fn ensure_device_dispatch_state(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .ensure_device_runtime_state(device_id)
}

pub(super) fn snapshot_device_dispatch_state(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<DeviceDispatchState, String> {
    app_handle
        .state::<MainProcessCtx>()
        .snapshot_device_dispatch_state(device_id)
}

pub(super) fn mark_active_dispatch(
    app_handle: &AppHandle,
    device_id: DeviceId,
    dispatch_id: Option<DispatchId>,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .mark_active_dispatch(device_id, dispatch_id)
}

pub(super) fn set_auto_dispatch_blocked(
    app_handle: &AppHandle,
    device_id: DeviceId,
    blocked: bool,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .set_auto_dispatch_blocked(device_id, blocked)
}

pub(super) fn push_debug_session(
    app_handle: &AppHandle,
    device_id: DeviceId,
    session: RuntimeSessionSnapshot,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .push_debug_session(device_id, session)
}

pub(super) fn pop_debug_session(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<Option<RuntimeSessionSnapshot>, String> {
    app_handle
        .state::<MainProcessCtx>()
        .pop_debug_session(device_id)
}

pub(super) fn reset_device_dispatch_state(
    app_handle: &AppHandle,
    device_id: DeviceId,
) -> Result<(), String> {
    app_handle
        .state::<MainProcessCtx>()
        .reset_device_dispatch_state(device_id)
}

pub(crate) fn notify_auto_dispatch_planner() {
    if let Some(notify) = AUTO_DISPATCH_NOTIFY.get() {
        notify.notify_one();
    }
}

pub(crate) fn notify_auto_dispatch_reschedule() {
    if let Some(notify) = AUTO_DISPATCH_RESCHEDULE_NOTIFY.get() {
        notify.notify_one();
    }
}

pub(super) fn auto_dispatch_notify() -> Arc<tokio::sync::Notify> {
    AUTO_DISPATCH_NOTIFY
        .get_or_init(|| Arc::new(tokio::sync::Notify::new()))
        .clone()
}

pub(super) fn auto_dispatch_reschedule_notify() -> Arc<tokio::sync::Notify> {
    AUTO_DISPATCH_RESCHEDULE_NOTIFY
        .get_or_init(|| Arc::new(tokio::sync::Notify::new()))
        .clone()
}
