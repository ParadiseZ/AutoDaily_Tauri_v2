use crate::domain::devices::device_schedule::TaskCycle;
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType, TaskTriggerMode};
use crate::infrastructure::core::{DeviceId, ScheduleId, TaskId};
use crate::infrastructure::ipc::message::{RunTarget, RuntimeQueueItem};
use crate::infrastructure::scripts::schedule_journal::ScheduleJournal;
use chrono::{DateTime, Datelike, Duration, Local};
use serde::Deserialize;
use std::collections::HashMap;

pub struct ExecutionPlanAssembler;

#[derive(Debug, Clone)]
pub enum ExecutionPlan {
    Task(TaskSelection),
    PolicyDebug,
}

#[derive(Debug, Clone)]
pub struct PlannedTask {
    pub task: ScriptTaskTable,
    pub task_cycle: TaskCycle,
    pub allow_root: bool,
    pub allow_link: bool,
}

#[derive(Debug, Clone)]
pub struct SkippedTask {
    pub task: ScriptTaskTable,
    pub task_cycle: TaskCycle,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct TaskSelection {
    pub root_tasks: Vec<PlannedTask>,
    pub linkable_tasks: HashMap<TaskId, PlannedTask>,
    pub skipped_tasks: Vec<SkippedTask>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TemplateValuesSnapshot {
    #[serde(default)]
    task_settings: HashMap<String, TemplateTaskSetting>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TemplateTaskSetting {
    enabled: Option<bool>,
    task_cycle: Option<TaskCycle>,
}

impl ExecutionPlanAssembler {
    pub async fn assemble(
        run_target: &RunTarget,
        device_id: DeviceId,
        queue_item: &RuntimeQueueItem,
        tasks: &[ScriptTaskTable],
    ) -> Result<ExecutionPlan, String> {
        if Self::is_policy_debug_target(run_target) {
            return Ok(ExecutionPlan::PolicyDebug);
        }

        let selection = Self::select_tasks(run_target, device_id, queue_item, tasks).await?;
        Ok(ExecutionPlan::Task(selection))
    }

    pub fn is_policy_debug_target(run_target: &RunTarget) -> bool {
        matches!(
            run_target,
            RunTarget::Policy { .. } | RunTarget::PolicyGroup { .. } | RunTarget::PolicySet { .. }
        )
    }

    pub fn should_record_schedule(run_target: &RunTarget, task: &ScriptTaskTable) -> bool {
        matches!(run_target, RunTarget::DeviceQueue)
            && task.record_schedule
            && matches!(task.row_type, TaskRowType::Task)
    }

    async fn select_tasks(
        run_target: &RunTarget,
        device_id: DeviceId,
        queue_item: &RuntimeQueueItem,
        tasks: &[ScriptTaskTable],
    ) -> Result<TaskSelection, String> {
        let template_values =
            Self::parse_template_values(queue_item.template_values_json.as_deref())?;
        let mut root_tasks = Vec::new();
        let mut linkable_tasks = HashMap::new();
        let mut skipped_tasks = Vec::new();
        let mut direct_target = None;
        let direct_task_id = match run_target {
            RunTarget::Task { task_id, .. } => Some(*task_id),
            _ => None,
        };

        for task in tasks
            .iter()
            .filter(|task| matches!(task.row_type, TaskRowType::Task))
            .filter(|task| !task.is_deleted)
        {
            let task = task.clone();
            if !Self::resolve_task_enabled(&task, template_values.as_ref()) {
                continue;
            }
            let task_cycle = Self::resolve_task_cycle(&task, template_values.as_ref());
            let (allow_root, allow_link) = Self::resolve_trigger_mode(&task);
            let planned_task = PlannedTask {
                task: task.clone(),
                task_cycle: task_cycle.clone(),
                allow_root,
                allow_link,
            };

            if allow_link {
                linkable_tasks.insert(task.id, planned_task.clone());
            }

            if Some(task.id) == direct_task_id {
                direct_target = Some(planned_task);
                continue;
            }

            if !allow_root {
                continue;
            }

            if let Some(reason) = Self::should_skip_by_schedule(
                run_target,
                device_id,
                queue_item.assignment_id,
                &task,
                &task_cycle,
            )
            .await?
            {
                skipped_tasks.push(SkippedTask {
                    task,
                    task_cycle,
                    reason,
                });
                continue;
            }

            root_tasks.push(planned_task);
        }

        match run_target {
            RunTarget::DeviceQueue | RunTarget::FullScript { .. } => Ok(TaskSelection {
                root_tasks,
                linkable_tasks,
                skipped_tasks,
            }),
            RunTarget::Task { task_id, .. } => direct_target
                .map(|planned_task| TaskSelection {
                    root_tasks: vec![planned_task],
                    linkable_tasks,
                    skipped_tasks: Vec::new(),
                })
                .ok_or_else(|| format!("运行目标中的任务[{}]不存在或不可执行", task_id)),
            RunTarget::PolicyGroup { .. }
            | RunTarget::PolicySet { .. }
            | RunTarget::Policy { .. } => Err("策略调试目标不进入 task 执行计划".to_string()),
        }
    }

    fn resolve_trigger_mode(task: &ScriptTaskTable) -> (bool, bool) {
        match task.trigger_mode {
            TaskTriggerMode::RootOnly => (true, false),
            TaskTriggerMode::LinkOnly => (false, true),
            TaskTriggerMode::RootAndLink => (true, true),
        }
    }

    fn parse_template_values(json: Option<&str>) -> Result<Option<TemplateValuesSnapshot>, String> {
        match json {
            Some(content) if !content.trim().is_empty() && content.trim() != "null" => {
                serde_json::from_str(content)
                    .map(Some)
                    .map_err(|error| format!("解析模板覆盖值失败: {}", error))
            }
            _ => Ok(None),
        }
    }

    fn resolve_task_enabled(
        task: &ScriptTaskTable,
        template_values: Option<&TemplateValuesSnapshot>,
    ) -> bool {
        template_values
            .and_then(|snapshot| snapshot.task_settings.get(&task.id.to_string()))
            .and_then(|setting| setting.enabled)
            .unwrap_or(task.default_enabled)
    }

    fn resolve_task_cycle(
        task: &ScriptTaskTable,
        template_values: Option<&TemplateValuesSnapshot>,
    ) -> TaskCycle {
        template_values
            .and_then(|snapshot| snapshot.task_settings.get(&task.id.to_string()))
            .and_then(|setting| setting.task_cycle.clone())
            .unwrap_or_else(|| task.default_task_cycle.0.clone())
    }

    async fn should_skip_by_schedule(
        run_target: &RunTarget,
        device_id: DeviceId,
        assignment_id: ScheduleId,
        task: &ScriptTaskTable,
        task_cycle: &TaskCycle,
    ) -> Result<Option<String>, String> {
        if !matches!(run_target, RunTarget::DeviceQueue) || !task.record_schedule {
            return Ok(None);
        }

        if matches!(task_cycle, TaskCycle::EveryRun) {
            return Ok(None);
        }

        let Some(record) =
            ScheduleJournal::load_latest_success_record(device_id, assignment_id, task.id).await?
        else {
            return Ok(Self::skip_reason_without_history(task_cycle));
        };

        let reference_at = record.completed_at.as_deref().unwrap_or(&record.started_at);
        let last_success = DateTime::parse_from_rfc3339(reference_at)
            .map_err(|error| format!("解析最近一次调度记录时间失败: {}", error))?
            .with_timezone(&Local);

        Ok(Self::skip_reason_with_history(task_cycle, last_success))
    }

    fn skip_reason_without_history(task_cycle: &TaskCycle) -> Option<String> {
        let now = Local::now();
        match task_cycle {
            TaskCycle::EveryRun => None,
            TaskCycle::WeekDay(day) => {
                let today = now.weekday().number_from_monday() as u8;
                if today == *day {
                    None
                } else {
                    Some(format!(
                        "任务仅在周{}执行，今日不在执行日",
                        Self::weekday_label(*day)
                    ))
                }
            }
            TaskCycle::MonthDay(day) => {
                if now.day() as u8 == *day {
                    None
                } else {
                    Some(format!("任务仅在每月 {} 日执行，今日不在执行日", day))
                }
            }
            _ => None,
        }
    }

    fn skip_reason_with_history(
        task_cycle: &TaskCycle,
        last_success: DateTime<Local>,
    ) -> Option<String> {
        let now = Local::now();
        match task_cycle {
            TaskCycle::EveryRun => None,
            TaskCycle::Daily => {
                if now.date_naive() == last_success.date_naive() {
                    Some("任务今日已执行成功，按 Daily 周期跳过".to_string())
                } else {
                    None
                }
            }
            TaskCycle::Weekly => {
                if now < last_success + Duration::days(7) {
                    Some("任务距离上次成功未满 7 天，按 Weekly 周期跳过".to_string())
                } else {
                    None
                }
            }
            TaskCycle::WeekDay(day) => {
                let today = now.weekday().number_from_monday() as u8;
                if today != *day {
                    Some(format!(
                        "任务仅在周{}执行，今日不在执行日",
                        Self::weekday_label(*day)
                    ))
                } else if now.date_naive() == last_success.date_naive() {
                    Some("任务今日已执行成功，按 WeekDay 周期跳过".to_string())
                } else {
                    None
                }
            }
            TaskCycle::Monthly => {
                if now.year() == last_success.year() && now.month() == last_success.month() {
                    Some("任务本月已执行成功，按 Monthly 周期跳过".to_string())
                } else {
                    None
                }
            }
            TaskCycle::MonthDay(day) => {
                if now.day() as u8 != *day {
                    Some(format!("任务仅在每月 {} 日执行，今日不在执行日", day))
                } else if now.date_naive() == last_success.date_naive() {
                    Some("任务今日已执行成功，按 MonthDay 周期跳过".to_string())
                } else {
                    None
                }
            }
        }
    }

    fn weekday_label(day: u8) -> &'static str {
        match day {
            1 => "一",
            2 => "二",
            3 => "三",
            4 => "四",
            5 => "五",
            6 => "六",
            _ => "日",
        }
    }
}

impl ExecutionPlan {
    pub fn task_selection(self) -> TaskSelection {
        match self {
            ExecutionPlan::Task(selection) => selection,
            ExecutionPlan::PolicyDebug => TaskSelection::default(),
        }
    }

    pub fn is_policy_debug(&self) -> bool {
        matches!(self, ExecutionPlan::PolicyDebug)
    }
}
