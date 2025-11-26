use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::core::{Deserialize, DeviceId, HashMap, ScriptId, Serialize};
use crate::infrastructure::scripts::script_info_model::ScriptMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInfo {
    #[serde(flatten)]
    pub script_meta: ScriptMeta,

    // 设置的变量
    pub script_args: HashMap<String, String>,

    // 设置的策略
    pub decision: Vec<Step>,
    pub back_decision: Vec<Step>,
    pub global_decision: Vec<Step>, //设置的任务信息

                                    //设置的设置模板

                                    //设置的账号信息
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeType {
    BuildIn,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ScriptType {
    Local,
    Cloud,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptShow {
    pub script_id: ScriptId,
    pub script_info: ScriptInfo,
    pub target_device : Option<Vec<DeviceId>>,
    pub device_account: Option<HashMap<DeviceId, HashMap<String, String>>>,
}
