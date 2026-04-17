use crate::infrastructure::context::init_error::InitResult;
use crate::infrastructure::core::{DeviceId, ExecutionId, HashMap};
use crate::infrastructure::ipc::chanel_server::IpcClientState;
use crate::infrastructure::ipc::message::RuntimeRecoveryPhase;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::script_info_model::ScriptManager;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use tokio::sync::Notify;

#[derive(Debug, Clone)]
pub struct RecoverySignal {
    pub sequence: u64,
    pub phase: RuntimeRecoveryPhase,
    pub execution_id: Option<ExecutionId>,
    pub checkpoint_updated_at: Option<String>,
}

#[derive(Debug)]
pub struct RecoveryRuntimeState {
    pub latest_by_device: RwLock<HashMap<DeviceId, RecoverySignal>>,
    pub notify: Notify,
    next_sequence: AtomicU64,
}

impl RecoveryRuntimeState {
    pub fn new() -> Self {
        Self {
            latest_by_device: RwLock::new(HashMap::new()),
            notify: Notify::new(),
            next_sequence: AtomicU64::new(1),
        }
    }

    pub fn current_sequence(&self, device_id: DeviceId) -> u64 {
        self.latest_by_device
            .read()
            .ok()
            .and_then(|guard| guard.get(&device_id).map(|signal| signal.sequence))
            .unwrap_or(0)
    }

    pub fn latest_signal(&self, device_id: DeviceId) -> Option<RecoverySignal> {
        self.latest_by_device
            .read()
            .ok()
            .and_then(|guard| guard.get(&device_id).cloned())
    }

    pub fn record(
        &self,
        device_id: DeviceId,
        phase: RuntimeRecoveryPhase,
        execution_id: Option<ExecutionId>,
        checkpoint_updated_at: Option<String>,
    ) {
        let sequence = self
            .next_sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut guard) = self.latest_by_device.write() {
            guard.insert(
                device_id,
                RecoverySignal {
                    sequence,
                    phase,
                    execution_id,
                    checkpoint_updated_at,
                },
            );
        }
        self.notify.notify_waiters();
    }
}

/// 主进程上下文 - 优化的数据存储策略
pub struct MainProcessCtx {
    /// 脚本管理器（使用分页+缓存，不全量加载）
    pub script_manager: Arc<RwLock<ScriptManager>>,

    /// IPC通道映射（运行时数据，必须在内存中）
    pub ipc_servers: Arc<RwLock<HashMap<Arc<DeviceId>, Arc<IpcClientState>>>>,
    pub recovery_runtime: Arc<RecoveryRuntimeState>,
}

impl MainProcessCtx {
    pub fn new() -> Self {
        Self {
            script_manager: Arc::new(RwLock::new(ScriptManager::empty())),
            ipc_servers: Arc::new(RwLock::new(HashMap::new())),
            recovery_runtime: Arc::new(RecoveryRuntimeState::new()),
        }
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
