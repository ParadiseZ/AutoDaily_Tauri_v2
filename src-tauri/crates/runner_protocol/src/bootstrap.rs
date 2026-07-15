use ad_kernel::LogLevel;
use ad_kernel::ids::DeviceId;
use domain_device::DeviceConfig;
use domain_vision::VisionTextCacheRuntimeConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProcessInitData {
    pub device_id: DeviceId,
    pub device_config: DeviceConfig,
    pub shm_name: String,
    pub log_level: LogLevel,
    pub cpu_cores: Vec<usize>,
    pub db_path: PathBuf,
    pub vision_text_cache_config: VisionTextCacheRuntimeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_the_device_configuration_in_bootstrap_data() {
        let init_data = ChildProcessInitData {
            device_id: DeviceId::new_v7(),
            device_config: DeviceConfig::default(),
            shm_name: "runner".to_string(),
            log_level: LogLevel::Off,
            cpu_cores: vec![0],
            db_path: PathBuf::from("data"),
            vision_text_cache_config: VisionTextCacheRuntimeConfig::default(),
        };

        assert_eq!(init_data.device_config.cores, vec![0, 1]);
        assert_eq!(init_data.db_path, PathBuf::from("data"));
    }
}
