use crate::{PolicyGroupInfo, PolicyInfo, PolicySetInfo, ScriptTask, TaskCycle};
use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, ScriptId, TaskId};

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyProfile {
    pub id: PolicyId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub info: PolicyInfo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupProfile {
    pub id: PolicyGroupId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub info: PolicyGroupInfo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetProfile {
    pub id: PolicySetId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub info: PolicySetInfo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupPolicyLink {
    pub group_id: PolicyGroupId,
    pub policy_id: PolicyId,
    pub order_index: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetGroupLink {
    pub set_id: PolicySetId,
    pub group_id: PolicyGroupId,
    pub order_index: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTaskProfile {
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
    pub default_task_cycle: TaskCycle,
    pub exec_max: u32,
    #[serde(default = "default_show_enabled_toggle")]
    pub show_enabled_toggle: bool,
    #[serde(default = "default_default_enabled")]
    pub default_enabled: bool,
    #[serde(default = "default_task_tone")]
    pub task_tone: TaskTone,
    #[serde(default)]
    pub is_hidden: bool,
    pub task: ScriptTask,
    #[ts(type = "string")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string | null")]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub is_deleted: bool,
    pub index: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TaskRowType {
    Task,
    Title,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TaskTriggerMode {
    RootOnly,
    LinkOnly,
    RootAndLink,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TaskTone {
    Normal,
    Warning,
    Danger,
}
