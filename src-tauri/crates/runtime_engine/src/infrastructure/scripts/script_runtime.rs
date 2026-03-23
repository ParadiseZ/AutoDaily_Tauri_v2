use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::core::HashMap;
use crate::infrastructure::scripts::script_info_model::ScriptMeta;

pub struct ScriptRuntime {
    pub script_meta: ScriptMeta,
    //策略
    pub decision: Vec<Step>,
    pub back_decision: Vec<Step>,
    pub global_decision: Vec<Step>,

    // 参数
    pub script_args: HashMap<String, Vec<i32>>,
    pub script_start_time: String,
    pub script_end_time: String,
    pub script_duration: String,

    //模型
    //运行策略
}
