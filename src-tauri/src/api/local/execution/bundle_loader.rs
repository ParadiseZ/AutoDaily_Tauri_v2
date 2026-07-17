mod queue_loader;
mod target_validation;

use ad_kernel::ids::{ScriptId, TaskId};
use domain_schedule::PlannerQueueItem;
use domain_script::ScriptType;
use runner_protocol::message::{RunTarget, RuntimeQueueItem, ScriptBundleSnapshot};
use std::collections::HashSet;

pub(super) struct LoadedScriptBundle {
    pub(super) script_id: ScriptId,
    pub(super) script_name: String,
    pub(super) script_type: ScriptType,
    pub(super) runnable_task_ids: HashSet<TaskId>,
    pub(super) policy_ids: HashSet<ad_kernel::ids::PolicyId>,
    pub(super) policy_group_ids: HashSet<ad_kernel::ids::PolicyGroupId>,
    pub(super) policy_set_ids: HashSet<ad_kernel::ids::PolicySetId>,
    pub(super) snapshot: ScriptBundleSnapshot,
}

pub(super) async fn load_runtime_queue(
    device_id: ad_kernel::ids::DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    queue_loader::load_runtime_queue(device_id).await
}

pub(crate) async fn load_runtime_queue_for_current_window(
    device_id: ad_kernel::ids::DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    queue_loader::load_runtime_queue_for_current_window(device_id).await
}

pub(crate) fn planner_queue_items(items: &[RuntimeQueueItem]) -> Vec<PlannerQueueItem> {
    items
        .iter()
        .map(|item| PlannerQueueItem {
            assignment_id: item.assignment_id,
            script_id: item.script_id,
            time_template_id: item.time_template_id,
            window_start_at: item.window_start_at.clone(),
            scope_hash: item.dedup_scope_base_hash.clone(),
            dispatch_id: item.dispatch_id,
            order_index: item.order_index,
        })
        .collect()
}

pub(super) async fn load_script_bundles(
    run_target: &RunTarget,
    queue: &[RuntimeQueueItem],
) -> Result<Vec<LoadedScriptBundle>, String> {
    queue_loader::load_script_bundles(run_target, queue).await
}

pub(super) fn validate_run_target_support(
    run_target: &RunTarget,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    target_validation::validate_run_target_support(run_target, bundles)
}
