use crate::domain::devices::device_schedule::TaskCycle;
use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::core::{Deserialize, ScriptId, Serialize, TaskId};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::FromRow;

fn default_trigger_mode() -> TaskTriggerMode {
    TaskTriggerMode::RootOnly
}

fn default_record_schedule() -> bool {
    true
}

fn default_show_enabled_toggle() -> bool {
    true
}

fn default_default_enabled() -> bool {
    true
}

fn default_task_tone() -> TaskTone {
    TaskTone::Normal
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTask {
    /// 自定义 UI 数据
    pub ui_data: Value,
    /// 可更改的变量数据
    pub variables: Value,
    /// 任务数据
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTaskTable {
    pub id: TaskId,
    pub script_id: ScriptId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub row_type: TaskRowType,
    #[serde(default = "default_trigger_mode")]
    pub trigger_mode: TaskTriggerMode,
    #[serde(default = "default_record_schedule")]
    pub record_schedule: bool,
    pub section_id: Option<TaskId>,
    pub indent_level: u32,
    #[ts(as = "TaskCycle")]
    pub default_task_cycle: Json<TaskCycle>,
    pub exec_max: u32,
    #[serde(default = "default_show_enabled_toggle")]
    pub show_enabled_toggle: bool,
    #[serde(default = "default_default_enabled")]
    pub default_enabled: bool,
    #[serde(default = "default_task_tone")]
    pub task_tone: TaskTone,
    #[serde(default)]
    pub is_hidden: bool,
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
    #[serde(default)]
    pub is_deleted: bool,
    pub index: u32, //排序
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub enum TaskRowType {
    Task,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub enum TaskTriggerMode {
    RootOnly,
    LinkOnly,
    RootAndLink,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
pub enum TaskTone {
    Normal,
    Warning,
    Danger,
}
