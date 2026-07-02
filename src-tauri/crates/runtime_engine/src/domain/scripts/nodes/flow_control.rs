use crate::domain::scripts::nodes::data_handing::VarValue;
use crate::domain::scripts::nodes::task_control::{StateTarget, TaskControl};
use crate::domain::scripts::script_decision::Step;
use crate::domain::vision::ocr_search::LogicOp;
use crate::infrastructure::core::{
    Deserialize, PolicyGroupId, PolicyId, PolicySetId, Serialize, TaskId,
};

fn default_policy_set_det_input_var() -> String {
    "runtime.detResults".to_string()
}

fn default_policy_set_ocr_input_var() -> String {
    "runtime.ocrResults".to_string()
}

fn default_policy_set_search_hits_var() -> String {
    "runtime.searchHits".to_string()
}

fn default_current_task_logic_op() -> LogicOp {
    LogicOp::Or
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FlowControl {
    If {
        con: ConditionNode,
        then: Vec<Step>,
        else_steps: Option<Vec<Step>>,
    },
    While {
        con: ConditionNode,
        flow: Vec<Step>,
    },
    ForEach {
        input_var: String,
        item_var: String,
        index_var: String,
        flow: Vec<Step>,
    },
    Repeat {
        count_expr: String,
        index_var: String,
        flow: Vec<Step>,
    },
    Continue,
    Break,
    StopScript,
    WaitMs {
        ms: u64,
        input_var: Option<String>,
        runtime_var: Option<String>,
    },
    Link {
        target: TaskId,
    },
    AddPolicies {
        source: PolicySetId,
        target: PolicySetId,
        #[serde(default)]
        top: bool,
        #[serde(default)]
        reverse: bool,
    },
    RemovePolicies {
        source: PolicySetId,
        target: PolicySetId,
    },
    BindPolicyGroup {
        source: PolicyGroupId,
        target: PolicySetId,
        #[serde(default)]
        top: bool,
        #[serde(default)]
        reverse: bool,
    },
    RemovePolicyGroup {
        source: PolicyGroupId,
        target: PolicySetId,
    },
    AddPolicyGroups {
        source: PolicyGroupId,
        target: PolicyGroupId,
        #[serde(default)]
        top: bool,
        #[serde(default)]
        reverse: bool,
    },
    UnloadPolicyGroup {
        source: PolicyGroupId,
        target: PolicyGroupId,
    },
    BindPolicy {
        source: PolicyId,
        target: PolicyGroupId,
        #[serde(default)]
        top: bool,
        #[serde(default)]
        reverse: bool,
    },
    UnloadPolicy {
        source: PolicyId,
        target: PolicyGroupId,
    },
    HandlePolicySet {
        target: Vec<PolicySetId>,
        #[serde(default = "default_policy_set_det_input_var")]
        det_input_var: String,
        #[serde(default = "default_policy_set_ocr_input_var")]
        ocr_input_var: String,
        #[serde(default = "default_policy_set_search_hits_var")]
        search_hits_var: String,
        out_var: String,
    },
    HandlePolicy {
        target: Vec<PolicyId>,
        input_var: String,
        out_var: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CurrentTaskRule {
    Task { target: TaskId },
    Group {
        #[serde(default = "default_current_task_logic_op")]
        op: LogicOp,
        #[serde(default)]
        items: Vec<CurrentTaskRule>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConditionNode {
    /// rhai表达式
    RawExpr { expr: String },

    /// 执行次数
    ExecNumCompare { target: StateTarget, op: CompareOp },

    /// 策略/任务状态是否完成/跳过
    TaskStatus { a: TaskControl },

    /// 当前正在执行的任务是否命中指定逻辑组
    CurrentTaskIn {
        #[serde(default = "default_current_task_logic_op")]
        op: LogicOp,
        #[serde(default)]
        items: Vec<CurrentTaskRule>,
        #[serde(default)]
        targets: Vec<TaskId>,
    },

    /// ocr字体颜色/背景色判断
    ColorCompare {
        txt_target: String,
        is_font: bool,
        r: u8,
        g: u8,
        b: u8,
    },

    /// 变量比较
    VarCompare {
        var_name: String,
        op: CompareOp,
        value: VarValue,
    },

    /// 统计检测标签或 OCR 文字的匹配数量，并和指定数量比较
    VisionCountCompare {
        input_var: String,
        target_value: Option<String>,
        op: CompareOp,
        expected_count: i32,
    },

    /// 策略集处理结果判断
    PolicySetResult {
        result_var: String,
        field: PolicySetResultField,
        op: PolicySetResultCompareOp,
        value_bool: bool,
        value_id: String,
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
