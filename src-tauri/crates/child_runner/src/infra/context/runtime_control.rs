//! Child-process IPC, cancellation, scheduler, and stop-control state.

pub(crate) use super::RunningStatus;
use super::{ChildRuntimeInitError, ChildRuntimeInitResult};
use crate::infra::ipc::channel_client::IpcClient;
use crate::infra::logging::LogLevel;
use crate::infra::scripts::scheduler::ScriptScheduler;
use ad_kernel::ids::DeviceId;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

static RUNNING: AtomicU8 = AtomicU8::new(RunningStatus::Unknown as u8);
static IPC_CLIENT: OnceLock<Arc<IpcClient>> = OnceLock::new();
static IPC_CLIENT_STARTED: AtomicBool = AtomicBool::new(false);
static CANCEL_TOKEN: OnceLock<tokio_util::sync::CancellationToken> = OnceLock::new();
static SCHEDULER: OnceLock<Arc<ScriptScheduler>> = OnceLock::new();
static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_running_status(status: RunningStatus) {
    RUNNING.store(status as u8, Ordering::Relaxed);
}

pub(crate) fn get_running_status() -> RunningStatus {
    RunningStatus::from(RUNNING.load(Ordering::Acquire))
}

pub(crate) fn process_need_stop() -> bool {
    matches!(RUNNING.load(Ordering::Acquire), 4 | 5)
}

pub(crate) fn init_ipc_client(
    device_id: Arc<DeviceId>,
    log_level: LogLevel,
) -> ChildRuntimeInitResult<()> {
    let manager = Arc::new(IpcClient::new(device_id, AtomicU8::from(log_level as u8)));
    IPC_CLIENT
        .set(manager)
        .map_err(|e| ChildRuntimeInitError::InitChildIpcClientFailed {
            e: e.clone().to_string(),
        })
}

pub(crate) fn start_ipc_client() -> ChildRuntimeInitResult<()> {
    let manager = IPC_CLIENT.get().cloned().ok_or_else(|| {
        ChildRuntimeInitError::InitChildIpcClientFailed {
            e: "IPC 客户端尚未初始化".to_string(),
        }
    })?;
    if IPC_CLIENT_STARTED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        manager.spawn_reconnect_task();
    }
    Ok(())
}

pub(crate) fn get_ipc_client() -> Option<Arc<IpcClient>> {
    IPC_CLIENT.get().cloned()
}

pub(crate) fn init_cancel_token(token: tokio_util::sync::CancellationToken) {
    let _ = CANCEL_TOKEN.set(token);
}

pub(crate) fn init_scheduler(
    cancel_token: tokio_util::sync::CancellationToken,
) -> ChildRuntimeInitResult<Arc<ScriptScheduler>> {
    let scheduler = ScriptScheduler::new(cancel_token);
    SCHEDULER
        .set(scheduler.clone())
        .map_err(|_| ChildRuntimeInitError::InitChildAppCtxFailed {
            e: "ScriptScheduler already initialized".to_string(),
        })?;
    Ok(scheduler)
}

pub(crate) fn get_scheduler() -> Option<Arc<ScriptScheduler>> {
    SCHEDULER.get().cloned()
}

pub(crate) fn trigger_cancel() {
    if let Some(token) = CANCEL_TOKEN.get() {
        token.cancel();
    }
}

pub(crate) fn request_stop_execution() {
    STOP_REQUESTED.store(true, Ordering::Release);
}

pub(crate) fn clear_stop_request() {
    STOP_REQUESTED.store(false, Ordering::Release);
}

pub(crate) fn stop_requested() -> bool {
    STOP_REQUESTED.load(Ordering::Acquire)
}
