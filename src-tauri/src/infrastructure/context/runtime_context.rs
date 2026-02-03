use crate::domain::vision::ocr_search::{SearchHit, VisionSnapshot};
use crate::infrastructure::core::{HashMap, PolicyId, ScriptId, TaskId};
use crate::infrastructure::ipc::message::ExecuteTarget;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub script_id: ScriptId,
    pub target: ExecuteTarget,
    
    /// Rhai 变量映射
    pub var_map: HashMap<String, rhai::Dynamic>,
    
    /// 策略状态
    pub policy_states: HashMap<PolicyId, PolicyState>,
    
    /// 任务状态
    pub task_states: HashMap<TaskId, TaskState>,

    /// 每一帧的视觉快照缓存
    pub last_snapshot: Option<VisionSnapshot>,
    
    /// 每一帧的搜索命中结果缓存
    pub last_hits: Vec<SearchHit<'static>>, 

    /// 设备相关属性
    pub screen_size: (u32, u32),
}

impl RuntimeContext {
    pub fn new(script_id: ScriptId, target: ExecuteTarget) -> Self {
        Self {
            script_id,
            target,
            var_map: HashMap::new(),
            policy_states: HashMap::new(),
            task_states: HashMap::new(),
            last_snapshot: None,
            last_hits: Vec::new(),
            screen_size: (0, 0),
        }
    }
}

pub type SharedRuntimeContext = Arc<RwLock<RuntimeContext>>;
