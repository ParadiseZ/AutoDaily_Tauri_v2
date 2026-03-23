use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum RunningStatus {
    Idle = 1,
    Running = 2,
    Paused = 3,
    Stopped = 4,
    Error = 5,
    Stopping = 6,
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
            6 => RunningStatus::Stopping,
            _ => RunningStatus::Unknown,
        }
    }
}
