use runner_protocol::ChildProcessInitData;

use super::{ChildRuntimeInitError, ChildRuntimeInitResult};
use crate::infra::context::runtime_context::{RuntimeContext, init_runtime_ctx};
use crate::infra::context::runtime_control::{init_ipc_client, start_ipc_client};
use crate::infra::logging::child_log::LogChild;
use crate::infra::logging::log_trait::Log;
use crate::infra::process_affinity::set_process_affinity;
use infra_device_runtime::{DeviceCtx, init_device_ctx};
use infra_sqlite::init_db_with_path;
use infra_vision::OcrService;
use runner_protocol::message::RunTarget;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub(crate) async fn init_environment(
    init_data: &ChildProcessInitData,
) -> ChildRuntimeInitResult<()> {
    set_process_affinity(&init_data.cpu_cores)
        .map_err(|e| ChildRuntimeInitError::InitChildCoreAffinity { e: e.to_string() })?;

    Log::init_logger(Box::new(LogChild))
        .map_err(|e| ChildRuntimeInitError::InitLoggerFailed { e: e.to_string() })?;

    init_db_with_path(&init_data.db_path)
        .await
        .map_err(|e| ChildRuntimeInitError::InitChildDatabaseEnvFailed { e })?;

    init_ipc_client(Arc::new(init_data.device_id), init_data.log_level.clone()).map_err(|_| {
        ChildRuntimeInitError::InitChildIpcClientFailed {
            e: "初始化ipc客户端失败".to_string(),
        }
    })?;

    let img_det_service = Arc::new(Mutex::new(OcrService::new()));
    let ocr_service = Arc::new(Mutex::new(OcrService::new()));
    let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
        init_data.device_id,
        RunTarget::DeviceQueue,
        img_det_service,
        ocr_service,
        init_data.vision_text_cache_config.clone(),
    )));
    init_runtime_ctx(runtime_ctx)?;

    let device_ctx =
        Arc::new(DeviceCtx::new(Arc::new(RwLock::new(init_data.device_config.clone()))).await);
    init_device_ctx(device_ctx).map_err(|e| ChildRuntimeInitError::InitChildAppCtxFailed { e })?;
    start_ipc_client()?;

    Ok(())
}
