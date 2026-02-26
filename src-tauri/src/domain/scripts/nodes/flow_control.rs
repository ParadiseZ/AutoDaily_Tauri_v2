use crate::domain::scripts::nodes::data_handing::VarValue;
use crate::domain::scripts::nodes::task_control::{StateTarget, TaskControl};
use crate::domain::scripts::script_decision::Step;
use crate::domain::vision::ocr_search::LogicOp;
use crate::infrastructure::core::{Deserialize, PolicySetId, Serialize, TaskId};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FlowControl{
    If {
        con: ConditionNode,
        then: Vec<Step>,
        else_steps: Option<Vec<Step>>,
    },
    While{ con: ConditionNode,flow: Vec<Step> },
    For{ con: ConditionNode,flow: Vec<Step> },
    Continue,
    Break,
    WaitMs {
        ms: u64,
    },
    Link{
        target: TaskId,
    },
    AddPolicies{
        source: PolicySetId,
        target: PolicySetId,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConditionNode {
    /// rhai表达式
    RawExpr { expr: String },

    /// 执行次数
    ExecNumCompare{
        a: StateTarget,
    },

    /// 策略/任务状态是否完成/跳过
    TaskStatus {
        a: TaskControl,
    },

    /// ocr字体颜色/背景色判断
    ColorCompare{
        txt_target : String,
        is_font: bool,
        r: u8,
        g: u8,
        b: u8,
    },

    /// 变量比较
    VarCompare { var_name: String, op: CompareOp, value: VarValue },

    Group {
        op: LogicOp,
        items: Vec<ConditionNode>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Contains,
    NotContains,
}