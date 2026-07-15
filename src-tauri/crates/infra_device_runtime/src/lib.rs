mod device_ctx;
mod device_launcher;
mod device_runtime;

pub use device_ctx::{DeviceCtx, get_device_ctx, init_device_ctx, try_get_device_ctx};
pub use device_launcher::{
    ensure_device_connection_with_progress, probe_device_config_connection_with_timeout,
    resolve_runtime_connect_config,
};

pub(crate) struct Log;

impl Log {
    pub(crate) fn debug(message: &str) {
        tracing::debug!("{}", message);
    }
    pub(crate) fn info(message: &str) {
        tracing::info!("{}", message);
    }
    pub(crate) fn warn(message: &str) {
        tracing::warn!("{}", message);
    }
    pub(crate) fn error(message: &str) {
        tracing::error!("{}", message);
    }
}
