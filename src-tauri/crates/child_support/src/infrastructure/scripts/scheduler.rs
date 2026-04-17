// 脚本调度器
// 管理子进程中的脚本执行队列，按顺序执行脚本

use crate::domain::devices::device_schedule::RunStatus;
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_info::{ScriptInfo, ScriptTable};
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::infrastructure::context::runtime_context::get_runtime_ctx;
use crate::infrastructure::core::ExecutionId;
use crate::infrastructure::core::ScriptId;
use crate::infrastructure::ipc::message::{
    ResumeCheckpoint, ResumeMode, RunTarget, RuntimeProgressPhase, RuntimeQueueItem,
    RuntimeScheduleStatus, RuntimeSessionSnapshot,
};
use crate::infrastructure::ipc::runtime_reporter::{emit_progress_event, emit_schedule_event};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::execution_plan::{ExecutionPlanAssembler, PlannedTask};
use crate::infrastructure::scripts::executor::ScriptExecutor;
use crate::infrastructure::scripts::schedule_journal::ScheduleJournal;
use crate::infrastructure::session::recovery_checkpoint_store::{
    build_definition_fingerprint, clear_checkpoint_by_device,
};
use crate::infrastructure::session::runtime_session::{
    get_loaded_checkpoint, get_script_bundle_snapshot, take_loaded_checkpoint,
    try_current_session_summary,
};
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::ocr_service::OcrService;
use crate::infrastructure::vision::rec::RecognizerType;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

struct ParsedScriptBundle {
    script: ScriptTable,
    tasks: Vec<ScriptTaskTable>,
    policies: Vec<PolicyTable>,
    policy_groups: Vec<PolicyGroupTable>,
    policy_sets: Vec<PolicySetTable>,
    group_policies: Vec<GroupPolicyRelation>,
    set_groups: Vec<SetGroupRelation>,
}

#[derive(Debug, Clone)]
struct TaskResumeCursor {
    start_index: usize,
    message: String,
}

/// 脚本调度器
pub struct ScriptScheduler {
    /// 待执行的脚本队列
    queue: Arc<RwLock<VecDeque<RuntimeQueueItem>>>,
    /// 当前正在执行的脚本
    current_script: Arc<RwLock<Option<ScriptId>>>,
    /// 取消令牌
    cancel_token: CancellationToken,
}

/// 全局调度器
static SCHEDULER: std::sync::OnceLock<Arc<ScriptScheduler>> = std::sync::OnceLock::new();

pub fn init_scheduler(cancel_token: CancellationToken) -> Arc<ScriptScheduler> {
    let scheduler = Arc::new(ScriptScheduler {
        queue: Arc::new(RwLock::new(VecDeque::new())),
        current_script: Arc::new(RwLock::new(None)),
        cancel_token,
    });
    let _ = SCHEDULER.set(scheduler.clone());
    scheduler
}

pub fn get_scheduler() -> Option<Arc<ScriptScheduler>> {
    SCHEDULER.get().cloned()
}

impl ScriptScheduler {
    fn clone_model_config<T>(field: &str, value: &T) -> Result<T, String>
    where
        T: Serialize + DeserializeOwned,
    {
        let json = serde_json::to_string(value)
            .map_err(|error| format!("序列化 {} 配置失败: {}", field, error))?;
        serde_json::from_str(&json).map_err(|error| format!("复制 {} 配置失败: {}", field, error))
    }

    async fn configure_visual_services(
        runtime_ctx: &Arc<RwLock<crate::infrastructure::context::runtime_context::RuntimeContext>>,
        script_info: &ScriptInfo,
    ) -> Result<(), String> {
        let (img_det_service, ocr_service) = {
            let ctx = runtime_ctx.read().await;
            (ctx.img_det_service.clone(), ctx.ocr_service.clone())
        };

        {
            let mut service = img_det_service.lock().await;
            *service = OcrService::new();
            if let Some(model) = script_info.img_det_model.as_ref() {
                let detector = Self::clone_model_config::<DetectorType>("img_det_model", model)?;
                service
                    .init_detector(detector)
                    .await
                    .map_err(|error| format!("初始化目标检测模型失败: {}", error))?;
            }
        }

        {
            let mut service = ocr_service.lock().await;
            *service = OcrService::new();
            if let Some(model) = script_info.txt_det_model.as_ref() {
                let detector = Self::clone_model_config::<DetectorType>("txt_det_model", model)?;
                service
                    .init_detector(detector)
                    .await
                    .map_err(|error| format!("初始化文字检测模型失败: {}", error))?;
            }
            if let Some(model) = script_info.txt_rec_model.as_ref() {
                let recognizer =
                    Self::clone_model_config::<RecognizerType>("txt_rec_model", model)?;
                service
                    .init_recognizer(recognizer)
                    .await
                    .map_err(|error| format!("初始化文字识别模型失败: {}", error))?;
            }
        }

        Ok(())
    }

    fn parse_bundle_json<T: DeserializeOwned>(field: &str, json: &str) -> Result<T, String> {
        serde_json::from_str(json)
            .map_err(|error| format!("解析 bundle 字段 {} 失败: {}", field, error))
    }

