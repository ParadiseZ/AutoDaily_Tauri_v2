use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::context::init_error::InitResult;
use crate::infrastructure::core::{
    AssignmentId, DeviceId, DispatchId, HashMap, MessageId, ScriptId,
};
use crate::infrastructure::ipc::chanel_server::IpcClientState;
use crate::infrastructure::ipc::message::{ConnectionStatusKind, RuntimeDispatchPhase};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::script_info_model::ScriptManager;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct DeviceConnectionState {
    pub status: ConnectionStatusKind,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DeviceCaptureResult {
    pub device_id: DeviceId,
    pub image_data: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DeviceDispatchSignal {
    pub device_id: DeviceId,
    pub dispatch_id: Option<DispatchId>,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: Option<ScriptId>,
    pub phase: RuntimeDispatchPhase,
    pub message: Option<String>,
    pub at: String,
}

#[derive(Clone, Debug)]
pub enum RuntimeReconcileJob {
    DeviceConfig {
        job_id: String,
        device_id: DeviceId,
        previous: Option<DeviceTable>,
        current: DeviceTable,
    },
    DeviceSessionRefresh {
        job_id: String,
        device_id: DeviceId,
        sync_session: bool,
        reevaluate_dispatch: bool,
        reason: String,
    },
}

impl RuntimeReconcileJob {
    pub fn job_id(&self) -> &str {
        match self {
            Self::DeviceConfig { job_id, .. } | Self::DeviceSessionRefresh { job_id, .. } => job_id,
        }
    }

    pub fn device_id(&self) -> DeviceId {
        match self {
            Self::DeviceConfig { device_id, .. } | Self::DeviceSessionRefresh { device_id, .. } => {
                *device_id
            }
        }
    }

    pub fn job_type(&self) -> &'static str {
        match self {
            Self::DeviceConfig { .. } => "deviceConfig",
            Self::DeviceSessionRefresh { .. } => "deviceSessionRefresh",
        }
    }
}

/// 主进程上下文 - 优化的数据存储策略
pub struct MainProcessCtx {
    /// 脚本管理器（使用分页+缓存，不全量加载）
    pub script_manager: Arc<RwLock<ScriptManager>>,

    /// IPC通道映射（运行时数据，必须在内存中）
    pub ipc_servers: Arc<RwLock<HashMap<Arc<DeviceId>, Arc<IpcClientState>>>>,

    /// 设备连接状态（由子进程回传）
    pub device_connections: Arc<RwLock<HashMap<DeviceId, DeviceConnectionState>>>,

    /// 设备截图结果（由子进程按请求回传）
    pub device_capture_results: Arc<RwLock<HashMap<MessageId, DeviceCaptureResult>>>,

    /// dispatch 运行信号，供主进程调度器消费
    pub dispatch_signal_tx: mpsc::UnboundedSender<DeviceDispatchSignal>,

    /// runtime 副作用异步任务，按 device_id 串行执行
    pub runtime_reconcile_tx: mpsc::UnboundedSender<RuntimeReconcileJob>,
}

impl MainProcessCtx {
    pub fn new() -> (
        Self,
        mpsc::UnboundedReceiver<DeviceDispatchSignal>,
        mpsc::UnboundedReceiver<RuntimeReconcileJob>,
    ) {
        let (dispatch_signal_tx, dispatch_signal_rx) = mpsc::unbounded_channel();
        let (runtime_reconcile_tx, runtime_reconcile_rx) = mpsc::unbounded_channel();
        (
            Self {
                script_manager: Arc::new(RwLock::new(ScriptManager::empty())),
                ipc_servers: Arc::new(RwLock::new(HashMap::new())),
                device_connections: Arc::new(RwLock::new(HashMap::new())),
                device_capture_results: Arc::new(RwLock::new(HashMap::new())),
                dispatch_signal_tx,
                runtime_reconcile_tx,
            },
            dispatch_signal_rx,
            runtime_reconcile_rx,
        )
    }

    pub async fn init_scripts_mgr() -> InitResult<()> {
        Log::info("初始化脚本管理器...");
        // 创建脚本管理器并加载索引
        //let mut script_manager = ScriptManager::new(script_cache_size);

        // 加载所有脚本数据
        //script_manager .load_from_directory(&ScriptsConfig::get_dir().await).await.map_err(|e| InitError::InitMainScriptMgrErr { e: e.to_string() })?;
        Ok(())
    }

    /// 搜索脚本数据
    pub async fn init_(
        &mut self, //request: crate::infrastructure::scripts::script_info_model::ScriptPageReq,
    ) -> Result<
        //crate::infrastructure::scripts::script_info_model::ScriptPageResp,
        (),
        Box<dyn std::error::Error>,
    > {
        Log::info("初始化脚本管理器...");
        Ok(())
        //self.script_manager.read().unwrap().get_scripts_page(request)
    }

    /*    /// 添加设备配置（小量数据，直接存储在内存）
    pub fn add_device_config(&mut self, config: DeviceConfig) {
        self.device_config.push(config);
    }

    /// 获取设备配置（从内存中快速获取）
    pub fn get_device_config(&self, device_id: &DeviceId) -> Option<&DeviceConfig> {
        self.device_config.iter().find(|config| config.device_id == *device_id)
    }*/

    pub fn run(&self) {}
}
