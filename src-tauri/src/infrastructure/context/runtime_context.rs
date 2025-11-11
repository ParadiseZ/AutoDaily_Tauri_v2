use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::core::{HashMap, ScriptId, TaskId};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use tokio::sync::Mutex;


#[derive(Debug)]
pub struct RuntimeContext {
    script_id : ScriptId,
    ocr_result: Option<Vec<OcrResult>>, // key: "银币"
    ocr_cache : Option<HashMap<String,OcrResult>>,
    det_result: Option<Vec<DetResult>>,
    current_task: TaskId,
    var_map: HashMap<String,String>,
    screen_size: (u32, u32),

    pub exec_cmd_thread: Arc<AtomicBool>,
    pub adb_config: Arc<Mutex<Option<ADBConnectConfig>>>,
    pub interval : Arc<AtomicU64>
}

impl RuntimeContext {
    pub fn new(script_id: ScriptId, task_id: TaskId) -> Self {
        RuntimeContext {
            script_id,
            ocr_result: None,
            ocr_cache: None,
            det_result: None,
            current_task: task_id,
            var_map: HashMap::new(),
            screen_size: (0,0),
            exec_cmd_thread: Arc::new(AtomicBool::new(false)),
            adb_config: Arc::new(Mutex::new(None)),
            interval: Arc::new(AtomicU64::new(1000)),
        }
    }
}