    async fn load_script_bundle(script_id: ScriptId) -> Result<ParsedScriptBundle, String> {
        let snapshot = get_script_bundle_snapshot(script_id)
            .await
            .ok_or_else(|| format!("session 中不存在脚本[{}]的 bundle", script_id))?;

        Ok(ParsedScriptBundle {
            script: Self::parse_bundle_json("script_json", &snapshot.script_json)?,
            tasks: Self::parse_bundle_json("tasks_json", &snapshot.tasks_json)?,
            policies: Self::parse_bundle_json("policies_json", &snapshot.policies_json)?,
            policy_groups: Self::parse_bundle_json(
                "policy_groups_json",
                &snapshot.policy_groups_json,
            )?,
            policy_sets: Self::parse_bundle_json("policy_sets_json", &snapshot.policy_sets_json)?,
            group_policies: Self::parse_bundle_json(
                "group_policies_json",
                &snapshot.group_policies_json,
            )?,
            set_groups: Self::parse_bundle_json("set_groups_json", &snapshot.set_groups_json)?,
        })
    }

    fn current_run_target() -> RunTarget {
        try_current_session_summary()
            .map(|summary| summary.run_target)
            .unwrap_or(RunTarget::DeviceQueue)
    }

    async fn consume_task_skip_flag(
        runtime_ctx: &Arc<RwLock<crate::infrastructure::context::runtime_context::RuntimeContext>>,
        task_id: crate::infrastructure::core::TaskId,
    ) -> bool {
        let mut ctx = runtime_ctx.write().await;
        let Some(state) = ctx.execution.task_states.get_mut(&task_id) else {
            return false;
        };
        let skip_flag = state.skip_flag;
        state.skip_flag = false;
        skip_flag
    }

    async fn mark_task_succeeded(
        runtime_ctx: &Arc<RwLock<crate::infrastructure::context::runtime_context::RuntimeContext>>,
        task_id: crate::infrastructure::core::TaskId,
    ) {
        let mut ctx = runtime_ctx.write().await;
        let state = ctx.execution.task_states.entry(task_id).or_default();
        state.done_flag = true;
        state.exec_cur = state.exec_cur.saturating_add(1);
    }

    fn should_record_schedule(run_target: &RunTarget, task: &ScriptTaskTable) -> bool {
        matches!(run_target, RunTarget::DeviceQueue)
            && task.record_schedule
            && matches!(task.row_type, TaskRowType::Task)
    }

    async fn take_resume_checkpoint_for_execution(
        run_target: &RunTarget,
        queue_item: &RuntimeQueueItem,
    ) -> Option<ResumeCheckpoint> {
        let checkpoint = get_loaded_checkpoint().await?;
        let should_take = match run_target {
            RunTarget::DeviceQueue => checkpoint.assignment_id == Some(queue_item.assignment_id),
            RunTarget::FullScript { .. } | RunTarget::Task { .. } => {
                checkpoint.script_id == queue_item.script_id
            }
            _ => false,
        };

        if should_take {
            take_loaded_checkpoint().await
        } else {
            None
        }
    }

    async fn validate_resume_checkpoint(
        run_target: &RunTarget,
        queue_item: &RuntimeQueueItem,
        checkpoint: &ResumeCheckpoint,
    ) -> Result<(), String> {
        let expected_fingerprint = build_definition_fingerprint(checkpoint.script_id).await?;
        if checkpoint.definition_fingerprint != expected_fingerprint {
            return Err("checkpoint 定义指纹已失效，已降级为普通执行".to_string());
        }

        match run_target {
            RunTarget::DeviceQueue => {
                if checkpoint.assignment_id != Some(queue_item.assignment_id) {
                    return Err(format!(
                        "checkpoint assignment[{:?}] 与当前队列项[{}]不一致，已降级为普通执行",
                        checkpoint.assignment_id, queue_item.assignment_id
                    ));
                }
            }
            RunTarget::Task { task_id, .. } => {
                if checkpoint.task_id != Some(*task_id) {
                    return Err(format!(
                        "checkpoint task[{:?}] 与当前调试任务[{}]不一致，已降级为普通执行",
                        checkpoint.task_id, task_id
                    ));
                }
            }
            RunTarget::FullScript { .. } => {}
            _ => {
                return Err("当前运行目标不支持 checkpoint 恢复执行".to_string());
            }
        }

        Ok(())
    }

    fn apply_resume_checkpoint_to_pending_tasks(
        pending_tasks: &mut VecDeque<PlannedTask>,
        linkable_tasks: &std::collections::HashMap<
            crate::infrastructure::core::TaskId,
            PlannedTask,
        >,
        checkpoint: &ResumeCheckpoint,
    ) -> Result<String, String> {
        let Some(task_id) = checkpoint.task_id else {
            return Ok("checkpoint 未记录 task，已降级为从脚本起点恢复".to_string());
        };

        if let Some(position) = pending_tasks
            .iter()
            .position(|planned| planned.task.id == task_id)
        {
            for _ in 0..position {
                pending_tasks.pop_front();
            }
            return Ok(format!("将从 checkpoint 任务[{}]继续恢复", task_id));
        }

        let planned_task = linkable_tasks.get(&task_id).cloned().ok_or_else(|| {
            format!(
                "checkpoint 任务[{}]不在当前执行计划里，已降级为普通执行",
                task_id
            )
        })?;
        let target_index = planned_task.task.index;
        let trimmed: VecDeque<_> = pending_tasks
            .drain(..)
            .filter(|planned| planned.task.id != task_id && planned.task.index >= target_index)
            .collect();
        *pending_tasks = trimmed;
        pending_tasks.push_front(planned_task);

        Ok(format!(
            "checkpoint 命中的任务[{}]不是 root task，已按任务安全点插回队首恢复",
            task_id
        ))
    }

