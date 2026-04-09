use crate::domain::scripts::script_info::ScriptInfo;
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
use crate::domain::vision::ocr_search::{SearchHit, VisionSnapshot};
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::core::{HashMap, PolicyId, ScheduleId, ScriptId, StepId, TaskId};
use crate::infrastructure::ipc::message::RunTarget;
use crate::infrastructure::vision::ocr_service::OcrService;
use crate::infrastructure::vision::text_rec_cache::ScriptTextRecCacheRuntime;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

static RUNTIME_CTX: OnceLock<SharedRuntimeContext> = OnceLock::new();

pub fn get_runtime_ctx() -> SharedRuntimeContext {
    RUNTIME_CTX.get().expect("RuntimeContext not initialized").clone()
}

pub fn init_runtime_ctx(ctx: SharedRuntimeContext)->InitResult<()>  {
    RUNTIME_CTX.set(ctx).map_err(|_| InitError::InitChildAppCtxFailed { e: "RuntimeContext already initialized".to_string() })?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct PolicyState {
    pub skip_flag: bool,
    pub exec_cur: u32,
}

#[derive(Debug, Default, Clone)]
pub struct TaskState {
    pub skip_flag: bool,
    pub done_flag: bool,
}

#[derive(Debug)]
pub struct RuntimeContext {
    pub current_assignment_id: Option<ScheduleId>,
    pub script_id: ScriptId,
    pub target: RunTarget,
    pub script_info: Option<ScriptInfo>,
    pub current_task: Option<ScriptTaskTable>,
    pub current_step_id: Option<StepId>,
    
    /// 基础服务
    pub ocr_service: Arc<OcrService>,
    //pub adb_executor: Arc<RwLock<ADBExecutor>>,
    
    /// Rhai 变量映射
    pub var_map: HashMap<String, rhai::Dynamic>,
    
    /// 策略状态
    pub policy_states: HashMap<PolicyId, PolicyState>,
    
    /// 任务状态
    pub task_states: HashMap<TaskId, TaskState>,

    /// 每一帧的视觉快照缓存
    pub last_snapshot: Option<VisionSnapshot>,
    
    /// 每一帧的搜索命中结果缓存
    pub last_hits: Vec<SearchHit>, 

    /// 设备相关属性
    pub screen_size: (u32, u32),

    /// OCR 文字缓存运行时
    pub vision_text_cache: ScriptTextRecCacheRuntime,

    /// 视觉签名网格大小，用于稳定坐标和排序分桶。
    pub vision_signature_grid_size: u16,
}

impl RuntimeContext {
    pub fn new(
        script_id: ScriptId,
        target: RunTarget,
        ocr_service: Arc<OcrService>,
        vision_text_cache_config: VisionTextCacheRuntimeConfig,
        //adb_executor: Arc<RwLock<ADBExecutor>>,
    ) -> Self {
        let vision_signature_grid_size = vision_text_cache_config.signature_grid_size.max(1);
        Self {
            current_assignment_id: None,
            script_id,
            target,
            script_info: None,
            current_task: None,
            current_step_id: None,
            ocr_service,
            //adb_executor,
            var_map: HashMap::new(),
            policy_states: HashMap::new(),
            task_states: HashMap::new(),
            last_snapshot: None,
            last_hits: Vec::new(),
            screen_size: (0, 0),
            vision_text_cache: ScriptTextRecCacheRuntime::new(vision_text_cache_config),
            vision_signature_grid_size,
        }
    }
}

pub type SharedRuntimeContext = Arc<RwLock<RuntimeContext>>;
