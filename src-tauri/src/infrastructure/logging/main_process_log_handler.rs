use crate::infrastructure::core::{DeviceId, HashMap};
use crate::infrastructure::devices::device_conf::DeviceConfigAll;
use crate::infrastructure::logging::child_log::{ChildLogResult, LogEntry, SharedLogRingBuffer};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// 主进程的日志接收器（按设备维度维护共享内存缓冲区）
pub struct SharedLogReceiver {
    buffers: HashMap<DeviceId, Arc<SharedLogRingBuffer>>,
}

impl SharedLogReceiver {
    pub fn new() -> Self { Self { buffers: HashMap::new() } }

    /// 基于设备配置进行增量刷新：
    /// - 删除已存在但配置中不存在的设备（释放其缓冲区与命名共享内存句柄）
    /// - 为配置中新出现的设备创建并注册共享内存缓冲区
    pub fn refresh_from_config(&mut self, device_config_all: &DeviceConfigAll) -> ChildLogResult<()> {
        let desired: HashSet<DeviceId> = device_config_all.keys().cloned().collect();
        let current: HashSet<DeviceId> = self.buffers.keys().cloned().collect();

        // 需要移除的设备
        for device_id in current.difference(&desired) {
            self.buffers.remove(device_id);
            // 移除后，Arc 计数为 0 则自动 Drop，释放命名共享内存映射与句柄
        }

        // 需要新增的设备
        for device_id in desired.difference(&current) {
            if let Some(conf) = device_config_all.get(device_id) {
                let shm_name = conf.device_name.lock().unwrap().clone();
                let buffer = SharedLogRingBuffer::create_named(&shm_name, *device_id, 1024)?;
                self.buffers.insert(*device_id, Arc::new(buffer));
            }
        }
        Ok(())
    }

    /// 注册现成的缓冲区（按设备）
    pub fn register_device_buffer(&mut self, device_id: DeviceId, buffer: Arc<SharedLogRingBuffer>) {
        self.buffers.insert(device_id, buffer);
    }

    /// 通过命名共享内存注册（主进程打开或创建）
    pub fn register_named_buffer(
        &mut self,
        device_id: DeviceId,
        shm_name: &str,
        buffer_size: usize,
    ) -> ChildLogResult<()> {
        let buf = SharedLogRingBuffer::open_named(shm_name, device_id, buffer_size)?;
        self.buffers.insert(device_id, Arc::new(buf));
        Ok(())
    }

    /// 注销设备缓冲区
    pub fn unregister_device(&mut self, device_id: &DeviceId) {
        self.buffers.remove(device_id);
    }

    /// 主循环：实时接收日志通知并处理
    pub async fn run(&mut self) -> ChildLogResult<()> {
        loop {
            for (_dev, buffer) in self.buffers.iter() {
                for bid in 0..2usize {
                    let logs = buffer.read_buffer(bid)?;
                    for (log_entry, message) in logs {
                        self.process_single_log(log_entry, message).await?;
                    }
                }
            }
            sleep(Duration::from_millis(5)).await; // 轮询间隔
        }
    }

    /// 处理日志通知（每条日志都实时处理）
    /// 处理单条日志（实时）
    async fn process_single_log(&self, log: LogEntry, message: String) -> ChildLogResult<()> {
        // TODO: 写入文件、发送到UI等
        let _ = (log, message);
        Ok(())
    }
}