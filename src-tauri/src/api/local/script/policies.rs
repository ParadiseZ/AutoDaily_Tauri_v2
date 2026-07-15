use crate::api::local::script::dto::{PolicyGroupTable, PolicySetTable, PolicyTable};
use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, ScriptId};
use domain_script::{PolicyGroupProfile, PolicyProfile, PolicySetProfile};
use infra_sqlite::{
    delete_policy, delete_policy_group, delete_policy_set, list_group_ids_in_set, list_policies,
    list_policy_groups, list_policy_ids_in_group, list_policy_sets, replace_group_policy_links,
    replace_set_group_links, save_policy, save_policy_group, save_policy_set,
};
use tauri::command;

#[command]
pub async fn get_all_policies_cmd(script_id: ScriptId) -> Result<Vec<PolicyTable>, String> {
    list_policies(script_id)
        .await
        .map(|policies| policies.into_iter().map(Into::into).collect())
}

#[command]
pub async fn save_policy_cmd(policy: PolicyTable) -> Result<(), String> {
    let policy: PolicyProfile = policy.into();
    save_policy(&policy).await
}

#[command]
pub async fn delete_policy_cmd(id: PolicyId) -> Result<(), String> {
    delete_policy(id).await
}

#[command]
pub async fn get_all_policy_groups_cmd(
    script_id: ScriptId,
) -> Result<Vec<PolicyGroupTable>, String> {
    list_policy_groups(script_id)
        .await
        .map(|groups| groups.into_iter().map(Into::into).collect())
}

#[command]
pub async fn save_policy_group_cmd(group: PolicyGroupTable) -> Result<(), String> {
    let group: PolicyGroupProfile = group.into();
    save_policy_group(&group).await
}

#[command]
pub async fn delete_policy_group_cmd(id: PolicyGroupId) -> Result<(), String> {
    delete_policy_group(id).await
}

#[command]
pub async fn get_group_policies_cmd(group_id: PolicyGroupId) -> Result<Vec<PolicyId>, String> {
    list_policy_ids_in_group(group_id).await
}

#[command]
pub async fn update_group_policies_cmd(
    group_id: PolicyGroupId,
    policy_ids: Vec<PolicyId>,
) -> Result<(), String> {
    replace_group_policy_links(group_id, policy_ids).await
}

#[command]
pub async fn get_all_policy_sets_cmd(script_id: ScriptId) -> Result<Vec<PolicySetTable>, String> {
    list_policy_sets(script_id)
        .await
        .map(|sets| sets.into_iter().map(Into::into).collect())
}

#[command]
pub async fn save_policy_set_cmd(set: PolicySetTable) -> Result<(), String> {
    let set: PolicySetProfile = set.into();
    save_policy_set(&set).await
}

#[command]
pub async fn delete_policy_set_cmd(id: PolicySetId) -> Result<(), String> {
    delete_policy_set(id).await
}

#[command]
pub async fn get_set_groups_cmd(set_id: PolicySetId) -> Result<Vec<PolicyGroupId>, String> {
    list_group_ids_in_set(set_id).await
}

#[command]
pub async fn update_set_groups_cmd(
    set_id: PolicySetId,
    group_ids: Vec<PolicyGroupId>,
) -> Result<(), String> {
    replace_set_group_links(set_id, group_ids).await
}
