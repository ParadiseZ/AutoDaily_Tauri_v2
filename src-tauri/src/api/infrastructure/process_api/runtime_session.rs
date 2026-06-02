mod access_control;
mod session_builder;

use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::context::child_process::ChildProcessInitData;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::message::{RunTarget, RuntimeQueueItem, RuntimeSessionSnapshot};

pub(super) async fn load_device_table(device_id: DeviceId) -> Result<DeviceTable, String> {
    session_builder::load_device_table(device_id).await
}

pub(super) fn validate_runtime_platform_supported(device_table: &DeviceTable) -> Result<(), String> {
    session_builder::validate_runtime_platform_supported(device_table)
}

pub(super) async fn load_runtime_session_for_target(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    run_target: RunTarget,
) -> Result<RuntimeSessionSnapshot, String> {
    session_builder::load_runtime_session_for_target(app_handle, device_id, run_target).await
}

pub(super) async fn load_runtime_session_for_queue_item(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    queue_item: RuntimeQueueItem,
) -> Result<RuntimeSessionSnapshot, String> {
    session_builder::load_runtime_session_for_queue_item(app_handle, device_id, queue_item).await
}

pub(super) async fn build_child_init_data(
    app_handle: &tauri::AppHandle,
    device_id: DeviceId,
    force_prepare_device: bool,
) -> Result<ChildProcessInitData, String> {
    session_builder::build_child_init_data(app_handle, device_id, force_prepare_device).await
}
