use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
use crate::domain::scripts::script_info::ScriptInfo;
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::domain::vision::ocr_search::{SearchHit, VisionSnapshot};
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::core::{
    AccountId, DeviceId, ExecutionId, HashMap, PolicyId, PolicySetId, ScheduleId, ScriptId,
    StepId, TaskId, TemplateId,
};
use crate::infrastructure::ipc::message::RunTarget;
use crate::infrastructure::vision::ocr_service::OcrService;
use crate::infrastructure::vision::text_rec_cache::ScriptTextRecCacheRuntime;
use image::RgbaImage;
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, RwLock};

static RUNTIME_CTX: OnceLock<SharedRuntimeContext> = OnceLock::new();

pub fn get_runtime_ctx() -> SharedRuntimeContext {
    RUNTIME_CTX
        .get()
        .expect("RuntimeContext not initialized")
        .clone()
}

pub fn init_runtime_ctx(ctx: SharedRuntimeContext) -> InitResult<()> {
    RUNTIME_CTX
        .set(ctx)
        .map_err(|_| InitError::InitChildAppCtxFailed {
            e: "RuntimeContext already initialized".to_string(),
        })?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct PolicyState {
    pub skip_flag: bool,
    pub done_flag: bool,
    pub exec_cur: u32,
    pub click_pos: Option<u16>,
}

#[derive(Debug, Default, Clone)]
pub struct ActionState {
    pub exec_cur: u32,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub enabled_flag: bool,
    pub skip_flag: bool,
    pub done_flag: bool,
    pub exec_cur: u32,
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            enabled_flag: true,
            skip_flag: false,
            done_flag: false,
            exec_cur: 0,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionState {
    pub current_execution_id: Option<ExecutionId>,
    pub current_assignment_id: Option<ScheduleId>,
    pub current_device_id: Option<DeviceId>,
    pub current_time_template_id: Option<TemplateId>,
    pub current_account_id: Option<AccountId>,
    pub script_id: ScriptId,
    pub target: RunTarget,
    pub script_info: Option<ScriptInfo>,
    pub current_task: Option<ScriptTaskTable>,
    pub current_step_id: Option<StepId>,

    /// Rhai 变量映射
    pub var_map: HashMap<String, rhai::Dynamic>,

    /// 当前执行使用的模板变量快照。运行时自动切换 UI 变量时会更新它，供后续任务 hydrate 使用。
    pub template_values_json: Option<String>,

    /// 策略状态
    pub policy_states: HashMap<PolicyId, PolicyState>,

    /// 任务状态
    pub task_states: HashMap<TaskId, TaskState>,

    /// 动作状态（按 action step id 计数）
    pub action_states: HashMap<StepId, ActionState>,

    /// 运行时策略集附加关系：target <- [source...]
    pub policy_set_overlays: HashMap<PolicySetId, Vec<PolicySetId>>,
}

impl ExecutionState {
    pub fn new(script_id: ScriptId, target: RunTarget) -> Self {
        Self {
            current_execution_id: None,
            current_assignment_id: None,
            current_device_id: None,
            current_time_template_id: None,
            current_account_id: None,
            script_id,
            target,
            script_info: None,
            current_task: None,
            current_step_id: None,
            var_map: HashMap::new(),
            template_values_json: None,
            policy_states: HashMap::new(),
            task_states: HashMap::new(),
            action_states: HashMap::new(),
            policy_set_overlays: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ObservationState {
    /// 最近一次截图动作得到的原始图像。
    pub last_capture_image: Option<Arc<RgbaImage>>,

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

impl ObservationState {
    pub fn new(vision_text_cache_config: VisionTextCacheRuntimeConfig) -> Self {
        let vision_signature_grid_size = vision_text_cache_config.signature_grid_size.max(1);
        Self {
            last_capture_image: None,
            last_snapshot: None,
            last_hits: Vec::new(),
            screen_size: (0, 0),
            vision_text_cache: ScriptTextRecCacheRuntime::new(vision_text_cache_config),
            vision_signature_grid_size,
        }
    }
}

#[derive(Debug)]
pub struct RuntimeContext {
    pub execution: ExecutionState,
    pub observation: ObservationState,

    /// 基础服务
    pub img_det_service: Arc<Mutex<OcrService>>,
    pub ocr_service: Arc<Mutex<OcrService>>,
    //pub adb_executor: Arc<RwLock<ADBExecutor>>,
}

impl RuntimeContext {
    pub fn new(
        script_id: ScriptId,
        target: RunTarget,
        img_det_service: Arc<Mutex<OcrService>>,
        ocr_service: Arc<Mutex<OcrService>>,
        vision_text_cache_config: VisionTextCacheRuntimeConfig,
        //adb_executor: Arc<RwLock<ADBExecutor>>,
    ) -> Self {
        Self {
            execution: ExecutionState::new(script_id, target),
            observation: ObservationState::new(vision_text_cache_config),
            img_det_service,
            ocr_service,
        }
    }
}

pub type SharedRuntimeContext = Arc<RwLock<RuntimeContext>>;
