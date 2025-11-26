use crate::infrastructure::context::child_process_sec::init_ipc_client;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::core::cores_affinity::set_process_affinity;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::devices::device_conf::DeviceConfig;
use crate::infrastructure::logging::child_log::LogChild;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::logging::LogLevel;
use crate::infrastructure::vision::base_traits::TextDetector;
use crate::infrastructure::vision::ocr_service::OcrService;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// 子进程上下文（设备进程）
pub struct ChildProcessCtx {
    pub device_id: DeviceId,
    /// 设备信息
    pub device_config: DeviceConfig,

    /// 运行状态
    pub is_running: Arc<AtomicBool>,

    /// 检测模型
    pub det_model: Option<Box<dyn TextDetector>>,

    /// OCR模型
    pub ocr_service: Option<OcrService>,

    /// cpu核心
    pub cpu_cores: Vec<usize>,
}

/// 子进程初始化数据（可序列化，仅承载描述信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProcessInitData {
    pub device_id: DeviceId,
    pub device_config: DeviceConfig,
    pub shm_name: String,
    pub buffer_size: usize,
    pub log_level: LogLevel,
    /// cpu核心
    pub cpu_cores: Vec<usize>,
}

impl ChildProcessInitData {
    /// 从初始化数据构建（反序列化仅用于 InitData，不包含运行时资源）
    pub fn init_data_from_main(&self) -> InitResult<()> {
        // 设置线程亲和性
        set_process_affinity(&self.cpu_cores)
            .map_err(|e| InitError::InitChildCoreAffinity { e: e.to_string() })?;
        //初始化ipc客户端
        init_ipc_client(Arc::new(self.device_id), self.log_level.clone()).map_err(|_| {
            InitError::InitChildIpcClientFailed {
                e: "初始化ipc客户端失败".to_string(),
            }
        })?;

        //初始化日志
        Log::init_logger(Box::new(LogChild))?;
        // 初始化 Rayon 线程池
        //let rayon_pool = ThreadPoolBuilder::new().num_threads(4).build().map_err(|e| ChildProcessError::FailedToInitializeRayonPool)?;
        //init_rayon_pool(Arc::new(RwLock::new(rayon_pool))).map_err(|_| ChildProcessError::FailedToInitialize {e:"初始化全局线程池失败".into_string()})?;

        Ok(())
    }

    pub fn init_ort_env() -> bool {
        ort::init().with_telemetry(false).commit().unwrap_or(false)
    }
}

impl ChildProcessCtx {
    /// 从初始化数据构建（反序列化仅用于 InitData，不包含运行时资源）
    pub fn from_init(init: ChildProcessInitData) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device_id: init.device_id,
            device_config: init.device_config,
            is_running: Arc::new(AtomicBool::new(false)),
            det_model: None,
            ocr_service: None,
            cpu_cores: init.cpu_cores,
        })
    }

    /// 运行主循环
    pub async fn run_main_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
        while self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            // 处理来自主进程的消息
            self.process_messages().await?;

            // 执行设备相关任务
            self.process_device_tasks().await?;

            // 心跳和健康检查
            self.send_heartbeat().await?;

            // 短暂休眠，避免CPU占用过高
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    /// 处理消息
    async fn process_messages(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 从IPC通道接收和处理消息
        Ok(())
    }

    /// 处理设备任务
    async fn process_device_tasks(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 执行OCR识别、YOLO检测等任务
        Ok(())
    }

    /// 关闭子进程
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        //self.logger.info("Shutting down child process");

        // 设置停止标志
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // 清理资源
        // TODO: 释放模型资源、关闭IPC连接等

        //self.logger.info("Child process shutdown completed");
        Ok(())
    }
}
