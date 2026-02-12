use crate::infrastructure::core::{Deserialize, Serialize, TaskId, ScriptId};
use sqlx::types::Json;
use sqlx::FromRow;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTask {
    /// 自定义 UI 数据
    pub ui_data: Value,
    /// 可更改的变量数据
    pub variables: Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTaskTable {
    pub id: TaskId,
    pub script_id: ScriptId,
    pub name: String,
    pub is_hidden: bool,
    #[ts(as = "serde_json::Value")]
    pub nodes: Json<Value>,
    #[ts(as = "serde_json::Value")]
    pub edges: Json<Value>,
    #[ts(as = "ScriptTask")]
    pub data: Json<ScriptTask>,
}
