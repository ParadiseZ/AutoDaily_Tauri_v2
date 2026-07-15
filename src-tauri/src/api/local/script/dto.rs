use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, ScriptId, TaskId};
use domain_script::{
    PolicyGroupInfo, PolicyGroupProfile, PolicyInfo, PolicyProfile, PolicySetInfo,
    PolicySetProfile, ScriptInfo, ScriptProfile, ScriptTask, ScriptTaskProfile, TaskCycle,
    TaskRowType, TaskTone, TaskTriggerMode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTable {
    pub id: ScriptId,
    pub data: ScriptInfo,
}

impl From<ScriptProfile> for ScriptTable {
    fn from(script: ScriptProfile) -> Self {
        Self {
            id: script.id,
            data: script.info,
        }
    }
}

impl From<ScriptTable> for ScriptProfile {
    fn from(script: ScriptTable) -> Self {
        Self {
            id: script.id,
            info: script.data,
        }
    }
}

macro_rules! table_dto {
    ($table:ident, $profile:ident, $id:ty, $info:ty) => {
        #[derive(Debug, Serialize, Deserialize, TS)]
        #[ts(export)]
        #[serde(rename_all = "camelCase")]
        pub struct $table {
            pub id: $id,
            pub script_id: ScriptId,
            pub order_index: i32,
            pub data: $info,
        }

        impl From<$profile> for $table {
            fn from(item: $profile) -> Self {
                Self {
                    id: item.id,
                    script_id: item.script_id,
                    order_index: item.order_index,
                    data: item.info,
                }
            }
        }

        impl From<$table> for $profile {
            fn from(item: $table) -> Self {
                Self {
                    id: item.id,
                    script_id: item.script_id,
                    order_index: item.order_index,
                    info: item.data,
                }
            }
        }
    };
}

table_dto!(PolicyTable, PolicyProfile, PolicyId, PolicyInfo);
table_dto!(
    PolicyGroupTable,
    PolicyGroupProfile,
    PolicyGroupId,
    PolicyGroupInfo
);
table_dto!(PolicySetTable, PolicySetProfile, PolicySetId, PolicySetInfo);

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTaskTable {
    pub id: TaskId,
    pub script_id: ScriptId,
    pub name: String,
    pub description: String,
    pub row_type: TaskRowType,
    pub trigger_mode: TaskTriggerMode,
    pub record_schedule: bool,
    pub section_id: Option<TaskId>,
    pub indent_level: u32,
    pub default_task_cycle: TaskCycle,
    pub exec_max: u32,
    pub show_enabled_toggle: bool,
    pub default_enabled: bool,
    pub task_tone: TaskTone,
    pub is_hidden: bool,
    pub data: ScriptTask,
    #[ts(type = "string")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string | null")]
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_deleted: bool,
    pub index: u32,
}

impl From<ScriptTaskProfile> for ScriptTaskTable {
    fn from(task: ScriptTaskProfile) -> Self {
        Self {
            id: task.id,
            script_id: task.script_id,
            name: task.name,
            description: task.description,
            row_type: task.row_type,
            trigger_mode: task.trigger_mode,
            record_schedule: task.record_schedule,
            section_id: task.section_id,
            indent_level: task.indent_level,
            default_task_cycle: task.default_task_cycle,
            exec_max: task.exec_max,
            show_enabled_toggle: task.show_enabled_toggle,
            default_enabled: task.default_enabled,
            task_tone: task.task_tone,
            is_hidden: task.is_hidden,
            data: task.task,
            created_at: task.created_at,
            updated_at: task.updated_at,
            deleted_at: task.deleted_at,
            is_deleted: task.is_deleted,
            index: task.index,
        }
    }
}

impl From<ScriptTaskTable> for ScriptTaskProfile {
    fn from(task: ScriptTaskTable) -> Self {
        Self {
            id: task.id,
            script_id: task.script_id,
            name: task.name,
            description: task.description,
            row_type: task.row_type,
            trigger_mode: task.trigger_mode,
            record_schedule: task.record_schedule,
            section_id: task.section_id,
            indent_level: task.indent_level,
            default_task_cycle: task.default_task_cycle,
            exec_max: task.exec_max,
            show_enabled_toggle: task.show_enabled_toggle,
            default_enabled: task.default_enabled,
            task_tone: task.task_tone,
            is_hidden: task.is_hidden,
            task: task.data,
            created_at: task.created_at,
            updated_at: task.updated_at,
            deleted_at: task.deleted_at,
            is_deleted: task.is_deleted,
            index: task.index,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptEditorSaveRequest {
    pub script: ScriptTable,
    pub tasks: Vec<ScriptTaskTable>,
    pub policies: Vec<PolicyTable>,
    pub policy_groups: Vec<PolicyGroupTable>,
    pub policy_sets: Vec<PolicySetTable>,
    pub group_policy_ids_by_group_id: HashMap<String, Vec<String>>,
    pub set_group_ids_by_set_id: HashMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_table_serializes_the_frontend_data_field() {
        let value = serde_json::to_value(ScriptTable::from(ScriptProfile::default())).unwrap();
        assert!(value.get("data").is_some());
        assert!(value.get("info").is_none());
    }
}
