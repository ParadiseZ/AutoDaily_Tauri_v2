use crate::infrastructure::core::time_format::LocalTimer;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::message::LogMessage;
use crate::infrastructure::logging::logger::LOG_DIR;
use chrono::Local;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

/// 子进程日志接收器
/// 负责将子进程通过 IPC 发送的日志写入对应设备的日志文件，
/// 并通过 tauri event emit 到前端供实时展示
pub struct ChildLogReceiver {
    /// 设备ID → (设备名称, 日志文件写入器)
    writers: RwLock<HashMap<DeviceId, DeviceLogWriter>>,
}

struct DeviceLogWriter {
    device_name: String,
    file: Option<std::fs::File>,
    current_date: String,
    log_dir: PathBuf,
}

impl DeviceLogWriter {
    fn new(device_name: String, log_dir: PathBuf) -> Self {
        let current_date = Local::now().format("%y%m%d").to_string();
        let mut writer = Self {
            device_name,
            file: None,
            current_date,
            log_dir,
        };
        writer.ensure_file();
        writer
    }

    /// 确保日志文件已打开，如果日期变了则创建新文件
    fn ensure_file(&mut self) {
        let today = Local::now().format("%y%m%d").to_string();
        if self.file.is_some() && self.current_date == today {
            return;
        }

        // 日期变了或文件未打开，创建/打开新文件
        self.current_date = today.clone();
        let filename = format!("{}_{}.log", self.device_name, today);
        let filepath = self.log_dir.join(&filename);

        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)
        {
            Ok(f) => {
                self.file = Some(f);
            }
            Err(e) => {
                tracing::warn!(
                    "打开设备[{}]日志文件失败: {} - {}",
                    self.device_name,
                    filepath.display(),
                    e
                );
                self.file = None;
            }
        }
    }

    /// 写入一条日志
    fn write_log(&mut self, log: &LogMessage) {
        self.ensure_file();
        if let Some(ref mut file) = self.file {
            let timestamp = Local::now().format("%m-%d %H:%M:%S%.3f");
            let line = format!("{} [{}] {}\n", timestamp, log.level, log.message);
            if let Err(e) = file.write_all(line.as_bytes()) {
                tracing::warn!("写入设备[{}]日志失败: {}", self.device_name, e);
            }
        }
    }
}

/// 全局唯一的 ChildLogReceiver 实例
static CHILD_LOG_RECEIVER: std::sync::OnceLock<Arc<ChildLogReceiver>> = std::sync::OnceLock::new();

/// 初始化全局 ChildLogReceiver
pub fn init_child_log_receiver() -> Arc<ChildLogReceiver> {
    let receiver = Arc::new(ChildLogReceiver {
        writers: RwLock::new(HashMap::new()),
    });
    let _ = CHILD_LOG_RECEIVER.set(receiver.clone());
    receiver
}

/// 获取全局 ChildLogReceiver
pub fn get_child_log_receiver() -> Option<Arc<ChildLogReceiver>> {
    CHILD_LOG_RECEIVER.get().cloned()
}

impl ChildLogReceiver {
    /// 注册一个设备（子进程连接时调用）
    pub async fn register_device(&self, device_id: DeviceId, device_name: String) {
        let log_dir = LOG_DIR.read().await.clone();
        let writer = DeviceLogWriter::new(device_name.clone(), log_dir);
        self.writers.write().await.insert(device_id, writer);
        tracing::info!("[ log ] 已注册设备[{}]的日志写入器", device_name);
    }

    /// 注销一个设备（子进程断开时调用）
    pub async fn unregister_device(&self, device_id: &DeviceId) {
        if let Some(writer) = self.writers.write().await.remove(device_id) {
            tracing::info!("[ log ] 已注销设备[{}]的日志写入器", writer.device_name);
        }
    }

    /// 处理来自子进程的日志消息
    /// - 写入对应设备的日志文件
    /// - 通过 tauri event emit 到前端
    pub async fn handle_log(&self, device_id: &DeviceId, log: &LogMessage) {
        // 写入文件
        let mut writers = self.writers.write().await;
        if let Some(writer) = writers.get_mut(device_id) {
            writer.write_log(log);
        }
        // emit 到前端由调用方负责（在 chanel_server.rs 中处理）
    }
}
