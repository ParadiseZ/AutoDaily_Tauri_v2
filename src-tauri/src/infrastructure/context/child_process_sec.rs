use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::core::{Deserialize, DeviceId, Serialize};
use crate::infrastructure::ipc::chanel_client::IpcClient;
use crate::infrastructure::logging::LogLevel;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

/// 运行标志
static RUNNING: AtomicU8 = AtomicU8::new(0);
#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum RunningStatus{
    Idle = 1,
    Running = 2,
    Paused = 3,
    Stopped = 4,
    Error = 5,
    Unknown = 0,
}

/// ipc/子进程运行状态
impl From<u8> for RunningStatus {
    fn from(v: u8) -> Self {
        match v {
            1 => RunningStatus::Idle,
            2 => RunningStatus::Running,
            3 => RunningStatus::Paused,
            4 => RunningStatus::Stopped,
            5 => RunningStatus::Error,
            _ => RunningStatus::Unknown,
        }
    }
}
pub fn is_idle() -> bool {
    RUNNING.load(Ordering::Acquire) == 1u8
}
pub fn set_running_status(status: RunningStatus) {
    RUNNING.store(status as u8, Ordering::Relaxed);
}
pub fn get_running_status() -> RunningStatus {
    RunningStatus::from(RUNNING.load(Ordering::Acquire))
}
pub fn process_need_stop() ->bool{
    matches!(RUNNING.load(Ordering::Acquire), 4 | 5) // Stopped=4, Error=5
}

/// InitData 类型移动到 context::child_process 模块


/// IPC客户端
static IPC_CLIENT: OnceLock<Arc<IpcClient>> = OnceLock::new();

pub fn init_ipc_client(device_id: Arc<DeviceId>, log_level: LogLevel) -> InitResult<()> {
    let manager = Arc::new(IpcClient::new(device_id, AtomicU8::from(log_level as u8)));
    manager.spawn_reconnect_task();
    IPC_CLIENT.set(manager).map_err(|e| InitError::InitChildIpcClientFailed {e: e.to_string()})
}
pub fn get_ipc_client() -> Arc<IpcClient> {
    IPC_CLIENT.get().unwrap().clone()
}