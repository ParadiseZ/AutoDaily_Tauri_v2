use crate::infrastructure::core::{Deserialize, PolicyId, Serialize, TaskId};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum TaskControl{
    // 状态与流程管理
    SetState {
        target: StateTarget,
        status: StateStatus,
    },
    GetState {
        target: StateTarget,
        status: StateStatus,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum StateTarget {
    Policy { id: PolicyId },
    Task { id: TaskId },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum StateStatus {
    Skip { value: bool },
    Done { value: bool },
}