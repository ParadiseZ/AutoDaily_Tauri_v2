#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum AssignmentScheduleStatus {
    Planned,
    Dispatched,
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
    Stopped,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum AssignmentTriggerSource {
    Planner,
    User,
    Debug,
}
