use crate::domain::devices::device_conf::{CapMethod, DeviceConfig};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::context::child_process_sec::init_ipc_client;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::context::runtime_context::{init_runtime_ctx, RuntimeContext, SharedRuntimeContext};
use crate::infrastructure::core::cores_affinity::set_process_affinity;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::db::init_db_with_path;
use crate::infrastructure::devices::device_ctx::init_device_ctx;
use crate::infrastructure::ipc::message::ExecuteTarget;
use crate::infrastructure::logging::child_log::LogChild;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::logging::LogLevel;
use crate::infrastructure::vision::ocr_service::OcrService;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 子进程上下文（设备进程）
/// 子进程上下文（设备进程）
pub struct ChildProcessCtx {
    pub device_id: DeviceId,
    pub is_running: Arc<AtomicBool>,
    pub runtime_ctx: SharedRuntimeContext,
}

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
    /// 初始化子进程环境并构建全局上下文
    pub async fn init_environment(&self) -> InitResult<()> {
        // 1. 设置线程亲和性
        set_process_affinity(&self.cpu_cores)
            .map_err(|e| InitError::InitChildCoreAffinity { e: e.to_string() })?;

        // 2. 初始化日志
        Log::init_logger(Box::new(LogChild))?;

        // 3. 初始化数据库连接
        init_db_with_path(&self.db_path)
            .await
            .map_err(|e| InitError::InitChildDatabaseEnvFailed { e })?;

        // 4. 初始化 IPC 客户端
        init_ipc_client(Arc::new(self.device_id), self.log_level.clone()).map_err(|_| {
            InitError::InitChildIpcClientFailed {
                e: "初始化ipc客户端失败".to_string(),
            }
        })?;

        // 5. 初始化 ADB 上下文 (在子进程中使用 OnceLock 模型)
        let adb_config = if let Some(adb_info) = &self.device_config.adb_info {
            ADBConnectConfig::DirectTcp(Some(SocketAddrV4::new(adb_info.ip_addr, adb_info.port)))
        } else {
            ADBConnectConfig::DirectTcp(Some(SocketAddrV4::new([127, 0, 0, 1].into(), 16416)))
        };
        ADBCtx::new(adb_config).await;

        // 6. 初始化运行时上下文 (RUNTIME_CTX)
        let ocr_service = Arc::new(OcrService::new());
        //let adb_executor = Arc::new(RwLock::new(crate::infrastructure::adb_cli_local::adb_context::get_adb_ctx().adb_executor.clone_for_child()));
        
        let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
            self.device_id, // 占位
            ExecuteTarget::FullScript, // 占位
            ocr_service.clone(),
        )));
        init_runtime_ctx(runtime_ctx)?;
        // 7. 初始化设备上下文 (DEVICE_CTX)
        let (cap_method, title) = match &self.device_config.cap_method {
            CapMethod::Window(title) => (CaptureMethod::Window, Some(title.clone())),
            CapMethod::ADB => (CaptureMethod::ADB, None),
        };
        
        let device_ctx = Arc::new(crate::infrastructure::devices::device_ctx::DeviceCtx::new(
            Arc::new(RwLock::new(self.device_config.clone())),
            cap_method,
            title,
        ));
        init_device_ctx(device_ctx)?;

        Ok(())
    }

    pub fn init_ort_env() -> bool {
        ort::init().with_telemetry(false).commit().unwrap_or(false)
    }
}

impl ChildProcessCtx {
    /// 从初始化数据构建（反序列化仅用于 InitData，不包含运行时资源）
/*    pub fn from_init(init: ChildProcessInitData) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device_id: init.device_id,
            device_config: init.device_config,
            is_running: Arc::new(AtomicBool::new(false)),
            det_model: None,
            ocr_service: None,
            cpu_cores: init.cpu_cores,
            runtime_ctx: Arc::new(()),
        })
    }
*/
    /// 运行主循环
    pub async fn run_main_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
        while self.is_running.load(std::sync::atomic::Ordering::SeqCst) {
            // 处理来自主进程的消息
            self.process_messages().await?;

            // 执行设备相关任务
            self.process_device_tasks().await?;

            // 心跳和健康检查
            //self.send_heartbeat().await?;

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
