use crate::Step;
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTask {
    pub ui_data: Value,
    pub variables: Value,
    pub steps: Vec<Step>,
}
