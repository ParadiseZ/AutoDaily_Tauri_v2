use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FlowControl{
    If {
        then: Vec<Step>,
        else_steps: Option<Vec<Step>>,
    },
    While{ flow: Vec<Step> },
    For{ flow: Vec<Step> },
    Continue,
    Break,
    WaitMs {
        ms: u64,
    },
}