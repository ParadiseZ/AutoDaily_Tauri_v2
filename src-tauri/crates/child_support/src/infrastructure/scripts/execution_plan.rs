use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::infrastructure::ipc::message::RunTarget;

pub struct ExecutionPlanAssembler;

impl ExecutionPlanAssembler {
    pub fn select_tasks(
        run_target: &RunTarget,
        tasks: &[ScriptTaskTable],
    ) -> Result<Vec<ScriptTaskTable>, String> {
        let runnable_tasks: Vec<ScriptTaskTable> = tasks
            .iter()
            .filter(|task| matches!(task.row_type, TaskRowType::Task))
            .filter(|task| !task.is_deleted)
            .filter(|task| task.default_enabled)
            .map(|task| ScriptTaskTable {
                id: task.id,
                script_id: task.script_id,
                name: task.name.clone(),
                row_type: task.row_type.clone(),
                trigger_mode: task.trigger_mode.clone(),
                record_schedule: task.record_schedule,
                section_id: task.section_id,
                indent_level: task.indent_level,
                default_task_cycle: task.default_task_cycle.clone(),
                show_enabled_toggle: task.show_enabled_toggle,
                default_enabled: task.default_enabled,
                task_tone: task.task_tone.clone(),
                is_hidden: task.is_hidden,
                data: task.data.clone(),
                created_at: task.created_at,
                updated_at: task.updated_at,
                deleted_at: task.deleted_at,
                is_deleted: task.is_deleted,
                index: task.index,
            })
            .collect();

        match run_target {
            RunTarget::DeviceQueue | RunTarget::FullScript { .. } => Ok(runnable_tasks),
            RunTarget::Task { task_id, .. } => runnable_tasks
                .into_iter()
                .find(|task| task.id == *task_id)
                .map(|task| vec![task])
                .ok_or_else(|| format!("运行目标中的任务[{}]不存在或不可执行", task_id)),
            RunTarget::PolicyGroup {
                policy_group_id, ..
            } => Err(format!(
                "策略组目标[{}]的执行计划尚未接入，当前不执行降级推断",
                policy_group_id
            )),
            RunTarget::PolicySet { policy_set_id, .. } => Err(format!(
                "策略集目标[{}]的执行计划尚未接入，当前不执行降级推断",
                policy_set_id
            )),
        }
    }
}
