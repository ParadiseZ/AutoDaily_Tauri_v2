// 脚本调度器
// 管理子进程中的脚本执行队列，按顺序执行脚本

use crate::infra::context::{TaskState, runtime_context::get_runtime_ctx};
use crate::infra::ipc::runtime_reporter::{
    emit_dispatch_event, emit_progress_event, emit_schedule_event,
};
use crate::infra::logging::log_trait::Log;
use crate::infra::scripts::execution_plan::ExecutionPlanAssembler;
use crate::infra::scripts::executor::ScriptExecutor;
use crate::infra::scripts::schedule_journal::ScheduleJournal;
use crate::infra::session::runtime_session::{
    get_script_bundle_snapshot, try_current_session_summary,
};
use ad_kernel::ids::ExecutionId;
use ad_kernel::ids::ScriptId;
use domain_schedule::TaskRunStatus;
use domain_script::ScriptInfo;
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptProfile, ScriptTaskProfile,
};
use domain_vision::DetectorType;
use domain_vision::RecognizerType;
use infra_vision::OcrService;
use runner_protocol::message::{
    RunTarget, RuntimeDispatchPhase, RuntimeProgressPhase, RuntimeQueueItem, RuntimeScheduleStatus,
    RuntimeSessionSnapshot,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

enum ScriptExecutionOutcome {
    Completed(String),
    Stopped(String),
}

struct ParsedScriptBundle {
    script: ScriptProfile,
    tasks: Vec<ScriptTaskProfile>,
    policies: Vec<PolicyProfile>,
    policy_groups: Vec<PolicyGroupProfile>,
    policy_sets: Vec<PolicySetProfile>,
    group_policies: Vec<PolicyGroupPolicyLink>,
    set_groups: Vec<PolicySetGroupLink>,
}

/// 脚本调度器
pub(crate) struct ScriptScheduler {
    /// 待执行的脚本队列
    queue: Arc<RwLock<VecDeque<RuntimeQueueItem>>>,
    /// 当前正在执行的脚本
    current_script: Arc<RwLock<Option<ScriptId>>>,
    /// 取消令牌
    cancel_token: CancellationToken,
    #[cfg(feature = "testkit")]
    test_hooks: Option<Arc<crate::testkit::TestRuntimeHooks>>,
}

impl ScriptScheduler {
    pub(crate) fn new(cancel_token: CancellationToken) -> Arc<Self> {
        Arc::new(Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            current_script: Arc::new(RwLock::new(None)),
            cancel_token,
            #[cfg(feature = "testkit")]
            test_hooks: None,
        })
    }

    #[cfg(feature = "testkit")]
    pub(crate) fn new_with_test_hooks(
        cancel_token: CancellationToken,
        test_hooks: Arc<crate::testkit::TestRuntimeHooks>,
    ) -> Arc<Self> {
        Arc::new(Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            current_script: Arc::new(RwLock::new(None)),
            cancel_token,
            test_hooks: Some(test_hooks),
        })
    }

    fn create_executor(
        &self,
        runtime_ctx: Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
    ) -> ScriptExecutor {
        #[cfg(feature = "testkit")]
        if let Some(test_hooks) = self.test_hooks.as_ref() {
            return ScriptExecutor::new_with_test_hooks(runtime_ctx, test_hooks.clone());
        }

        ScriptExecutor::new(runtime_ctx)
    }
}