    fn resolve_task_resume_cursor(
        task: &ScriptTaskTable,
        checkpoint: &ResumeCheckpoint,
    ) -> TaskResumeCursor {
        match checkpoint.resume_mode {
            ResumeMode::FromTaskStart => TaskResumeCursor {
                start_index: 0,
                message: format!("任务[{}]按 checkpoint 从任务起点恢复", task.name),
            },
            ResumeMode::FromStepStart => match checkpoint.step_id {
                Some(step_id) => match task
                    .data
                    .0
                    .steps
                    .iter()
                    .position(|step| step.id == Some(step_id))
                {
                    Some(index) => TaskResumeCursor {
                        start_index: index,
                        message: format!(
                            "任务[{}]按 checkpoint 从步骤[{}]起点恢复",
                            task.name, step_id
                        ),
                    },
                    None => TaskResumeCursor {
                        start_index: 0,
                        message: format!(
                            "checkpoint 步骤[{}]不在任务[{}]顶层步骤中，已降级为从任务起点恢复",
                            step_id, task.name
                        ),
                    },
                },
                None => TaskResumeCursor {
                    start_index: 0,
                    message: format!(
                        "checkpoint 未记录步骤游标，任务[{}]已降级为从任务起点恢复",
                        task.name
                    ),
                },
            },
            ResumeMode::FromNextStep => match checkpoint.step_id {
                Some(step_id) => match task
                    .data
                    .0
                    .steps
                    .iter()
                    .position(|step| step.id == Some(step_id))
                {
                    Some(index) => TaskResumeCursor {
                        start_index: (index + 1).min(task.data.0.steps.len()),
                        message: format!(
                            "任务[{}]按 checkpoint 从步骤[{}]之后继续恢复",
                            task.name, step_id
                        ),
                    },
                    None => TaskResumeCursor {
                        start_index: 0,
                        message: format!(
                            "checkpoint 步骤[{}]不在任务[{}]顶层步骤中，已降级为从任务起点恢复",
                            step_id, task.name
                        ),
                    },
                },
                None => TaskResumeCursor {
                    start_index: 0,
                    message: format!(
                        "checkpoint 未记录步骤游标，任务[{}]已降级为从任务起点恢复",
                        task.name
                    ),
                },
            },
        }
    }

    /// 用完整 session 替换当前队列
    pub async fn load_session(&self, session: RuntimeSessionSnapshot) {
        let mut queue = self.queue.write().await;
        queue.clear();
        queue.extend(session.queue);
        *self.current_script.write().await = None;
        Log::info(&format!(
            "[ scheduler ] 已加载 session[{}]，队列长度: {}",
            session.session_id,
            queue.len()
        ));
    }

    /// 获取当前正在执行的脚本
    pub async fn current_script(&self) -> Option<ScriptId> {
        *self.current_script.read().await
    }

    /// 非阻塞快照读取，供事件上报使用
    pub fn current_script_snapshot(&self) -> Option<ScriptId> {
        self.current_script.try_read().ok().and_then(|guard| *guard)
    }

    /// 获取队列长度
    pub async fn queue_len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// 调度循环 — 在 Running 状态下被 main_child 调用
    /// 从队列取出脚本执行，执行完后取下一个
    /// 返回 true 表示还有任务可执行，false 表示队列为空
    pub async fn tick(&self) -> bool {
        // 检查取消
        if self.cancel_token.is_cancelled() {
            return false;
        }

        // 从队列取出下一个脚本
        let queue_item = {
            let mut queue = self.queue.write().await;
            queue.pop_front()
        };

        let queue_item = match queue_item {
            Some(item) => item,
            None => return false, // 队列为空
        };
        let script_id = queue_item.script_id;
        let assignment_id = queue_item.assignment_id;
        let execution_id = ExecutionId::new_v7();

        // 标记当前脚本
        *self.current_script.write().await = Some(script_id);
        Log::info(&format!("[ scheduler ] 开始执行脚本: {}", script_id));
        emit_progress_event(
            RuntimeProgressPhase::Loading,
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some("开始加载脚本 bundle".to_string()),
        );
        emit_schedule_event(
            RuntimeScheduleStatus::Running,
            Some(execution_id),
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some("脚本已进入执行".to_string()),
        );

        // 执行脚本
        let result = self.execute_script(queue_item, execution_id).await;

        // 清除当前脚本
        *self.current_script.write().await = None;

        match result {
            Ok(()) => {
                Log::info(&format!("[ scheduler ] 脚本[{}]执行完成", script_id));
            }
            Err(e) => {
                Log::error(&format!("[ scheduler ] 脚本[{}]执行失败: {}", script_id, e));
                emit_progress_event(
                    RuntimeProgressPhase::Failed,
                    Some(assignment_id),
                    Some(script_id),
                    None,
                    None,
                    Some(e.clone()),
                );
                emit_schedule_event(
                    RuntimeScheduleStatus::Failed,
                    Some(execution_id),
                    Some(assignment_id),
                    Some(script_id),
                    None,
                    None,
                    Some(e),
                );
            }
        }

        // 还有更多脚本？
        self.queue.read().await.len() > 0
    }

