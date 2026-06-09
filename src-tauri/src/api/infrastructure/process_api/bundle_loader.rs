mod queue_loader;
mod target_validation;

use crate::domain::scripts::script_info::ScriptType;
use crate::infrastructure::core::{ScriptId, TaskId};
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem, ScriptBundleSnapshot,
};
use std::collections::HashSet;

pub(super) struct LoadedScriptBundle {
    pub(super) script_id: ScriptId,
    pub(super) script_name: String,
    pub(super) script_type: ScriptType,
    pub(super) recovery_task_id: Option<TaskId>,
    pub(super) runnable_task_ids: HashSet<TaskId>,
    pub(super) policy_ids: HashSet<crate::infrastructure::core::PolicyId>,
    pub(super) policy_group_ids: HashSet<crate::infrastructure::core::PolicyGroupId>,
    pub(super) policy_set_ids: HashSet<crate::infrastructure::core::PolicySetId>,
    pub(super) snapshot: ScriptBundleSnapshot,
}

pub(super) async fn load_runtime_queue(
    device_id: crate::infrastructure::core::DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    queue_loader::load_runtime_queue(device_id).await
}

pub(crate) async fn load_runtime_queue_for_current_window(
    device_id: crate::infrastructure::core::DeviceId,
) -> Result<Vec<RuntimeQueueItem>, String> {
    queue_loader::load_runtime_queue_for_current_window(device_id).await
}

pub(super) async fn load_script_bundles(
    run_target: &RunTarget,
    queue: &[RuntimeQueueItem],
) -> Result<Vec<LoadedScriptBundle>, String> {
    queue_loader::load_script_bundles(run_target, queue).await
}

pub(super) fn validate_recovery_task_config(
    run_target: &RunTarget,
    runtime_policy: &RuntimeExecutionPolicy,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    target_validation::validate_recovery_task_config(run_target, runtime_policy, bundles)
}

pub(super) fn validate_run_target_support(
    run_target: &RunTarget,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    target_validation::validate_run_target_support(run_target, bundles)
}
