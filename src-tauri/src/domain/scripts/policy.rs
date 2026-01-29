use crate::infrastructure::core::{Deserialize, PolicyId, PolicyGroupId, PolicySetId, ScriptId, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;
use crate::domain::scripts::action::click::Click;
use crate::domain::scripts::script_decision::Step;
use crate::domain::vision::ocr_search::SearchRule;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTable {
    pub id: PolicyId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub data: Json<PolicyInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInfo {
    pub name: String,
    pub log_print: Option<String>,

    pub cur_pos: u16,

    pub skip_flag : bool,
    pub exec_cur: u16,
    pub exec_max: u16,

    pub cond: SearchRule,
    pub action: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupTable {
    pub id: PolicyGroupId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub data: Json<PolicyGroupInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupInfo {
    pub name: String,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetTable {
    pub id: PolicySetId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub data: Json<PolicySetInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetInfo {
    pub name: String,
    pub note: String,
}

// Structs for Many-to-Many fetching
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GroupPolicyRelation {
    pub group_id: PolicyGroupId,
    pub policy_id: PolicyId,
    pub order_index: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SetGroupRelation {
    pub set_id: PolicySetId,
    pub group_id: PolicyGroupId,
    pub order_index: i32,
}
