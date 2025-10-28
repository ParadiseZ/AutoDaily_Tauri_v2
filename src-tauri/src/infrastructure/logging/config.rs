use crate::infrastructure::config::conf_write_guard::ConfigCategory;
use crate::infrastructure::core::{Deserialize, Serialize};
// 日志配置 - 临时实现
use crate::infrastructure::logging::logger::LogLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMain {
    pub log_level: LogLevel,
    pub log_dir: String,
    pub max_file_size: usize,
    pub retention_days: u32,
}

impl ConfigCategory for LogMain {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            log_dir: "logs".to_string(),
            max_file_size: 10240, // 10MB
            retention_days: 7,
        }
    }
}