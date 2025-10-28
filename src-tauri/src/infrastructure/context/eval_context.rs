use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::core::{HashMap, ScriptId, TaskId};


#[derive(Debug)]
pub struct EvalContext {
    script_id : ScriptId,
    ocr_result: Option<Vec<OcrResult>>, // key: "银币"
    ocr_cache : Option<HashMap<String,OcrResult>>,
    det_result: Option<Vec<DetResult>>,
    current_task: TaskId,
    var_map: HashMap<String,String>,
    screen_size: (u32, u32),
}