use crate::domain::scripts::point::PointU16;
use crate::domain::vision::result::BoundingBox;
use crate::infrastructure::core::{PolicyGroupId, PolicyId, PolicySetId};
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicyActionKind {
    Unknown,
    Click,
    Swipe,
    Input,
    Press,
    Reboot,
    StartApp,
    StopApp,
    Back,
    Home,
    Menu,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicyActionSource {
    Ocr,
    Det,
    Label,
    Fixed,
    Text,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PolicyActionTargetRole {
    Primary,
    Secondary,
    Start,
    End,
    Path,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyActionTarget {
    pub role: PolicyActionTargetRole,
    pub point: Option<PointU16>,
    pub box_area: Option<BoundingBox>,
    pub text: Option<String>,
    pub label_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyActionTrace {
    /// 单轮执行内的动作序号。用它区分“第一下点击 / 第二下点击”这类场景。
    pub action_index: u16,
    pub kind: PolicyActionKind,
    pub source: PolicyActionSource,
    pub signature: String,
    pub targets: Vec<PolicyActionTarget>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyExecutionRound {
    /// 本轮是否选中了有效策略。
    pub matched: bool,
    pub policy_set_id: Option<PolicySetId>,
    pub policy_group_id: Option<PolicyGroupId>,
    pub policy_id: Option<PolicyId>,
    /// 单轮策略执行过程中采集到的页面轨迹。
    pub page_fingerprints: Vec<String>,
    /// 单轮策略执行过程中记录的动作签名序列。
    pub action_signatures: Vec<String>,
    /// 单轮策略执行过程中的结构化动作详情。
    pub actions: Vec<PolicyActionTrace>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyExecutionResult {
    /// 顶层字段表示本次 HandlePolicy / HandlePolicySet 的最终摘要结果。
    pub matched: bool,
    pub policy_set_id: Option<PolicySetId>,
    pub policy_group_id: Option<PolicyGroupId>,
    pub policy_id: Option<PolicyId>,
    /// 逐轮执行明细。每一轮都记录自己的页面轨迹与动作序列，避免和顶层摘要冲突。
    pub rounds: Vec<PolicyExecutionRound>,
}
