use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::context::init_error::InitResult;
use crate::infrastructure::core::{
    AssignmentId, DeviceId, DispatchId, HashMap, JobId, MessageId, ScriptId,
};
use crate::infrastructure::ipc::chanel_server::IpcClientState;
use crate::infrastructure::ipc::message::{
    ConnectionStatusKind, RuntimeDispatchPhase, RuntimeQueueItem, RuntimeSessionSnapshot,
};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::script_info_model::ScriptManager;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, watch};

#[derive(Clone, Debug)]
pub struct DeviceConnectionState {
    pub status: ConnectionStatusKind,
    pub message: Option<String>,
}

impl Default for DeviceConnectionState {
    fn default() -> Self {
        Self {
            status: ConnectionStatusKind::DeviceUnknown,
            message: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DeviceDispatchState {
    pub active_dispatch: Option<DispatchId>,
    pub pending_dispatches: VecDeque<RuntimeQueueItem>,
    pub pending_debug_sessions: VecDeque<RuntimeSessionSnapshot>,
    pub auto_dispatch_blocked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DeviceProgressState {
    pub phase: Option<String>,
    pub message: Option<String>,
    pub at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ChildRuntimeStatus {
    #[default]
    Unknown,
    Starting,
    IpcWaiting,
    IpcReady,
    Exited,
    Crashed,
}

#[derive(Clone, Debug, Default)]
pub struct DeviceRuntimeState {
    pub child_runtime_status: ChildRuntimeStatus,
    pub connection: DeviceConnectionState,
    pub dispatch: DeviceDispatchState,
    pub progress: DeviceProgressState,
    pub last_error: Option<String>,
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
        job_id: JobId,
        device_id: DeviceId,
        previous: Option<DeviceTable>,
        current: DeviceTable,
    },
    DeviceSessionRefresh {
        job_id: JobId,
        device_id: DeviceId,
        sync_session: bool,
        reevaluate_dispatch: bool,
        reason: String,
    },
}

impl RuntimeReconcileJob {
    pub fn job_id(&self) -> JobId {
        match self {
            Self::DeviceConfig { job_id, .. } | Self::DeviceSessionRefresh { job_id, .. } => {
                *job_id
            }
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

    /// 每台设备唯一运行态（子进程、连接、派发、当前进度都收口到这里）
    pub device_runtime_states: Arc<RwLock<HashMap<DeviceId, DeviceRuntimeState>>>,

    /// 每台设备独立的运行态通知（由主进程状态变化驱动）
    pub device_runtime_signals: Arc<RwLock<HashMap<DeviceId, watch::Sender<DeviceRuntimeState>>>>,

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
                device_runtime_states: Arc::new(RwLock::new(HashMap::new())),
                device_runtime_signals: Arc::new(RwLock::new(HashMap::new())),
                device_capture_results: Arc::new(RwLock::new(HashMap::new())),
                dispatch_signal_tx,
                runtime_reconcile_tx,
            },
            dispatch_signal_rx,
            runtime_reconcile_rx,
        )
    }

    fn notify_runtime_state(
        &self,
        device_id: DeviceId,
        runtime_state: DeviceRuntimeState,
    ) -> Result<(), String> {
        let mut guard = self
            .device_runtime_signals
            .write()
            .map_err(|_| "写入设备运行态通知失败".to_string())?;
        match guard.get(&device_id) {
            Some(sender) => {
                sender.send_replace(runtime_state);
            }
            None => {
                let (sender, _) = watch::channel(runtime_state);
                guard.insert(device_id, sender);
            }
        }
        Ok(())
    }

    pub fn ensure_device_runtime_state(&self, device_id: DeviceId) -> Result<(), String> {
        let runtime_state = {
            let mut guard = self
                .device_runtime_states
                .write()
                .map_err(|_| "写入设备运行态失败".to_string())?;
            guard.entry(device_id).or_default().clone()
        };
        self.notify_runtime_state(device_id, runtime_state)
    }

    pub fn clear_device_runtime_state(&self, device_id: DeviceId) -> Result<(), String> {
        {
            let mut guard = self
                .device_runtime_states
                .write()
                .map_err(|_| "写入设备运行态失败".to_string())?;
            guard.remove(&device_id);
        }
        let mut signal_guard = self
            .device_runtime_signals
            .write()
            .map_err(|_| "写入设备运行态通知失败".to_string())?;
        signal_guard.remove(&device_id);
        Ok(())
    }

    pub fn snapshot_device_runtime_state(
        &self,
        device_id: DeviceId,
    ) -> Result<DeviceRuntimeState, String> {
        let guard = self
            .device_runtime_states
            .read()
            .map_err(|_| "读取设备运行态失败".to_string())?;
        Ok(guard.get(&device_id).cloned().unwrap_or_default())
    }

    pub fn subscribe_device_runtime_state(
        &self,
        device_id: DeviceId,
    ) -> Result<watch::Receiver<DeviceRuntimeState>, String> {
        let current = self.snapshot_device_runtime_state(device_id)?;
        let mut guard = self
            .device_runtime_signals
            .write()
            .map_err(|_| "写入设备运行态通知失败".to_string())?;
        Ok(match guard.get(&device_id) {
            Some(sender) => sender.subscribe(),
            None => {
                let (sender, receiver) = watch::channel(current);
                guard.insert(device_id, sender);
                receiver
            }
        })
    }

    pub fn mutate_device_runtime_state<F>(
        &self,
        device_id: DeviceId,
        mutate: F,
    ) -> Result<DeviceRuntimeState, String>
    where
        F: FnOnce(&mut DeviceRuntimeState),
    {
        let runtime_state = {
            let mut guard = self
                .device_runtime_states
                .write()
                .map_err(|_| "写入设备运行态失败".to_string())?;
            let state = guard.entry(device_id).or_default();
            mutate(state);
            state.clone()
        };
        self.notify_runtime_state(device_id, runtime_state.clone())?;
        Ok(runtime_state)
    }

    pub fn set_child_runtime_status(
        &self,
        device_id: DeviceId,
        status: ChildRuntimeStatus,
    ) -> Result<DeviceRuntimeState, String> {
        self.mutate_device_runtime_state(device_id, move |state| {
            state.child_runtime_status = status;
        })
    }

    pub fn set_device_connection_state(
        &self,
        device_id: DeviceId,
        status: ConnectionStatusKind,
        message: Option<String>,
    ) -> Result<DeviceRuntimeState, String> {
        self.mutate_device_runtime_state(device_id, move |state| {
            state.connection.status = status.clone();
            state.connection.message = message.clone();
            match status {
                ConnectionStatusKind::DeviceConnected => {
                    state.last_error = None;
                }
                ConnectionStatusKind::DeviceDisconnected => {
                    state.last_error = message.clone();
                }
                _ => {}
            }
        })
    }

    pub fn set_device_progress(
        &self,
        device_id: DeviceId,
        phase: impl Into<String>,
        message: impl Into<String>,
        at: Option<String>,
    ) -> Result<DeviceRuntimeState, String> {
        let phase = phase.into();
        let message = message.into();
        self.mutate_device_runtime_state(device_id, move |state| {
            state.progress.phase = Some(phase.clone());
            state.progress.message = Some(message.clone());
            state.progress.at = at.clone();
        })
    }

    pub fn replace_pending_dispatches(
        &self,
        device_id: DeviceId,
        queue: Vec<RuntimeQueueItem>,
    ) -> Result<(), String> {
        self.mutate_device_runtime_state(device_id, move |state| {
            state.dispatch.pending_dispatches = queue.into_iter().collect();
        })?;
        Ok(())
    }

    pub fn push_debug_session(
        &self,
        device_id: DeviceId,
        session: RuntimeSessionSnapshot,
    ) -> Result<(), String> {
        self.mutate_device_runtime_state(device_id, move |state| {
            state.dispatch.pending_debug_sessions.push_back(session);
        })?;
        Ok(())
    }

    pub fn pop_debug_session(
        &self,
        device_id: DeviceId,
    ) -> Result<Option<RuntimeSessionSnapshot>, String> {
        let mut result = None;
        self.mutate_device_runtime_state(device_id, |state| {
            result = state.dispatch.pending_debug_sessions.pop_front();
        })?;
        Ok(result)
    }

    pub fn mark_active_dispatch(
        &self,
        device_id: DeviceId,
        dispatch_id: Option<DispatchId>,
    ) -> Result<(), String> {
        self.mutate_device_runtime_state(device_id, move |state| {
            state.dispatch.active_dispatch = dispatch_id;
        })?;
        Ok(())
    }

    pub fn set_auto_dispatch_blocked(
        &self,
        device_id: DeviceId,
        blocked: bool,
    ) -> Result<(), String> {
        self.mutate_device_runtime_state(device_id, move |state| {
            state.dispatch.auto_dispatch_blocked = blocked;
        })?;
        Ok(())
    }

    pub fn pop_next_dispatch(
        &self,
        device_id: DeviceId,
    ) -> Result<Option<RuntimeQueueItem>, String> {
        let mut result = None;
        self.mutate_device_runtime_state(device_id, |state| {
            result = state.dispatch.pending_dispatches.pop_front();
        })?;
        Ok(result)
    }

    pub fn snapshot_device_dispatch_state(
        &self,
        device_id: DeviceId,
    ) -> Result<DeviceDispatchState, String> {
        Ok(self.snapshot_device_runtime_state(device_id)?.dispatch)
    }

    pub fn reset_device_dispatch_state(&self, device_id: DeviceId) -> Result<(), String> {
        self.mutate_device_runtime_state(device_id, |state| {
            state.dispatch = DeviceDispatchState::default();
        })?;
        Ok(())
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
