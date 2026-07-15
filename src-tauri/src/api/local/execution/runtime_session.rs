mod access_control;
mod session_builder;

use ad_kernel::ids::DeviceId;
use domain_device::DeviceProfile;
use runner_protocol::ChildProcessInitData;
use runner_protocol::message::{RunTarget, RuntimeQueueItem, RuntimeSessionSnapshot};

pub(super) async fn load_device_profile(device_id: DeviceId) -> Result<DeviceProfile, String> {
    session_builder::load_device_profile(device_id).await
}

pub(super) fn validate_runtime_platform_supported(
    device_profile: &DeviceProfile,
) -> Result<(), String> {
    session_builder::validate_runtime_platform_supported(device_profile)
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
) -> Result<ChildProcessInitData, String> {
    session_builder::build_child_init_data(app_handle, device_id).await
}
