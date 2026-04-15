// 脚本调度器
// 管理子进程中的脚本执行队列，按顺序执行脚本

use crate::infrastructure::context::runtime_context::get_runtime_ctx;
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_info::{ScriptInfo, ScriptTable};
use crate::domain::scripts::script_task::{ScriptTaskTable, TaskRowType};
use crate::domain::devices::device_schedule::RunStatus;
use crate::infrastructure::core::ExecutionId;
use crate::infrastructure::core::ScriptId;
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::ocr_service::OcrService;
use crate::infrastructure::vision::rec::RecognizerType;
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeProgressPhase, RuntimeQueueItem, RuntimeScheduleStatus,
    RuntimeSessionSnapshot,
};
use crate::infrastructure::ipc::runtime_reporter::{emit_progress_event, emit_schedule_event};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::execution_plan::ExecutionPlanAssembler;
use crate::infrastructure::scripts::executor::ScriptExecutor;
use crate::infrastructure::scripts::schedule_journal::ScheduleJournal;
use crate::infrastructure::session::runtime_session::{
    get_script_bundle_snapshot, try_current_session_summary,
};
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
        serde_json::from_str(&json)
            .map_err(|error| format!("复制 {} 配置失败: {}", field, error))
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
        let script_info = bundle.script.data.0;
        let script_name = script_info.name.clone();
        Self::configure_visual_services(&runtime_ctx, &script_info).await?;
        let run_target = Self::current_run_target();
        let task_selection =
            ExecutionPlanAssembler::select_tasks(&run_target, device_id, &queue_item, &bundle.tasks)
                .await?;
        let runnable_task_count = task_selection.root_tasks.len();
        let skipped_task_count = task_selection.skipped_tasks.len();
        let linkable_task_count = task_selection.linkable_tasks.len();

        // 更新运行时上下文的 script_id
        {
            let mut ctx = runtime_ctx.write().await;
            ctx.execution.current_execution_id = Some(execution_id);
            ctx.execution.current_assignment_id = Some(assignment_id);
            ctx.execution.script_id = script_id;
            ctx.execution.target = run_target;
            ctx.execution.script_info = Some(script_info);
            ctx.execution.current_task = None;
            ctx.execution.current_step_id = None;
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

            if skipped.task.record_schedule && matches!(skipped.task.row_type, TaskRowType::Task) {
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

        let mut executor = ScriptExecutor::new(runtime_ctx.clone());
        let mut pending_tasks: VecDeque<_> = task_selection.root_tasks.into_iter().collect();
        let linkable_tasks = task_selection.linkable_tasks;
        while let Some(planned_task) = pending_tasks.pop_front() {
            let task_cycle = planned_task.task_cycle;
            let task = planned_task.task;
            let task_started_at = chrono::Utc::now().to_rfc3339();
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
                Some(format!("开始执行任务: {}", task.name)),
            );

            executor.reset_node_indices();
            let task_result = executor
                .execute(task.data.0.steps.as_slice())
                .await;

            let completion_at = chrono::Utc::now().to_rfc3339();
            match task_result {
                Ok(flow) => {
                    let linked_task = match flow {
                        crate::infrastructure::scripts::executor::ControlFlow::Link(target) => {
                            Some(
                                linkable_tasks
                                    .get(&target)
                                    .cloned()
                                    .ok_or_else(|| {
                                        format!(
                                            "跳转目标任务[{}]不存在，或不允许通过 link 进入",
                                            target
                                        )
                                    })?,
                            )
                        }
                        _ => None,
                    };
                    let link_target = linked_task.as_ref().map(|planned| planned.task.id);
                    emit_schedule_event(
                        RuntimeScheduleStatus::Success,
                        Some(execution_id),
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(match link_target {
                            Some(target) => {
                                format!("任务执行完成并跳转到任务[{}]: {}", target, task.name)
                            }
                            None => format!("任务执行完成: {}", task.name),
                        }),
                    );
                    emit_progress_event(
                        RuntimeProgressPhase::Completed,
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(match link_target {
                            Some(target) => {
                                format!("任务执行完成，下一步跳转到任务[{}]", target)
                            }
                            None => format!("任务执行完成: {}", task.name),
                        }),
                    );

                    if task.record_schedule && matches!(task.row_type, TaskRowType::Task) {
                        ScheduleJournal::append_task_record(
                            device_id,
                            execution_id,
                            assignment_id,
                            script_id,
                            &task,
                            &task_cycle,
                            RunStatus::Success,
                            task_started_at.clone(),
                            Some(completion_at.clone()),
                            None,
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
                        if let Some(position) =
                            pending_tasks.iter().position(|planned| planned.task.id == target)
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

                    if task.record_schedule && matches!(task.row_type, TaskRowType::Task) {
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
                        if let Err(error) = ctx.observation.vision_text_cache.flush_current_script() {
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

        Ok(())
    }

    /// 开发者调试执行 — 直接执行指定目标，不走队列
    pub async fn debug_execute(
        &self,
        target: RunTarget,
    ) -> Result<(), String> {
        let script_id = target
            .script_id()
            .ok_or_else(|| "调试运行目标必须携带 script_id".to_string())?;
        Log::info(&format!(
            "[ scheduler ] 调试执行脚本[{}] target: {:?}",
            script_id, target
        ));

        let bundle = Self::load_script_bundle(script_id).await?;
        let runtime_ctx = get_runtime_ctx();
        let script_info = bundle.script.data.0;
        let script_name = script_info.name.clone();
        Self::configure_visual_services(&runtime_ctx, &script_info).await?;
        {
            let mut ctx = runtime_ctx.write().await;
            ctx.execution.script_id = script_id;
            ctx.execution.target = target.clone();
            ctx.execution.script_info = Some(script_info);
            ctx.execution.current_task = None;
            ctx.observation.last_snapshot = None;
            ctx.observation.last_hits.clear();
            if let Err(error) = ctx
                .observation
                .vision_text_cache
                .load_for_script(script_id, &script_name)
            {
                Log::warn(&format!(
                    "[ scheduler ] 调试执行脚本[{}]加载 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        let _executor = ScriptExecutor::new(runtime_ctx.clone());

        // TODO: M4 根据 target 装配 bundle 内对应的步骤。
        // match target {
        //     RunTarget::FullScript { script_id } => { ... }
        //     RunTarget::Task { script_id, task_id } => { ... }
        //     RunTarget::PolicyGroup { script_id, policy_group_id } => { ... }
        //     RunTarget::PolicySet { script_id, policy_set_id } => { ... }
        // }

        Log::info(&format!(
            "[ scheduler ] 调试执行脚本[{}]完成 (TODO: 实际执行逻辑)",
            script_id
        ));

        {
            let mut ctx = runtime_ctx.write().await;
            if let Err(error) = ctx.observation.vision_text_cache.flush_current_script() {
                Log::warn(&format!(
                    "[ scheduler ] 调试执行脚本[{}]写回 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        Ok(())
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
