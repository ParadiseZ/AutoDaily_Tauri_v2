use ad_kernel::ids::{PolicyId, TaskId};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum TaskControl {
    SetState {
        target: StateTarget,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        targets: Vec<StateTarget>,
        status: StateStatus,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum StateTarget {
    Policy { id: PolicyId },
    Task { id: TaskId },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum StateStatus {
    Enabled { value: bool },
    Skip { value: bool },
    Done { value: bool },
}
