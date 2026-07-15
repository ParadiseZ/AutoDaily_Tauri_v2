use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId};
use domain_script::PointU16;
use domain_vision::BoundingBox;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PolicyActionKind {
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PolicyActionSource {
    Ocr,
    Det,
    Label,
    Fixed,
    Text,
    Custom,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PolicyActionTargetRole {
    Primary,
    Secondary,
    Start,
    End,
    Path,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolicyActionTarget {
    pub role: PolicyActionTargetRole,
    pub point: Option<PointU16>,
    pub box_area: Option<BoundingBox>,
    pub text: Option<String>,
    pub label_id: Option<i32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolicyActionTrace {
    pub action_index: u16,
    pub kind: PolicyActionKind,
    pub source: PolicyActionSource,
    pub signature: String,
    pub targets: Vec<PolicyActionTarget>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolicyExecutionRound {
    pub matched: bool,
    pub policy_set_id: Option<PolicySetId>,
    pub policy_group_id: Option<PolicyGroupId>,
    pub policy_id: Option<PolicyId>,
    pub page_fingerprints: Vec<String>,
    pub action_signatures: Vec<String>,
    pub actions: Vec<PolicyActionTrace>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolicyExecutionResult {
    pub matched: bool,
    pub policy_set_id: Option<PolicySetId>,
    pub policy_group_id: Option<PolicyGroupId>,
    pub policy_id: Option<PolicyId>,
    pub rounds: Vec<PolicyExecutionRound>,
}
