use super::LoadedScriptBundle;
use ad_kernel::ids::ScriptId;
use runner_protocol::message::RunTarget;

pub(super) fn validate_run_target_support(
    run_target: &RunTarget,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    let find_bundle =
        |script_id: ScriptId| bundles.iter().find(|bundle| bundle.script_id == script_id);

    match run_target {
        RunTarget::DeviceQueue | RunTarget::FullScript { .. } => Ok(()),
        RunTarget::Task { script_id, task_id } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if bundle.runnable_task_ids.contains(task_id) {
                Ok(())
            } else {
                Err(format!(
                    "脚本[{}]中的任务[{}]不存在，或不是可执行 Task",
                    bundle.script_name, task_id
                ))
            }
        }
        RunTarget::Policy {
            script_id,
            policy_id,
        } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if bundle.policy_ids.contains(policy_id) {
                Ok(())
            } else {
                Err(format!(
                    "脚本[{}]中的策略[{}]不存在",
                    bundle.script_name, policy_id
                ))
            }
        }
        RunTarget::PolicyGroup {
            script_id,
            policy_group_id,
        } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if !bundle.policy_group_ids.contains(policy_group_id) {
                return Err(format!(
                    "脚本[{}]中的策略组[{}]不存在",
                    bundle.script_name, policy_group_id
                ));
            }
            Ok(())
        }
        RunTarget::PolicySet {
            script_id,
            policy_set_id,
        } => {
            let bundle = find_bundle(*script_id)
                .ok_or_else(|| format!("运行目标中的脚本[{}]未装入当前 session", script_id))?;
            if !bundle.policy_set_ids.contains(policy_set_id) {
                return Err(format!(
                    "脚本[{}]中的策略集[{}]不存在",
                    bundle.script_name, policy_set_id
                ));
            }
            Ok(())
        }
    }
}
