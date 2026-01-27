use crate::infrastructure::core::{Deserialize, PolicyId, PolicyGroupId, PolicySetId, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTable {
    pub id: PolicyId,
    pub data: Json<PolicyInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInfo {
    pub name: String,
    pub note: String,
    pub conditions: Vec<String>, // Placeholder for actual condition structure
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupTable {
    pub id: PolicyGroupId,
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
