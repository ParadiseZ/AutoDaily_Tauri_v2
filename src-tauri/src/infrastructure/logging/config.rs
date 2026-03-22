use crate::infrastructure::core::{Deserialize, Serialize};
use std::fmt;
use crate::infrastructure::logging::LogLevel;

/// 主进程日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMain {
    /// 主进程日志级别
    pub log_level: LogLevel,
    /// 日志文件目录（相对路径或绝对路径）
    pub log_dir: String,
    /// 日志保留天数（超过此天数的日志文件将被自动清理）
    pub retention_days: u32,
}

impl Default for LogMain {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            log_dir: "logs".to_string(),
            retention_days: 7,
        }
    }
}

impl fmt::Display for LogMain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "loglevel:{}, log_dir:{}, retention_days:{}day",
            self.log_level, self.log_dir, self.retention_days
        )
    }
}
