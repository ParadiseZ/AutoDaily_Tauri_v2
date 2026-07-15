use ad_kernel::ids::UuidV7;
use domain_script::{PolicyGroupInfo, PolicyInfo, PolicySetInfo};
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(FromRow)]
pub(crate) struct PolicyRow {
    id: String,
    script_id: String,
    order_index: i32,
    data: Json<PolicyInfo>,
}
#[derive(FromRow)]
pub(crate) struct PolicyGroupRow {
    id: String,
    script_id: String,
    order_index: i32,
    data: Json<PolicyGroupInfo>,
}
#[derive(FromRow)]
pub(crate) struct PolicySetRow {
    id: String,
    script_id: String,
    order_index: i32,
    data: Json<PolicySetInfo>,
}
#[derive(FromRow)]
pub(crate) struct GroupPolicyRow {
    group_id: String,
    policy_id: String,
    order_index: i32,
}
#[derive(FromRow)]
pub(crate) struct SetGroupRow {
    set_id: String,
    group_id: String,
    order_index: i32,
}

fn id(value: String) -> Result<UuidV7, String> {
    Uuid::parse_str(&value)
        .map(Into::into)
        .map_err(|error| error.to_string())
}

impl TryFrom<PolicyRow> for PolicyProfile {
    type Error = String;
    fn try_from(row: PolicyRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id(row.id)?,
            script_id: id(row.script_id)?,
            order_index: row.order_index,
            info: row.data.0,
        })
    }
}

impl TryFrom<PolicyGroupRow> for PolicyGroupProfile {
    type Error = String;
    fn try_from(row: PolicyGroupRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id(row.id)?,
            script_id: id(row.script_id)?,
            order_index: row.order_index,
            info: row.data.0,
        })
    }
}

impl TryFrom<PolicySetRow> for PolicySetProfile {
    type Error = String;
    fn try_from(row: PolicySetRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id(row.id)?,
            script_id: id(row.script_id)?,
            order_index: row.order_index,
            info: row.data.0,
        })
    }
}

impl TryFrom<GroupPolicyRow> for PolicyGroupPolicyLink {
    type Error = String;
    fn try_from(row: GroupPolicyRow) -> Result<Self, Self::Error> {
        Ok(Self {
            group_id: id(row.group_id)?,
            policy_id: id(row.policy_id)?,
            order_index: row.order_index,
        })
    }
}

impl TryFrom<SetGroupRow> for PolicySetGroupLink {
    type Error = String;
    fn try_from(row: SetGroupRow) -> Result<Self, Self::Error> {
        Ok(Self {
            set_id: id(row.set_id)?,
            group_id: id(row.group_id)?,
            order_index: row.order_index,
        })
    }
}
