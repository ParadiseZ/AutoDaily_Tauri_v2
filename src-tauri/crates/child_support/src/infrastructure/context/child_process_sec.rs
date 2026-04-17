use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::chanel_client::IpcClient;
use crate::infrastructure::logging::LogLevel;
pub use runtime_engine::infrastructure::context::child_process_sec::RunningStatus;
use runtime_engine::infrastructure::context::init_error::{InitError, InitResult};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

static RUNNING: AtomicU8 = AtomicU8::new(RunningStatus::Unknown as u8);
static IPC_CLIENT: OnceLock<Arc<IpcClient>> = OnceLock::new();
static CANCEL_TOKEN: OnceLock<tokio_util::sync::CancellationToken> = OnceLock::new();

pub fn is_idle() -> bool {
    RUNNING.load(Ordering::Acquire) == RunningStatus::Idle as u8
}

pub fn set_running_status(status: RunningStatus) {
    RUNNING.store(status as u8, Ordering::Relaxed);
}

pub fn get_running_status() -> RunningStatus {
    RunningStatus::from(RUNNING.load(Ordering::Acquire))
}

pub fn process_need_stop() -> bool {
    matches!(RUNNING.load(Ordering::Acquire), 4 | 5 | 6)
}

pub fn init_ipc_client(device_id: Arc<DeviceId>, log_level: LogLevel) -> InitResult<()> {
    let manager = Arc::new(IpcClient::new(device_id, AtomicU8::from(log_level as u8)));
    manager.clone().spawn_reconnect_task();
    IPC_CLIENT
        .set(manager)
        .map_err(|e| InitError::InitChildIpcClientFailed {
            e: e.clone().to_string(),
        })
}

pub fn get_ipc_client() -> Option<Arc<IpcClient>> {
    IPC_CLIENT.get().cloned()
}

pub fn init_cancel_token(token: tokio_util::sync::CancellationToken) {
    let _ = CANCEL_TOKEN.set(token);
}

pub fn get_cancel_token() -> Option<&'static tokio_util::sync::CancellationToken> {
    CANCEL_TOKEN.get()
}

pub fn trigger_cancel() {
    if let Some(token) = CANCEL_TOKEN.get() {
        token.cancel();
    }
}
