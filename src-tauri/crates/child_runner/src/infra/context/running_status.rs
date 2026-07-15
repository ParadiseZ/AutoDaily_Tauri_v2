use serde::{Deserialize, Serialize};

/// child 进程执行循环的本地运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub(crate) enum RunningStatus {
    Unknown = 0,
    Idle = 1,
    Running = 2,
    Paused = 3,
    Stopped = 4,
    Error = 5,
    Stopping = 6,
}

impl From<u8> for RunningStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Idle,
            2 => Self::Running,
            3 => Self::Paused,
            4 => Self::Stopped,
            5 => Self::Error,
            6 => Self::Stopping,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunningStatus;

    #[test]
    fn maps_unknown_running_status() {
        assert_eq!(RunningStatus::from(2), RunningStatus::Running);
        assert_eq!(RunningStatus::from(99), RunningStatus::Unknown);
    }
}
