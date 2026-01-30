use crate::infrastructure::core::{Deserialize, Serialize, TaskId, ScriptId};
use sqlx::types::Json;
use sqlx::FromRow;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTask {
    /// 自定义 UI 数据
    pub ui_data: Value,
    /// 可更改的变量数据
    pub variables: Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTaskTable {
    pub id: TaskId,
    pub script_id: ScriptId,
    pub name: String,
    pub is_hidden: bool,
    pub nodes: Json<Value>,
    pub edges: Json<Value>,
    pub data: Json<ScriptTask>,
}