    /// 执行单个脚本
    async fn execute_script(
        &self,
        queue_item: RuntimeQueueItem,
        execution_id: ExecutionId,
    ) -> Result<(), String> {
        let script_id = queue_item.script_id;
        let assignment_id = queue_item.assignment_id;
        let device_id = try_current_session_summary()
            .map(|summary| summary.device_id)
            .ok_or_else(|| "当前 child session 未加载 device_id".to_string())?;
        let bundle = Self::load_script_bundle(script_id).await?;
        let tasks_len = bundle.tasks.len();
        let policy_count = bundle.policies.len();
        let policy_group_count = bundle.policy_groups.len();
        let policy_set_count = bundle.policy_sets.len();
        let group_policy_count = bundle.group_policies.len();
        let set_group_count = bundle.set_groups.len();
        let runtime_ctx = get_runtime_ctx();
        let script_info = Self::clone_model_config("script_info", &bundle.script.data.0)?;
        let script_name = script_info.name.clone();
        Self::configure_visual_services(&runtime_ctx, &script_info).await?;
        let run_target = Self::current_run_target();
        let is_policy_debug_target = matches!(
            run_target,
            RunTarget::Policy { .. } | RunTarget::PolicyGroup { .. } | RunTarget::PolicySet { .. }
        );
        let task_selection = if is_policy_debug_target {
            crate::infrastructure::scripts::execution_plan::TaskSelection::default()
        } else {
            ExecutionPlanAssembler::select_tasks(&run_target, device_id, &queue_item, &bundle.tasks)
                .await?
        };
        let runnable_task_count = task_selection.root_tasks.len();
        let skipped_task_count = task_selection.skipped_tasks.len();
        let linkable_task_count = task_selection.linkable_tasks.len();
        let mut resume_checkpoint = if is_policy_debug_target {
            None
        } else {
            Self::take_resume_checkpoint_for_execution(&run_target, &queue_item).await
        };
        let mut clear_checkpoint_on_success = false;

        // 更新运行时上下文的 script_id
        {
            let mut ctx = runtime_ctx.write().await;
            ctx.execution.current_execution_id = Some(execution_id);
            ctx.execution.current_assignment_id = Some(assignment_id);
            ctx.execution.script_id = script_id;
            ctx.execution.target = run_target.clone();
            ctx.execution.script_info = Some(script_info);
            ctx.execution.current_task = None;
            ctx.execution.current_step_id = None;
            ctx.execution.var_map.clear();
            ctx.execution.policy_states.clear();
            ctx.execution.task_states.clear();
            ctx.execution.action_states.clear();
            ctx.execution.policy_set_overlays.clear();
            ctx.observation.last_capture_image = None;
            ctx.observation.last_snapshot = None;
            ctx.observation.last_hits.clear();
            if let Err(error) = ctx
                .observation
                .vision_text_cache
                .load_for_script(script_id, &script_name)
            {
                Log::warn(&format!(
                    "[ scheduler ] 脚本[{}]加载 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        emit_progress_event(
            RuntimeProgressPhase::Planning,
            Some(queue_item.assignment_id),
            Some(script_id),
            None,
            None,
            Some(format!(
                "已加载 bundle: tasks={}, policies={}, groups={}, sets={}",
                tasks_len, policy_count, policy_group_count, policy_set_count
            )),
        );
        Log::info(&format!(
            "[ scheduler ] 脚本[{}] bundle 已加载，task={}, root_task={}, linkable_task={}, skipped_task={}, policy={}, group_relation={}, set_relation={}",
            script_id, tasks_len, runnable_task_count, linkable_task_count, skipped_task_count, policy_count, group_policy_count, set_group_count
        ));
        emit_progress_event(
            RuntimeProgressPhase::Executing,
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some(format!(
                "执行计划已装配，一级任务 {} 个，可跳转任务 {} 个，跳过 {} 个",
                runnable_task_count, linkable_task_count, skipped_task_count
            )),
        );

        if let Some(checkpoint) = resume_checkpoint.as_ref() {
            match Self::validate_resume_checkpoint(&run_target, &queue_item, checkpoint).await {
                Ok(()) => {
                    clear_checkpoint_on_success = true;
                    emit_progress_event(
                        RuntimeProgressPhase::Executing,
                        Some(assignment_id),
                        Some(script_id),
                        checkpoint.task_id,
                        checkpoint.step_id,
                        Some(format!(
                            "已加载 checkpoint，准备按 {:?} 恢复执行",
                            checkpoint.resume_mode
                        )),
                    );
                }
                Err(reason) => {
                    Log::warn(&format!(
                        "[ scheduler ] 脚本[{}] checkpoint 不可恢复，已降级: {}",
                        script_id, reason
                    ));
                    emit_progress_event(
                        RuntimeProgressPhase::Executing,
                        Some(assignment_id),
                        Some(script_id),
                        checkpoint.task_id,
                        checkpoint.step_id,
                        Some(reason),
                    );
                    if let Err(error) = clear_checkpoint_by_device(device_id).await {
                        Log::warn(&format!(
                            "[ scheduler ] 清理失效 checkpoint 失败，已忽略: {}",
                            error
                        ));
                    }
                    resume_checkpoint = None;
                }
            }
        }

        if is_policy_debug_target {
            return Self::execute_debug_policy_target(
                &run_target,
                assignment_id,
                script_id,
                &bundle,
                &runtime_ctx,
            )
            .await;
        }

        for skipped in &task_selection.skipped_tasks {
            emit_schedule_event(
                RuntimeScheduleStatus::Skipped,
                Some(execution_id),
                Some(assignment_id),
                Some(script_id),
                Some(skipped.task.id),
                None,
                Some(skipped.reason.clone()),
            );

            if Self::should_record_schedule(&run_target, &skipped.task) {
                let now = chrono::Utc::now().to_rfc3339();
                ScheduleJournal::append_task_record(
                    device_id,
                    execution_id,
                    assignment_id,
                    script_id,
                    &skipped.task,
                    &skipped.task_cycle,
                    RunStatus::Skipped,
                    now.clone(),
                    Some(now),
                    Some(skipped.reason.clone()),
                )
                .await?;
            }
        }

        let root_tasks = task_selection.root_tasks.clone();
        let mut executor = ScriptExecutor::new(runtime_ctx.clone());
        let mut pending_tasks: VecDeque<_> = root_tasks.clone().into_iter().collect();
        let linkable_tasks = task_selection.linkable_tasks;
        if let Some(checkpoint) = resume_checkpoint.as_ref() {
            match Self::apply_resume_checkpoint_to_pending_tasks(
                &mut pending_tasks,
                &linkable_tasks,
                checkpoint,
            ) {
                Ok(message) => {
                    emit_progress_event(
                        RuntimeProgressPhase::Executing,
                        Some(assignment_id),
                        Some(script_id),
                        checkpoint.task_id,
                        checkpoint.step_id,
                        Some(message),
                    );
                }
                Err(reason) => {
                    Log::warn(&format!(
                        "[ scheduler ] 脚本[{}] checkpoint 任务恢复失败，已降级: {}",
                        script_id, reason
                    ));
                    emit_progress_event(
                        RuntimeProgressPhase::Executing,
                        Some(assignment_id),
                        Some(script_id),
                        checkpoint.task_id,
                        checkpoint.step_id,
                        Some(reason),
                    );
                    if let Err(error) = clear_checkpoint_by_device(device_id).await {
                        Log::warn(&format!(
                            "[ scheduler ] 清理不可恢复 checkpoint 失败，已忽略: {}",
                            error
                        ));
                    }
                    resume_checkpoint = None;
                    clear_checkpoint_on_success = false;
                    pending_tasks = root_tasks.clone().into_iter().collect();
                }
            }
        }
        while let Some(planned_task) = pending_tasks.pop_front() {
            let task_cycle = planned_task.task_cycle;
            let task = planned_task.task;
            let task_started_at = chrono::Utc::now().to_rfc3339();
            let task_resume_cursor = resume_checkpoint
                .as_ref()
                .filter(|checkpoint| checkpoint.task_id == Some(task.id))
                .map(|checkpoint| Self::resolve_task_resume_cursor(&task, checkpoint));
            {
                let mut ctx = runtime_ctx.write().await;
                ctx.execution.current_task = Some(task.clone());
                ctx.execution.current_step_id = None;
            }

            emit_progress_event(
                RuntimeProgressPhase::Executing,
                Some(assignment_id),
                Some(script_id),
                Some(task.id),
                None,
                Some(
                    task_resume_cursor
                        .as_ref()
                        .map(|cursor| cursor.message.clone())
                        .unwrap_or_else(|| format!("开始执行任务: {}", task.name)),
                ),
            );

            executor.reset_node_indices();
            let task_steps = match task_resume_cursor.as_ref() {
                Some(cursor) => &task.data.0.steps[cursor.start_index..],
                None => task.data.0.steps.as_slice(),
            };
            let task_result = executor.execute(task_steps).await;
            if task_resume_cursor.is_some() {
                resume_checkpoint = None;
            }

            let completion_at = chrono::Utc::now().to_rfc3339();
            match task_result {
                Ok(flow) => {
                    let task_skipped = Self::consume_task_skip_flag(&runtime_ctx, task.id).await;
                    if !task_skipped {
                        Self::mark_task_succeeded(&runtime_ctx, task.id).await;
                    }
                    let linked_task = match flow {
                        crate::infrastructure::scripts::executor::ControlFlow::Link(target) => {
                            Some(linkable_tasks.get(&target).cloned().ok_or_else(|| {
                                format!("跳转目标任务[{}]不存在，或不允许通过 link 进入", target)
                            })?)
                        }
                        _ => None,
                    };
                    let link_target = linked_task.as_ref().map(|planned| planned.task.id);
                    let schedule_status = if task_skipped {
                        RuntimeScheduleStatus::Skipped
                    } else {
                        RuntimeScheduleStatus::Success
                    };
                    let schedule_message = if task_skipped {
                        match link_target {
                            Some(target) => {
                                format!("任务已跳过并跳转到任务[{}]: {}", target, task.name)
                            }
                            None => format!("任务已跳过: {}", task.name),
                        }
                    } else {
                        match link_target {
                            Some(target) => {
                                format!("任务执行完成并跳转到任务[{}]: {}", target, task.name)
                            }
                            None => format!("任务执行完成: {}", task.name),
                        }
                    };
                    emit_schedule_event(
                        schedule_status,
                        Some(execution_id),
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(schedule_message.clone()),
                    );
                    emit_progress_event(
                        RuntimeProgressPhase::Completed,
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(if task_skipped {
                            match link_target {
                                Some(target) => {
                                    format!("任务已跳过，下一步跳转到任务[{}]", target)
                                }
                                None => format!("任务已跳过: {}", task.name),
                            }
                        } else {
                            match link_target {
                                Some(target) => {
                                    format!("任务执行完成，下一步跳转到任务[{}]", target)
                                }
                                None => format!("任务执行完成: {}", task.name),
                            }
                        }),
                    );

                    if Self::should_record_schedule(&run_target, &task) {
                        ScheduleJournal::append_task_record(
                            device_id,
                            execution_id,
                            assignment_id,
                            script_id,
                            &task,
                            &task_cycle,
                            if task_skipped {
                                RunStatus::Skipped
                            } else {
                                RunStatus::Success
                            },
                            task_started_at.clone(),
                            Some(completion_at.clone()),
                            task_skipped.then(|| "任务在执行过程中被标记为跳过".to_string()),
                        )
                        .await?;
                    }
                    {
                        let mut ctx = runtime_ctx.write().await;
                        ctx.execution.current_task = None;
                        ctx.execution.current_step_id = None;
                    }

                    if let Some(linked_task) = linked_task {
                        let target = linked_task.task.id;
                        if let Some(position) = pending_tasks
                            .iter()
                            .position(|planned| planned.task.id == target)
                        {
                            pending_tasks.remove(position);
                        }
                        pending_tasks.push_front(linked_task);
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    emit_schedule_event(
                        RuntimeScheduleStatus::Failed,
                        Some(execution_id),
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(message.clone()),
                    );
                    emit_progress_event(
                        RuntimeProgressPhase::Failed,
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(message.clone()),
                    );

                    if Self::should_record_schedule(&run_target, &task) {
                        ScheduleJournal::append_task_record(
                            device_id,
                            execution_id,
                            assignment_id,
                            script_id,
                            &task,
                            &task_cycle,
                            RunStatus::Failed,
                            task_started_at,
                            Some(completion_at),
                            Some(message.clone()),
                        )
                        .await?;
                    }

                    {
                        let mut ctx = runtime_ctx.write().await;
                        if let Err(error) = ctx.observation.vision_text_cache.flush_current_script()
                        {
                            Log::warn(&format!(
                                "[ scheduler ] 脚本[{}]失败后写回 OCR 文字缓存失败，已忽略: {}",
                                script_id, error
                            ));
                        }
                        ctx.execution.current_execution_id = None;
                        ctx.execution.current_assignment_id = None;
                        ctx.execution.current_task = None;
                        ctx.execution.current_step_id = None;
                    }
                    return Err(message);
                }
            }
        }

        {
            let mut ctx = runtime_ctx.write().await;
            ctx.execution.current_execution_id = None;
            ctx.execution.current_assignment_id = None;
            ctx.execution.current_task = None;
            ctx.execution.current_step_id = None;
            if let Err(error) = ctx.observation.vision_text_cache.flush_current_script() {
                Log::warn(&format!(
                    "[ scheduler ] 脚本[{}]写回 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        emit_progress_event(
            RuntimeProgressPhase::Completed,
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some(format!("脚本执行完成，共 {} 个任务", runnable_task_count)),
        );
        emit_schedule_event(
            RuntimeScheduleStatus::Success,
            Some(execution_id),
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some(format!(
                "脚本执行完成，成功执行 {} 个任务，跳过 {} 个任务",
                runnable_task_count, skipped_task_count
            )),
        );

        if clear_checkpoint_on_success {
            if let Err(error) = clear_checkpoint_by_device(device_id).await {
                Log::warn(&format!(
                    "[ scheduler ] 脚本[{}]恢复执行成功后清理 checkpoint 失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        Ok(())
    }

    async fn execute_debug_policy_target(
        run_target: &RunTarget,
        assignment_id: crate::infrastructure::core::ScheduleId,
        script_id: ScriptId,
        bundle: &ParsedScriptBundle,
        runtime_ctx: &Arc<RwLock<crate::infrastructure::context::runtime_context::RuntimeContext>>,
    ) -> Result<(), String> {
        emit_progress_event(
            RuntimeProgressPhase::Executing,
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some("开始执行策略调试目标".to_string()),
        );

        let mut executor = ScriptExecutor::new(runtime_ctx.clone());
        let result = match run_target {
            RunTarget::Policy { policy_id, .. } => {
                let policy = bundle
                    .policies
                    .iter()
                    .find(|policy| policy.id == *policy_id)
                    .ok_or_else(|| format!("策略[{}]不存在", policy_id))?;
                Log::info(&format!(
                    "[ scheduler ] 开始调试策略: {} ({})",
                    policy.data.0.name, policy.id
                ));
                match executor
                    .debug_execute_policy(*policy_id)
                    .await
                    .map_err(|error| error.to_string())
                {
                    Ok(result) => {
                        emit_progress_event(
                            RuntimeProgressPhase::Completed,
                            Some(assignment_id),
                            Some(script_id),
                            None,
                            None,
                            Some(format!(
                                "策略调试完成: {} matched={}",
                                policy.data.0.name, result.matched
                            )),
                        );
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            RunTarget::PolicyGroup {
                policy_group_id, ..
            } => {
                let group = bundle
                    .policy_groups
                    .iter()
                    .find(|group| group.id == *policy_group_id)
                    .ok_or_else(|| format!("策略组[{}]不存在", policy_group_id))?;
                Log::info(&format!(
                    "[ scheduler ] 开始调试策略组: {} ({})",
                    group.data.0.name, group.id
                ));
                match executor
                    .debug_execute_policy_group(*policy_group_id)
                    .await
                    .map_err(|error| error.to_string())
                {
                    Ok(result) => {
                        emit_progress_event(
                            RuntimeProgressPhase::Completed,
                            Some(assignment_id),
                            Some(script_id),
                            None,
                            None,
                            Some(format!(
                                "策略组调试完成: {} matched={}",
                                group.data.0.name, result.matched
                            )),
                        );
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            RunTarget::PolicySet { policy_set_id, .. } => {
                let set = bundle
                    .policy_sets
                    .iter()
                    .find(|set| set.id == *policy_set_id)
                    .ok_or_else(|| format!("策略集[{}]不存在", policy_set_id))?;
                Log::info(&format!(
                    "[ scheduler ] 开始调试策略集: {} ({})",
                    set.data.0.name, set.id
                ));
                match executor
                    .debug_execute_policy_set(*policy_set_id)
                    .await
                    .map_err(|error| error.to_string())
                {
                    Ok(result) => {
                        emit_progress_event(
                            RuntimeProgressPhase::Completed,
                            Some(assignment_id),
                            Some(script_id),
                            None,
                            None,
                            Some(format!(
                                "策略集调试完成: {} matched={}",
                                set.data.0.name, result.matched
                            )),
                        );
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            _ => Ok(()),
        };

        {
            let mut ctx = runtime_ctx.write().await;
            ctx.execution.current_execution_id = None;
            ctx.execution.current_assignment_id = None;
            ctx.execution.current_task = None;
            ctx.execution.current_step_id = None;
            if let Err(error) = ctx.observation.vision_text_cache.flush_current_script() {
                Log::warn(&format!(
                    "[ scheduler ] 脚本[{}]调试执行后写回 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        if let Err(error) = &result {
            emit_progress_event(
                RuntimeProgressPhase::Failed,
                Some(assignment_id),
                Some(script_id),
                None,
                None,
                Some(error.clone()),
            );
        }

        result
    }

    /// 清空队列
    pub async fn clear_queue(&self) {
        self.queue.write().await.clear();
        Log::info("[ scheduler ] 队列已清空");
    }

    /// 清空当前会话
    pub async fn clear_session(&self) {
        self.clear_queue().await;
        *self.current_script.write().await = None;
        Log::info("[ scheduler ] 当前 session 已清空");
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptScheduler;
    use crate::domain::devices::device_schedule::TaskCycle;
    use crate::domain::scripts::script_decision::{Step, StepKind};
    use crate::domain::scripts::script_task::{
        ScriptTask, ScriptTaskTable, TaskRowType, TaskTone, TaskTriggerMode,
    };
    use crate::infrastructure::core::{StepId, TaskId, UuidV7};
    use crate::infrastructure::ipc::message::{ResumeCheckpoint, ResumeMode, RunTarget};
    use serde_json::Value;
    use sqlx::types::Json;
    use std::collections::VecDeque;

    fn build_step(id: StepId) -> Step {
        Step {
            id: Some(id),
            source_id: None,
            target_id: None,
            label: None,
            skip_flag: false,
            kind: StepKind::Sequence { steps: Vec::new() },
        }
    }

    fn build_task(
        id: TaskId,
        index: u32,
        trigger_mode: TaskTriggerMode,
        steps: Vec<Step>,
    ) -> ScriptTaskTable {
        ScriptTaskTable {
            id,
            script_id: UuidV7(1),
            name: format!("task-{}", index),
            row_type: TaskRowType::Task,
            trigger_mode,
            record_schedule: false,
            section_id: None,
            indent_level: 0,
            default_task_cycle: Json(TaskCycle::EveryRun),
            exec_max: 0,
            show_enabled_toggle: true,
            default_enabled: true,
            task_tone: TaskTone::Normal,
            is_hidden: false,
            data: Json(ScriptTask {
                ui_data: Value::Null,
                variables: Value::Null,
                steps,
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            index,
        }
    }

    fn build_checkpoint(
        task_id: TaskId,
        step_id: Option<StepId>,
        resume_mode: ResumeMode,
    ) -> ResumeCheckpoint {
        ResumeCheckpoint {
            execution_id: UuidV7(1),
            source_session_id: UuidV7(1),
            device_id: UuidV7(1),
            run_target: RunTarget::FullScript {
                script_id: UuidV7(1),
            },
            assignment_id: None,
            script_id: UuidV7(1),
            time_template_id: None,
            account_id: None,
            task_id: Some(task_id),
            step_id,
            resume_mode,
            definition_fingerprint: "fp".to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn apply_resume_checkpoint_trims_root_tasks_before_target() {
        let task_a = build_task(UuidV7(11), 10, TaskTriggerMode::RootOnly, Vec::new());
        let task_b = build_task(UuidV7(22), 20, TaskTriggerMode::RootOnly, Vec::new());
        let task_c = build_task(UuidV7(33), 30, TaskTriggerMode::RootOnly, Vec::new());
        let mut pending = VecDeque::from(vec![
            crate::infrastructure::scripts::execution_plan::PlannedTask {
                task: task_a,
                task_cycle: TaskCycle::EveryRun,
                allow_root: true,
                allow_link: false,
            },
            crate::infrastructure::scripts::execution_plan::PlannedTask {
                task: task_b.clone(),
                task_cycle: TaskCycle::EveryRun,
                allow_root: true,
                allow_link: false,
            },
            crate::infrastructure::scripts::execution_plan::PlannedTask {
                task: task_c.clone(),
                task_cycle: TaskCycle::EveryRun,
                allow_root: true,
                allow_link: false,
            },
        ]);

        let message = ScriptScheduler::apply_resume_checkpoint_to_pending_tasks(
            &mut pending,
            &std::collections::HashMap::new(),
            &build_checkpoint(task_b.id, None, ResumeMode::FromTaskStart),
        )
        .unwrap();

        assert!(message.contains(&format!("{}", task_b.id)));
        assert_eq!(pending.len(), 2);
        assert_eq!(pending.front().map(|task| task.task.id), Some(task_b.id));
        assert_eq!(pending.back().map(|task| task.task.id), Some(task_c.id));
    }

    #[test]
    fn apply_resume_checkpoint_can_restore_link_only_task() {
        let task_a = build_task(UuidV7(11), 10, TaskTriggerMode::RootOnly, Vec::new());
        let task_b = build_task(UuidV7(22), 20, TaskTriggerMode::LinkOnly, Vec::new());
        let task_c = build_task(UuidV7(33), 30, TaskTriggerMode::RootOnly, Vec::new());
        let mut pending = VecDeque::from(vec![
            crate::infrastructure::scripts::execution_plan::PlannedTask {
                task: task_a,
                task_cycle: TaskCycle::EveryRun,
                allow_root: true,
                allow_link: false,
            },
            crate::infrastructure::scripts::execution_plan::PlannedTask {
                task: task_c.clone(),
                task_cycle: TaskCycle::EveryRun,
                allow_root: true,
                allow_link: false,
            },
        ]);
        let mut linkable = std::collections::HashMap::new();
        linkable.insert(
            task_b.id,
            crate::infrastructure::scripts::execution_plan::PlannedTask {
                task: task_b.clone(),
                task_cycle: TaskCycle::EveryRun,
                allow_root: false,
                allow_link: true,
            },
        );

        ScriptScheduler::apply_resume_checkpoint_to_pending_tasks(
            &mut pending,
            &linkable,
            &build_checkpoint(task_b.id, None, ResumeMode::FromTaskStart),
        )
        .unwrap();

        assert_eq!(pending.len(), 2);
        assert_eq!(pending.front().map(|task| task.task.id), Some(task_b.id));
        assert_eq!(pending.back().map(|task| task.task.id), Some(task_c.id));
    }

    #[test]
    fn resolve_task_resume_cursor_supports_step_start_and_next_step() {
        let step_a = UuidV7(101);
        let step_b = UuidV7(102);
        let step_c = UuidV7(103);
        let task = build_task(
            UuidV7(22),
            20,
            TaskTriggerMode::RootOnly,
            vec![build_step(step_a), build_step(step_b), build_step(step_c)],
        );

        let from_step = ScriptScheduler::resolve_task_resume_cursor(
            &task,
            &build_checkpoint(task.id, Some(step_b), ResumeMode::FromStepStart),
        );
        let from_next = ScriptScheduler::resolve_task_resume_cursor(
            &task,
            &build_checkpoint(task.id, Some(step_b), ResumeMode::FromNextStep),
        );

        assert_eq!(from_step.start_index, 1);
        assert_eq!(from_next.start_index, 2);
        assert!(from_step.message.contains(&format!("{}", step_b)));
        assert!(from_next.message.contains(&format!("{}", step_b)));
    }

    #[test]
    fn resolve_task_resume_cursor_falls_back_to_task_start_when_step_missing() {
        let task = build_task(
            UuidV7(22),
            20,
            TaskTriggerMode::RootOnly,
            vec![build_step(UuidV7(101))],
        );

        let cursor = ScriptScheduler::resolve_task_resume_cursor(
            &task,
            &build_checkpoint(task.id, Some(UuidV7(999)), ResumeMode::FromStepStart),
        );

        assert_eq!(cursor.start_index, 0);
        assert!(cursor.message.contains("降级"));
    }
}
