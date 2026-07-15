use ad_kernel::ids::{
    AccountId, AssignmentId, AssignmentScheduleId, BatchId, DeviceId, DispatchId, ExecutionId,
    ScheduleId, ScriptId, ScriptTemplateValueId, TaskId, TemplateId,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentProfile {
    pub id: AssignmentId,
    pub device_id: DeviceId,
    pub script_id: ScriptId,
    pub time_template_id: Option<TemplateId>,
    #[ts(type = "any")]
    pub account_data: serde_json::Value,
    pub index: u32,
}

impl Default for AssignmentProfile {
    fn default() -> Self {
        Self {
            id: AssignmentId::new_v7(),
            device_id: DeviceId::new_v7(),
            script_id: ScriptId::new_v7(),
            time_template_id: None,
            account_data: serde_json::Value::Null,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionScheduleProfile {
    pub id: ScheduleId,
    pub device_id: DeviceId,
    pub execution_id: Option<ExecutionId>,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: ScriptId,
    pub task_id: TaskId,
    pub dedup_scope_hash: String,
    pub task_cycle: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentScheduleProfile {
    pub id: AssignmentScheduleId,
    pub batch_id: BatchId,
    pub device_id: DeviceId,
    pub assignment_id: Option<AssignmentId>,
    pub script_id: Option<ScriptId>,
    pub time_template_id: Option<TemplateId>,
    pub window_start_at: Option<String>,
    pub scope_hash: String,
    pub dispatch_id: DispatchId,
    pub order_index: u32,
    pub created_at: String,
    pub run_target_json: Option<String>,
    pub status: String,
    pub trigger_source: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlannerQueueItem {
    pub assignment_id: AssignmentId,
    pub script_id: ScriptId,
    pub time_template_id: Option<TemplateId>,
    pub window_start_at: Option<String>,
    pub scope_hash: String,
    pub dispatch_id: DispatchId,
    pub order_index: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TimeTemplateProfile {
    pub id: TemplateId,
    pub name: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl Default for TimeTemplateProfile {
    fn default() -> Self {
        Self {
            id: TemplateId::new_v7(),
            name: String::new(),
            start_time: None,
            end_time: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TemplateValueProfile {
    pub id: ScriptTemplateValueId,
    pub device_id: Option<DeviceId>,
    pub script_id: ScriptId,
    pub time_template_id: TemplateId,
    pub account_id: Option<AccountId>,
    #[ts(type = "any")]
    pub values: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for TemplateValueProfile {
    fn default() -> Self {
        Self {
            id: ScriptTemplateValueId::new_v7(),
            device_id: None,
            script_id: ScriptId::new_v7(),
            time_template_id: TemplateId::new_v7(),
            account_id: None,
            values: serde_json::json!({}),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
