use crate::domain::scripts::nodes::data_handing::VarValue;
use crate::domain::scripts::nodes::task_control::{StateTarget, TaskControl};
use crate::domain::scripts::script_decision::Step;
use crate::domain::vision::ocr_search::{LogicOp, PolicyConditionRule};
use crate::infrastructure::core::{Deserialize, PolicyId, PolicySetId, Serialize, TaskId};

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
    ForEach {
        input_var: String,
        item_var: String,
        index_var: String,
        flow: Vec<Step>,
    },
    Continue,
    Break,
    WaitMs {
        ms: u64,
    },
    Link {
        target: TaskId,
    },
    AddPolicies{
        source: PolicySetId,
        target: PolicySetId,
    },
    HandlePolicySet{
        target: Vec<PolicySetId>,
        input_var: String,
        out_var: String,
    },
    HandlePolicy{
        target: Vec<PolicyId>,
        input_var: String,
        out_var: String,
    },
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

    /// 策略集处理结果判断
    PolicySetResult {
        result_var: String,
        field: PolicySetResultField,
        op: PolicySetResultCompareOp,
        value_bool: bool,
        value_id: String,
    },

    /// 视觉精判规则，可在线性步骤流中使用
    PolicyCondition {
        input_var: Option<String>,
        rule: PolicyConditionRule,
    },

    Group {
        op: LogicOp,
        items: Vec<ConditionNode>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicySetResultField {
    Matched,
    PolicySetId,
    PolicyGroupId,
    PolicyId,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicySetResultCompareOp {
    Eq,
    Ne,
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
