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
use std::sync::Arc;
use tokio::sync::RwLock;

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

        let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
            self.device_id,
            ExecuteTarget::FullScript,
            ocr_service.clone(),
        )));
        init_runtime_ctx(runtime_ctx)?;

        // 7. 初始化设备上下文 (DEVICE_CTX)
        let (cap_method, title) = match &self.device_config.cap_method {
            CapMethod::Window(title) => (CaptureMethod::Window, Some(title.clone())),
            CapMethod::Adb => (CaptureMethod::Adb, None),
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
