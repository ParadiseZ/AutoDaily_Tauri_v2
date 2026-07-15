use crate::{Action, DataHanding, FlowControl, TaskControl, VisionNode};
use ad_kernel::ids::StepId;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
pub struct Step {
    pub id: Option<StepId>,
    pub source_id: Option<StepId>,
    pub target_id: Option<StepId>,
    pub label: Option<String>,
    #[serde(default)]
    pub skip_flag: bool,
    #[serde(flatten)]
    pub kind: StepKind,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "op")]
pub enum StepKind {
    Sequence { steps: Vec<Step> },
    Action { exec_max: u32, a: Action },
    DataHanding { a: DataHanding },
    FlowControl { a: FlowControl },
    TaskControl { a: TaskControl },
    Vision { a: VisionNode },
}
