use crate::domain::devices::device_schedule::TaskCycle;
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::infrastructure::core::{DeviceId, ScheduleId};
use crate::infrastructure::ipc::message::{RunTarget, RuntimeQueueItem};
use crate::infrastructure::scripts::schedule_journal::ScheduleJournal;
use chrono::{DateTime, Datelike, Duration, Local};
use serde::Deserialize;
use std::collections::HashMap;

pub struct ExecutionPlanAssembler;

#[derive(Debug, Clone)]
pub struct PlannedTask {
    pub task: ScriptTaskTable,
    pub task_cycle: TaskCycle,
}

#[derive(Debug, Clone)]
pub struct SkippedTask {
    pub task: ScriptTaskTable,
    pub task_cycle: TaskCycle,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct TaskSelection {
    pub runnable_tasks: Vec<PlannedTask>,
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
    pub async fn select_tasks(
        run_target: &RunTarget,
        device_id: DeviceId,
        queue_item: &RuntimeQueueItem,
        tasks: &[ScriptTaskTable],
    ) -> Result<TaskSelection, String> {
        let template_values = Self::parse_template_values(queue_item.template_values_json.as_deref())?;
        let mut runnable_tasks = Vec::new();
        let mut skipped_tasks = Vec::new();

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

            runnable_tasks.push(PlannedTask { task, task_cycle });
        }

        match run_target {
            RunTarget::DeviceQueue | RunTarget::FullScript { .. } => Ok(TaskSelection {
                runnable_tasks,
                skipped_tasks,
            }),
            RunTarget::Task { task_id, .. } => runnable_tasks
                .into_iter()
                .find(|planned_task| planned_task.task.id == *task_id)
                .map(|planned_task| TaskSelection {
                    runnable_tasks: vec![planned_task],
                    skipped_tasks: Vec::new(),
                })
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
                    Some(format!("任务仅在周{}执行，今日不在执行日", Self::weekday_label(*day)))
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
                    Some(format!("任务仅在周{}执行，今日不在执行日", Self::weekday_label(*day)))
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
