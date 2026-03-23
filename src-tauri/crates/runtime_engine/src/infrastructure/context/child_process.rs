use crate::domain::devices::device_conf::DeviceConfig;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::logging::LogLevel;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 子进程初始化数据（可序列化，仅承载描述信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProcessInitData {
    pub device_id: DeviceId,
    pub device_config: DeviceConfig,
    pub shm_name: String,
    pub log_level: LogLevel,
    pub cpu_cores: Vec<usize>,
    pub db_path: PathBuf,
}

impl ChildProcessInitData {
    pub fn init_ort_env() -> bool {
        ort::init().with_telemetry(false).commit()
    }
}