impl ScriptScheduler {
    fn runtime_model_config<T>(field: &str, value: &impl Serialize) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let json = serde_json::to_string(value)
            .map_err(|error| format!("序列化 {} 配置失败: {}", field, error))?;
        serde_json::from_str(&json).map_err(|error| format!("复制 {} 配置失败: {}", field, error))
    }

    async fn configure_visual_services(
        runtime_ctx: &Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
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
                let detector = Self::runtime_model_config::<DetectorType>("img_det_model", model)?;
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
                let detector = Self::runtime_model_config::<DetectorType>("txt_det_model", model)?;
                service
                    .init_detector(detector)
                    .await
                    .map_err(|error| format!("初始化文字检测模型失败: {}", error))?;
            }
            if let Some(model) = script_info.txt_rec_model.as_ref() {
                let recognizer =
                    Self::runtime_model_config::<RecognizerType>("txt_rec_model", model)?;
                service
                    .init_recognizer(recognizer)
                    .await
                    .map_err(|error| format!("初始化文字识别模型失败: {}", error))?;
            }
        }

        Ok(())
    }

    async fn reset_execution_state(
        runtime_ctx: &Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
    ) {
        let mut ctx = runtime_ctx.write().await;
        ctx.execution.current_execution_id = None;
        ctx.execution.current_assignment_id = None;
        ctx.execution.current_task = None;
        ctx.execution.current_step_id = None;
        ctx.execution.current_step_name = None;
    }

    async fn flush_ocr_cache(
        runtime_ctx: &Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
        script_name: &str,
        log_context: &str,
    ) -> Result<(), String> {
        let mut ctx = runtime_ctx.write().await;
        ctx.vision_text_cache
            .flush_current_script()
            .map_err(|error| {
                format!(
                    "脚本[{}]{}写入 OCR 文字缓存失败: {}",
                    script_name, log_context, error
                )
            })
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
        runtime_ctx: &Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
        task_id: ad_kernel::ids::TaskId,
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
        runtime_ctx: &Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
        task_id: ad_kernel::ids::TaskId,
    ) {
        let mut ctx = runtime_ctx.write().await;
        let state = ctx.execution.task_states.entry(task_id).or_default();
        state.done_flag = true;
        state.exec_cur = state.exec_cur.saturating_add(1);
    }

    /// 用完整 session 替换当前队列
    pub(crate) async fn load_session(&self, session: RuntimeSessionSnapshot) {
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

    /// 非阻塞快照读取，供事件上报使用
    pub(crate) fn current_script_snapshot(&self) -> Option<ScriptId> {
        self.current_script.try_read().ok().and_then(|guard| *guard)
    }

    /// 调度循环 — 在 child bootstrap 的 Running 状态下调用
    /// 从队列取出脚本执行，执行完后取下一个
    /// 返回 true 表示还有任务可执行，false 表示队列为空
    pub(crate) async fn tick(&self) -> bool {
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
        let dispatch_id = queue_item.dispatch_id;
        let execution_id = ExecutionId::new_v7();

        // 标记当前脚本
        *self.current_script.write().await = Some(script_id);
        emit_dispatch_event(
            Some(dispatch_id),
            Some(assignment_id),
            Some(script_id),
            RuntimeDispatchPhase::Started,
            Some("dispatch 已开始执行".to_string()),
        );
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
            Ok(ScriptExecutionOutcome::Completed(script_name)) => {
                Log::info(&format!("[ scheduler ] 脚本[{}]执行完成", script_name));
                emit_dispatch_event(
                    Some(dispatch_id),
                    Some(assignment_id),
                    Some(script_id),
                    RuntimeDispatchPhase::Finished,
                    Some("dispatch 执行完成".to_string()),
                );
            }
            Ok(ScriptExecutionOutcome::Stopped(script_name)) => {
                Log::info(&format!("[ scheduler ] 脚本[{}]已停止", script_name));
                self.clear_queue().await;
                emit_dispatch_event(
                    Some(dispatch_id),
                    Some(assignment_id),
                    Some(script_id),
                    RuntimeDispatchPhase::Stopped,
                    Some("dispatch 已停止".to_string()),
                );
                return false;
            }
            Err(e) => {
                Log::error(&format!("[ scheduler ] 脚本执行失败: {}", e));
                emit_dispatch_event(
                    Some(dispatch_id),
                    Some(assignment_id),
                    Some(script_id),
                    RuntimeDispatchPhase::Failed,
                    Some(e.clone()),
                );
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
                Log::warn("[ scheduler ] 当前脚本执行失败，保留剩余队列继续调度");
                return self.queue.read().await.len() > 0;
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
    ) -> Result<ScriptExecutionOutcome, String> {
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
        let script_info = bundle.script.info;
        let script_name = script_info.name.clone();
        let variable_catalog = script_info.variable_catalog.clone();
        Log::info(&format!("[ scheduler ] 开始执行脚本: {}", script_name));
        let capture_asset_signature = ScriptExecutor::build_capture_asset_signature(&script_info);
        let text_rec_model_signature = {
            let ctx = runtime_ctx.read().await;
            if ctx.vision_text_cache.is_enabled() {
                ScriptExecutor::build_text_rec_model_signature(&script_info)
            } else {
                String::new()
            }
        };
        Self::configure_visual_services(&runtime_ctx, &script_info).await?;
        let run_target = Self::current_run_target();
        let execution_plan =
            ExecutionPlanAssembler::assemble(&run_target, device_id, &queue_item, &bundle.tasks)
                .await?;
        let plan_summary = execution_plan.summary();
        let is_policy_debug_target = execution_plan.is_policy_debug();
        let task_selection = execution_plan.task_selection();
        // 更新运行时上下文的 script_id
        {
            let mut ctx = runtime_ctx.write().await;
            ctx.execution.current_execution_id = Some(execution_id);
            ctx.execution.current_assignment_id = Some(assignment_id);
            ctx.execution.current_device_id = Some(device_id);
            ctx.execution.current_time_template_id = queue_item.time_template_id;
            ctx.execution.current_account_id = queue_item.account_id.clone();
            ctx.execution.script_id = script_id;
            ctx.execution.target = run_target.clone();
            ctx.execution.script_info = Some(script_info);
            ctx.execution.current_task = None;
            ctx.execution.current_step_id = None;
            ctx.execution.current_step_name = None;
            ctx.execution.var_map.clear();
            ctx.execution.template_values_json = queue_item.template_values_json.clone();
            ctx.execution.policy_states.clear();
            ctx.execution.task_states.clear();
            ctx.execution
                .task_states
                .extend(bundle.tasks.iter().map(|task| {
                    (
                        task.id,
                        TaskState {
                            enabled_flag: task.default_enabled,
                            ..TaskState::default()
                        },
                    )
                }));
            ctx.execution.action_states.clear();
            ctx.execution.policy_set_bindings.clear();
            ctx.execution.policy_group_bindings.clear();
            ctx.execution.policy_set_candidate_cache_ready = false;
            ctx.execution.policy_set_candidate_cache.clear();
            ctx.observation.last_capture_image = None;
            ctx.observation.last_snapshot = None;
            ctx.observation.last_hits.clear();
            ctx.observation.capture_asset_signature = capture_asset_signature;
            ctx.observation.text_rec_model_signature = text_rec_model_signature;
            ctx.vision_text_cache
                .load_for_script(script_id, &script_name)
                .map_err(|error| {
                    format!("脚本[{}]加载 OCR 文字缓存失败: {}", script_name, error)
                })?;
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
            "[ scheduler ] 脚本[{}] bundle 已加载，mode={}, task={}, root_task={}, linkable_task={}, skipped_task={}, policy={}, group_relation={}, set_relation={}",
            script_name,
            plan_summary.mode_label(),
            tasks_len,
            plan_summary.root_task_count,
            plan_summary.linkable_task_count,
            plan_summary.skipped_task_count,
            policy_count,
            group_policy_count,
            set_group_count
        ));
        emit_progress_event(
            RuntimeProgressPhase::Executing,
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some(execution_plan.progress_message()),
        );

        if is_policy_debug_target {
            Self::execute_debug_policy_target(
                &run_target,
                assignment_id,
                script_id,
                &variable_catalog,
                &bundle.policies,
                &bundle.policy_groups,
                &bundle.policy_sets,
                queue_item.template_values_json.as_deref(),
                &runtime_ctx,
            )
            .await
            .map_err(|error| format!("脚本[{}] {}", script_name, error))?;
            if crate::infra::context::runtime_control::stop_requested() {
                return Ok(ScriptExecutionOutcome::Stopped(script_name));
            }
            return Ok(ScriptExecutionOutcome::Completed(script_name));
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

            if skipped.record_schedule {
                let now = chrono::Utc::now().to_rfc3339();
                ScheduleJournal::append_task_record(
                    device_id,
                    execution_id,
                    assignment_id,
                    script_id,
                    &skipped.task,
                    &ScheduleJournal::compute_dedup_scope_hash(
                        &queue_item.dedup_scope_base_hash,
                        skipped.task.id,
                    )?,
                    &skipped.task_cycle,
                    TaskRunStatus::Skipped,
                    now.clone(),
                    Some(now),
                    Some(skipped.reason.clone()),
                )
                .await?;
            }
        }

        let root_tasks = task_selection.root_tasks.clone();
        let mut executor = self.create_executor(runtime_ctx.clone());
        let mut pending_tasks: VecDeque<_> = root_tasks.clone().into_iter().collect();
        let linkable_tasks = task_selection.linkable_tasks;
        while let Some(planned_task) = pending_tasks.pop_front() {
            if crate::infra::context::runtime_control::stop_requested() {
                Self::reset_execution_state(&runtime_ctx).await;
                Self::flush_ocr_cache(&runtime_ctx, &script_name, "停止后").await?;
                emit_progress_event(
                    RuntimeProgressPhase::Stopping,
                    Some(assignment_id),
                    Some(script_id),
                    None,
                    None,
                    Some(format!("脚本已停止: {}", script_name)),
                );
                emit_schedule_event(
                    RuntimeScheduleStatus::Stopped,
                    Some(execution_id),
                    Some(assignment_id),
                    Some(script_id),
                    None,
                    None,
                    Some("收到停止命令，后续任务不再执行".to_string()),
                );
                return Ok(ScriptExecutionOutcome::Stopped(script_name));
            }
            let task_cycle = planned_task.task_cycle;
            let task = planned_task.task;
            let record_schedule = planned_task.record_schedule;
            let task_started_at = chrono::Utc::now().to_rfc3339();
            {
                let mut ctx = runtime_ctx.write().await;
                ctx.execution.current_task = Some(task.clone());
                ctx.execution.current_step_id = None;
                ctx.execution.current_step_name = None;
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
            executor
                .hydrate_input_scope(
                    &variable_catalog,
                    queue_item.template_values_json.as_deref(),
                    Some(&task),
                )
                .await
                .map_err(|error| error.to_string())?;
            let task_result = executor.execute(&task.task.steps).await;

            let completion_at = chrono::Utc::now().to_rfc3339();
            let stop_requested = crate::infra::context::runtime_control::stop_requested();
            match task_result {
                Ok(flow) => {
                    let task_skipped = Self::consume_task_skip_flag(&runtime_ctx, task.id).await;
                    if !task_skipped {
                        Self::mark_task_succeeded(&runtime_ctx, task.id).await;
                    }
                    let stop_script = matches!(
                        &flow,
                        crate::infra::scripts::executor::ControlFlow::StopScript
                    );
                    let externally_stopped = stop_requested;
                    let linked_task = match &flow {
                        crate::infra::scripts::executor::ControlFlow::Link(target) => {
                            Some(linkable_tasks.get(target).cloned().ok_or_else(|| {
                                format!("跳转目标任务[{}]不存在，或不允许通过 link 进入", target)
                            })?)
                        }
                        _ => None,
                    };
                    let link_target = linked_task.as_ref().map(|planned| planned.task.id);
                    let schedule_status = if externally_stopped {
                        RuntimeScheduleStatus::Stopped
                    } else if task_skipped {
                        RuntimeScheduleStatus::Skipped
                    } else {
                        RuntimeScheduleStatus::Success
                    };
                    let schedule_message = if externally_stopped {
                        format!("收到停止命令，任务已中断: {}", task.name)
                    } else if stop_script {
                        format!("脚本已跳过后续任务: {}", task.name)
                    } else if task_skipped {
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
                        if externally_stopped {
                            RuntimeProgressPhase::Stopping
                        } else {
                            RuntimeProgressPhase::Completed
                        },
                        Some(assignment_id),
                        Some(script_id),
                        Some(task.id),
                        None,
                        Some(if externally_stopped {
                            format!("收到停止命令，任务已中断: {}", task.name)
                        } else if stop_script {
                            format!("脚本已跳过后续任务: {}", task.name)
                        } else if task_skipped {
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

                    if record_schedule {
                        ScheduleJournal::append_task_record(
                            device_id,
                            execution_id,
                            assignment_id,
                            script_id,
                            &task,
                            &ScheduleJournal::compute_dedup_scope_hash(
                                &queue_item.dedup_scope_base_hash,
                                task.id,
                            )?,
                            &task_cycle,
                            if externally_stopped {
                                TaskRunStatus::Stopped
                            } else if task_skipped {
                                TaskRunStatus::Skipped
                            } else {
                                TaskRunStatus::Success
                            },
                            task_started_at.clone(),
                            Some(completion_at.clone()),
                            if externally_stopped {
                                Some("收到停止命令，任务已中断".to_string())
                            } else if stop_script {
                                Some("任务触发跳过脚本，后续任务不再执行".to_string())
                            } else {
                                task_skipped.then(|| "任务在执行过程中被标记为跳过".to_string())
                            },
                        )
                        .await?;
                    }
                    {
                        let mut ctx = runtime_ctx.write().await;
                        ctx.execution.current_task = None;
                        ctx.execution.current_step_id = None;
                        ctx.execution.current_step_name = None;
                    }

                    if stop_script {
                        pending_tasks.clear();
                    } else if let Some(linked_task) = linked_task {
                        let target = linked_task.task.id;
                        if let Some(position) = pending_tasks
                            .iter()
                            .position(|planned| planned.task.id == target)
                        {
                            pending_tasks.remove(position);
                        }
                        pending_tasks.push_front(linked_task);
                    }
                    if externally_stopped {
                        Self::reset_execution_state(&runtime_ctx).await;
                        Self::flush_ocr_cache(&runtime_ctx, &script_name, "停止后").await?;
                        emit_schedule_event(
                            RuntimeScheduleStatus::Stopped,
                            Some(execution_id),
                            Some(assignment_id),
                            Some(script_id),
                            None,
                            None,
                            Some("收到停止命令，当前脚本已停止".to_string()),
                        );
                        return Ok(ScriptExecutionOutcome::Stopped(script_name));
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    if stop_requested {
                        if record_schedule {
                            ScheduleJournal::append_task_record(
                                device_id,
                                execution_id,
                                assignment_id,
                                script_id,
                                &task,
                                &ScheduleJournal::compute_dedup_scope_hash(
                                    &queue_item.dedup_scope_base_hash,
                                    task.id,
                                )?,
                                &task_cycle,
                                TaskRunStatus::Stopped,
                                task_started_at,
                                Some(completion_at),
                                Some("收到停止命令，任务已中断".to_string()),
                            )
                            .await?;
                        }
                        emit_schedule_event(
                            RuntimeScheduleStatus::Stopped,
                            Some(execution_id),
                            Some(assignment_id),
                            Some(script_id),
                            Some(task.id),
                            None,
                            Some(format!("收到停止命令，任务已中断: {}", task.name)),
                        );
                        emit_progress_event(
                            RuntimeProgressPhase::Stopping,
                            Some(assignment_id),
                            Some(script_id),
                            Some(task.id),
                            None,
                            Some(format!("收到停止命令，任务已中断: {}", task.name)),
                        );
                        Self::reset_execution_state(&runtime_ctx).await;
                        Self::flush_ocr_cache(&runtime_ctx, &script_name, "停止后").await?;
                        emit_schedule_event(
                            RuntimeScheduleStatus::Stopped,
                            Some(execution_id),
                            Some(assignment_id),
                            Some(script_id),
                            None,
                            None,
                            Some("收到停止命令，当前脚本已停止".to_string()),
                        );
                        return Ok(ScriptExecutionOutcome::Stopped(script_name));
                    }
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

                    if record_schedule {
                        ScheduleJournal::append_task_record(
                            device_id,
                            execution_id,
                            assignment_id,
                            script_id,
                            &task,
                            &ScheduleJournal::compute_dedup_scope_hash(
                                &queue_item.dedup_scope_base_hash,
                                task.id,
                            )?,
                            &task_cycle,
                            TaskRunStatus::Failed,
                            task_started_at,
                            Some(completion_at),
                            Some(message.clone()),
                        )
                        .await?;
                    }

                    Self::reset_execution_state(&runtime_ctx).await;
                    Self::flush_ocr_cache(&runtime_ctx, &script_name, "失败后").await?;
                    return Err(format!("脚本[{}] {}", script_name, message));
                }
            }
        }

        Self::reset_execution_state(&runtime_ctx).await;
        Self::flush_ocr_cache(&runtime_ctx, &script_name, "").await?;

        emit_progress_event(
            RuntimeProgressPhase::Completed,
            Some(assignment_id),
            Some(script_id),
            None,
            None,
            Some(format!(
                "脚本执行完成，共 {} 个任务",
                plan_summary.root_task_count
            )),
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
                plan_summary.root_task_count, plan_summary.skipped_task_count
            )),
        );

        Ok(ScriptExecutionOutcome::Completed(script_name))
    }

    #[cfg(feature = "testkit")]
    pub(crate) async fn execute_test_item(
        &self,
        queue_item: RuntimeQueueItem,
    ) -> Result<bool, String> {
        match self
            .execute_script(queue_item, ExecutionId::new_v7())
            .await?
        {
            ScriptExecutionOutcome::Completed(_) => Ok(false),
            ScriptExecutionOutcome::Stopped(_) => Ok(true),
        }
    }

    async fn execute_debug_policy_target(
        run_target: &RunTarget,
        assignment_id: ad_kernel::ids::AssignmentId,
        script_id: ScriptId,
        variable_catalog: &domain_script::ScriptVariableCatalog,
        policies: &[PolicyProfile],
        policy_groups: &[PolicyGroupProfile],
        policy_sets: &[PolicySetProfile],
        template_values_json: Option<&str>,
        runtime_ctx: &Arc<RwLock<crate::infra::context::runtime_context::RuntimeContext>>,
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
        executor
            .hydrate_input_scope(variable_catalog, template_values_json, None)
            .await
            .map_err(|error| error.to_string())?;
        let result = match run_target {
            RunTarget::Policy { policy_id, .. } => {
                let policy = policies
                    .iter()
                    .find(|policy| policy.id == *policy_id)
                    .ok_or_else(|| format!("策略[{}]不存在", policy_id))?;
                Log::info(&format!(
                    "[ scheduler ] 开始调试策略: {} ({})",
                    policy.info.name, policy.id
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
                                policy.info.name, result.matched
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
                let group = policy_groups
                    .iter()
                    .find(|group| group.id == *policy_group_id)
                    .ok_or_else(|| format!("策略组[{}]不存在", policy_group_id))?;
                Log::info(&format!(
                    "[ scheduler ] 开始调试策略组: {} ({})",
                    group.info.name, group.id
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
                                group.info.name, result.matched
                            )),
                        );
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            RunTarget::PolicySet { policy_set_id, .. } => {
                let set = policy_sets
                    .iter()
                    .find(|set| set.id == *policy_set_id)
                    .ok_or_else(|| format!("策略集[{}]不存在", policy_set_id))?;
                Log::info(&format!(
                    "[ scheduler ] 开始调试策略集: {} ({})",
                    set.info.name, set.id
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
                                set.info.name, result.matched
                            )),
                        );
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            _ => Ok(()),
        };

        Self::reset_execution_state(runtime_ctx).await;
        Self::flush_ocr_cache(runtime_ctx, &script_id.to_string(), "调试执行后").await?;

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
    pub(crate) async fn clear_queue(&self) {
        self.queue.write().await.clear();
        Log::info("[ scheduler ] 队列已清空");
    }

    /// 清空当前会话
    pub(crate) async fn clear_session(&self) {
        self.clear_queue().await;
        *self.current_script.write().await = None;
        Log::info("[ scheduler ] 当前 session 已清空");
    }
}
