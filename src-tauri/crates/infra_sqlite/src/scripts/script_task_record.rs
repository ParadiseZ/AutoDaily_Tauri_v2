use ad_kernel::ids::UuidV7;
use domain_script::{ScriptTask, ScriptTaskProfile, TaskCycle};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(FromRow)]
pub(crate) struct ScriptTaskRow {
    id: String,
    script_id: String,
    name: String,
    description: String,
    row_type: String,
    trigger_mode: String,
    record_schedule: bool,
    section_id: Option<String>,
    indent_level: i64,
    default_task_cycle: Json<TaskCycle>,
    exec_max: i64,
    show_enabled_toggle: bool,
    default_enabled: bool,
    task_tone: String,
    is_hidden: bool,
    data: Json<ScriptTask>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    is_deleted: bool,
    index: i64,
}

fn id(value: String) -> Result<UuidV7, String> {
    Uuid::parse_str(&value)
        .map(Into::into)
        .map_err(|error| error.to_string())
}

fn enum_value<T: serde::de::DeserializeOwned>(value: String) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(value)).map_err(|error| error.to_string())
}

impl TryFrom<ScriptTaskRow> for ScriptTaskProfile {
    type Error = String;

    fn try_from(row: ScriptTaskRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id(row.id)?,
            script_id: id(row.script_id)?,
            name: row.name,
            description: row.description,
            row_type: enum_value(row.row_type)?,
            trigger_mode: enum_value(row.trigger_mode)?,
            record_schedule: row.record_schedule,
            section_id: row.section_id.map(id).transpose()?,
            indent_level: row
                .indent_level
                .try_into()
                .map_err(|_| "任务缩进层级无效".to_string())?,
            default_task_cycle: row.default_task_cycle.0,
            exec_max: row
                .exec_max
                .try_into()
                .map_err(|_| "任务最大执行次数无效".to_string())?,
            show_enabled_toggle: row.show_enabled_toggle,
            default_enabled: row.default_enabled,
            task_tone: enum_value(row.task_tone)?,
            is_hidden: row.is_hidden,
            task: row.data.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
            is_deleted: row.is_deleted,
            index: row
                .index
                .try_into()
                .map_err(|_| "任务排序值无效".to_string())?,
        })
    }
}
