pub use runtime_engine::infrastructure::context::child_process::ChildProcessInitData;

use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::context::child_process_sec::init_ipc_client;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::context::runtime_context::{init_runtime_ctx, RuntimeContext};
use crate::infrastructure::core::cores_affinity::set_process_affinity;
use crate::infrastructure::db::init_db_with_path;
use crate::infrastructure::devices::device_ctx::init_device_ctx;
use crate::infrastructure::ipc::message::ExecuteTarget;
use crate::infrastructure::logging::child_log::LogChild;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::ocr_service::OcrService;
use runtime_engine::domain::devices::device_conf::CapMethod;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn init_environment(init_data: &ChildProcessInitData) -> InitResult<()> {
    set_process_affinity(&init_data.cpu_cores)
        .map_err(|e| InitError::InitChildCoreAffinity { e: e.to_string() })?;

    Log::init_logger(Box::new(LogChild))
        .map_err(|e| InitError::InitLoggerFailed { e: e.to_string() })?;

    init_db_with_path(&init_data.db_path)
        .await
        .map_err(|e| InitError::InitChildDatabaseEnvFailed { e })?;

    init_ipc_client(Arc::new(init_data.device_id), init_data.log_level.clone()).map_err(|_| {
        InitError::InitChildIpcClientFailed {
            e: "初始化ipc客户端失败".to_string(),
        }
    })?;

    let adb_config = init_data
        .device_config
        .adb_connect
        .clone()
        .unwrap_or_else(|| ADBConnectConfig::DirectTcp(None));
    ADBCtx::new(adb_config).await;

    let ocr_service = Arc::new(OcrService::new());
    let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
        init_data.device_id,
        ExecuteTarget::FullScript,
        ocr_service,
    )));
    init_runtime_ctx(runtime_ctx)?;

    let (cap_method, title) = match &init_data.device_config.cap_method {
        CapMethod::Window(title) => (CaptureMethod::Window, Some(title.clone())),
        CapMethod::Adb => (CaptureMethod::Adb, None),
    };

    let device_ctx = Arc::new(crate::infrastructure::devices::device_ctx::DeviceCtx::new(
        Arc::new(RwLock::new(init_data.device_config.clone())),
        cap_method,
        title,
    ));
    init_device_ctx(device_ctx)?;

    Ok(())
}
