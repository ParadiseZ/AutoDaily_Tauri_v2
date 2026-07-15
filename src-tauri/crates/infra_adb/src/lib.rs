mod adb_command;
mod adb_config;
mod adb_context;
mod adb_error;
mod adb_executor;

pub use adb_command::ADBCommand;
pub use adb_config::{ADBConnectConfig, AdbServeByIdentifier, AdbServerConfig};
pub use adb_context::{ADBCtx, try_get_adb_ctx};

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
