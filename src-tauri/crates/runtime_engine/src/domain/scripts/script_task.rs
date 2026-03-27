use crate::infrastructure::core::{Deserialize, Serialize, TaskId, ScriptId};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::FromRow;
use crate::domain::scripts::script_decision::Step;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTask {
    /// 自定义 UI 数据
    pub ui_data: Value,
    /// 可更改的变量数据
    pub variables: Value,
    /// 任务数据
    pub steps: Vec<Step>
}

#[derive(Debug, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTaskTable {
    pub id: TaskId,
    pub script_id: ScriptId,
    pub name: String,
    pub is_hidden: bool,
    pub task_type: TaskType,
/*    #[ts(type = "Array<import('@vue-flow/core').Node>")]
    pub nodes: Json<Value>,
    #[ts(type = "import('@vue-flow/core').Edge[]")]
    pub edges: Json<Value>,*/
    #[ts(as = "ScriptTask")]
    pub data: Json<ScriptTask>,
    #[ts(type = "string")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string | null")]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_deleted: bool,
    pub index: u32,//排序
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub enum TaskType{
    Main,// 执行的任务（主循环执行）
    Child// 子任务（通过节点的链接功能执行）
}
