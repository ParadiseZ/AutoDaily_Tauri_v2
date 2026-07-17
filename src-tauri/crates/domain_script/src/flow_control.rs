use crate::{StateTarget, Step, TaskControl, VarValue};
use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, TaskId};
use domain_vision::LogicOp;

fn default_policy_set_det_input_var() -> String {
    "runtime.detResults".to_string()
}
fn default_policy_set_ocr_input_var() -> String {
    "runtime.ocrResults".to_string()
}
fn default_policy_set_search_hits_var() -> String {
    "runtime.searchHits".to_string()
}
fn default_current_task_expected() -> bool {
    true
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
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
    SearchPolicySetText {
        target: Vec<PolicySetId>,
        #[serde(default = "default_policy_set_ocr_input_var")]
        ocr_input_var: String,
        #[serde(default = "default_policy_set_search_hits_var")]
        out_var: String,
    },
    HandlePolicySet {
        target: Vec<PolicySetId>,
        #[serde(default = "default_policy_set_det_input_var")]
        det_input_var: String,
        #[serde(default = "default_policy_set_search_hits_var")]
        search_hits_var: String,
        /// legacy-editor-compat: 仅用于让旧脚本在编辑器中可读取和保存；运行时统一使用 search_hits_var。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ocr_input_var: Option<String>,
        out_var: String,
    },
    HandlePolicy {
        target: Vec<PolicyId>,
        input_var: String,
        out_var: String,
    },
}

#[derive(Debug, serde::Serialize, Clone, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CurrentTaskCondition {
    #[serde(default)]
    pub target: Option<TaskId>,
    #[serde(default = "default_current_task_expected")]
    pub expected: bool,
}

impl<'de> serde::Deserialize<'de> for CurrentTaskCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Compat {
            #[serde(default)]
            target: Option<TaskId>,
            #[serde(default = "default_current_task_expected")]
            expected: bool,
            #[serde(default)]
            targets: Vec<TaskId>,
            #[serde(default)]
            items: Vec<LegacyRule>,
        }
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase", tag = "type")]
        enum LegacyRule {
            Task {
                target: TaskId,
            },
            Group {
                #[serde(default)]
                items: Vec<LegacyRule>,
            },
        }
        fn first(items: &[LegacyRule]) -> Option<TaskId> {
            items.iter().find_map(|item| match item {
                LegacyRule::Task { target } => Some(*target),
                LegacyRule::Group { items } => first(items),
            })
        }
        let compat = Compat::deserialize(deserializer)?;
        Ok(Self {
            target: compat
                .target
                .or_else(|| compat.targets.into_iter().next())
                .or_else(|| first(&compat.items)),
            expected: compat.expected,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConditionNode {
    RawExpr {
        expr: String,
    },
    ExecNumCompare {
        target: StateTarget,
        op: CompareOp,
    },
    TaskStatus {
        a: TaskControl,
    },
    CurrentTaskIn {
        #[serde(flatten)]
        current: CurrentTaskCondition,
    },
    ColorCompare {
        txt_target: String,
        is_font: bool,
        r: u8,
        g: u8,
        b: u8,
    },
    VarCompare {
        var_name: String,
        op: CompareOp,
        value: VarValue,
    },
    VisionCountCompare {
        input_var: String,
        target_value: Option<String>,
        op: CompareOp,
        expected_count: i32,
    },
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicySetResultField {
    Matched,
    PolicySetId,
    PolicyGroupId,
    PolicyId,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicySetResultCompareOp {
    Eq,
    Ne,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
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
