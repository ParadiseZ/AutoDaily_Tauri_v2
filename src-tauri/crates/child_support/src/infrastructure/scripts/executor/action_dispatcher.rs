struct ActionRunner<'a> {
    executor: &'a mut ScriptExecutor,
}

impl ScriptExecutor {
    async fn execute_action_step(
        &mut self,
        step_id: Option<StepId>,
        step_label: Option<&str>,
        exec_max: u32,
        action: &Action,
    ) -> ExecuteResult<ControlFlow> {
        ActionRunner::new(self)
            .execute_action_step(step_id, step_label, exec_max, action)
            .await
    }

    async fn try_execute_action_sequence(
        &mut self,
        steps: &[Step],
    ) -> ExecuteResult<Option<ControlFlow>> {
        ActionRunner::new(self).try_execute_action_sequence(steps).await
    }
}

impl<'a> ActionRunner<'a> {
    fn new(executor: &'a mut ScriptExecutor) -> Self {
        Self { executor }
    }

    async fn execute_simple_device_operation(
        &self,
        step_type: &str,
        label: &str,
        operation: DeviceOperation,
        action_kind: PolicyActionKind,
        timeout_ms: u64,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        ScriptExecutor::await_device_result_with_timeout(
            step_type,
            label,
            timeout_ms,
            get_device_ctx().execute_operation(operation),
        )
        .await?;
        Ok((
            ControlFlow::Next,
            Some(ActionTraceBuilder::build_simple_action_trace(action_kind)),
        ))
    }

    async fn execute_action_step(
        &mut self,
        step_id: Option<StepId>,
        step_label: Option<&str>,
        exec_max: u32,
        action: &Action,
    ) -> ExecuteResult<ControlFlow> {
        if self.should_skip_action_execution(step_id, exec_max).await {
            return Ok(ControlFlow::Next);
        }

        if let Some(timeout_flow) = self
            .executor
            .record_progress_evidence("action.prepare", format!("准备执行动作 {:?}", action))
            .await?
        {
            return Ok(timeout_flow);
        }

        self.before_action(action).await?;
        self.log_active_policy_action(step_label, action);
        let (flow, action_trace) = self.dispatch_action(action).await?;
        if let Some(action_trace) = action_trace.as_ref() {
            self.executor.record_action_trace(action_trace.clone());
        }
        let post_action_flow = self.after_action(action, action_trace.as_ref()).await?;
        let final_flow = post_action_flow.unwrap_or(flow);
        if matches!(final_flow, ControlFlow::Next) {
            self.mark_action_succeeded(step_id).await;
        }
        Ok(final_flow)
    }

    async fn should_skip_action_execution(&self, step_id: Option<StepId>, exec_max: u32) -> bool {
        if exec_max == 0 {
            return false;
        }

        let Some(step_id) = step_id else {
            Log::warn("[ executor ] Action.exec_max 已配置，但当前 action step 缺少 id，无法做运行时计数限制");
            return false;
        };

        let ctx = self.executor.runtime_ctx.read().await;
        let exec_cur = ctx
            .execution
            .action_states
            .get(&step_id)
            .map(|state| state.exec_cur)
            .unwrap_or(0);
        exec_cur >= exec_max
    }

    async fn mark_action_succeeded(&self, step_id: Option<StepId>) {
        let Some(step_id) = step_id else {
            return;
        };

        let mut ctx = self.executor.runtime_ctx.write().await;
        let state = ctx.execution.action_states.entry(step_id).or_default();
        state.exec_cur = state.exec_cur.saturating_add(1);
    }

    fn log_active_policy_action(&self, step_label: Option<&str>, action: &Action) {
        let Some(context) = self.executor.active_policy_context.as_ref() else {
            return;
        };

        let step_label = step_label
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("未命名步骤");
        Log::debug(&format!(
            "[ policy_debug ] 策略[{}] 步骤[{}] 动作: {}",
            context.policy_name,
            step_label,
            Self::describe_action(action)
        ));
    }

    fn describe_action(action: &Action) -> String {
        match action {
            Action::Capture { output_var } => format!("截图输出变量 -> {}", output_var),
            Action::Click {
                mode,
                offset_x,
                offset_y,
            } => format!(
                "点击({}, offset_x={}, offset_y={})",
                Self::describe_click_mode(mode),
                offset_x,
                offset_y
            ),
            Action::Swipe { duration, mode } => {
                format!("滑动(duration={}ms, {})", duration, Self::describe_swipe_mode(mode))
            }
            Action::LongClick {
                mode,
                offset_x,
                offset_y,
            } => format!(
                "长按({}, offset_x={}, offset_y={})",
                Self::describe_click_mode(mode),
                offset_x,
                offset_y
            ),
            Action::Reboot => "重启".to_string(),
            Action::Back => "返回".to_string(),
            Action::Home => "主页".to_string(),
            Action::InputText { text } => format!("输入文本({})", text),
            Action::PosAdd { target } => format!("策略点击位置+1(target={})", target),
            Action::PosMinus { target } => format!("策略点击位置-1(target={})", target),
            Action::DropSetNext { task, variable_id } => {
                format!("写入追加策略集(task={}, variable={})", task, variable_id)
            }
            Action::LaunchApp {
                pkg_name,
                activity_name,
            } => format!("启动应用(pkg={}, activity={})", pkg_name, activity_name),
            Action::StopApp { pkg_name } => format!("停止应用(pkg={})", pkg_name),
        }
    }

    fn describe_click_mode(mode: &ClickMode) -> String {
        match mode {
            ClickMode::Point { p } => format!("point=({}, {})", p.x, p.y),
            ClickMode::Percent { p } => format!("percent=({:.3}, {:.3})", p.x, p.y),
            ClickMode::Txt {
                input_var,
                txt,
                txt_expr,
            } => format!(
                "txt(input_var={}, txt={:?}, txt_expr={:?})",
                input_var, txt, txt_expr
            ),
            ClickMode::LabelIdx { input_var, idx } => {
                format!("labelIdx(input_var={}, idx={:?})", input_var, idx)
            }
        }
    }

    fn describe_swipe_target(target: &SwipeTarget) -> String {
        match target {
            SwipeTarget::Txt {
                input_var,
                value,
                value_expr,
            } => format!(
                "txt(input_var={}, value={:?}, value_expr={:?})",
                input_var, value, value_expr
            ),
            SwipeTarget::LabelIdx { input_var, idx } => {
                format!("labelIdx(input_var={}, idx={})", input_var, idx)
            }
        }
    }

    fn describe_swipe_mode(mode: &SwipeMode) -> String {
        match mode {
            SwipeMode::Point { from, to } => {
                format!("point from=({}, {}) to=({}, {})", from.x, from.y, to.x, to.y)
            }
            SwipeMode::Percent { from, to } => format!(
                "percent from=({:.3}, {:.3}) to=({:.3}, {:.3})",
                from.x, from.y, to.x, to.y
            ),
            SwipeMode::LabelIdx {
                input_var,
                from,
                to,
            } => format!(
                "labelIdx(input_var={}, from={}, to={})",
                input_var, from, to
            ),
            SwipeMode::Txt {
                input_var,
                from,
                to,
                from_expr,
                to_expr,
            } => format!(
                "txt(input_var={}, from={:?}, to={:?}, from_expr={:?}, to_expr={:?})",
                input_var, from, to, from_expr, to_expr
            ),
            SwipeMode::Mixed { from, to } => format!(
                "mixed from={} to={}",
                Self::describe_swipe_target(from),
                Self::describe_swipe_target(to)
            ),
        }
    }

    async fn before_action(&mut self, _action: &Action) -> ExecuteResult<()> {
        Ok(())
    }

    async fn after_action(
        &mut self,
        action: &Action,
        action_trace: Option<&PolicyActionTrace>,
    ) -> ExecuteResult<Option<ControlFlow>> {
        if !ScriptExecutor::action_requires_wait(action) {
            return Ok(None);
        }

        let Some(runtime_policy) = get_runtime_execution_policy().await else {
            Log::warn("[ afterAction ] 当前运行时缺少 RuntimeExecutionPolicy，无法执行后续步骤");
            return Ok(None);
        };

        if runtime_policy.action_wait_ms > 0 {
            tokio::time::sleep(Duration::from_millis(runtime_policy.action_wait_ms)).await;
        }

        self.executor
            .observe_refresh_hook(action, action_trace)
            .await;
        self.executor
            .evaluate_action_progress_timeout(&runtime_policy, action, action_trace)
            .await
    }

    async fn try_execute_action_sequence(
        &mut self,
        steps: &[Step],
    ) -> ExecuteResult<Option<ControlFlow>> {
        let compiled_steps = match self.executor.compile_sequence_operations(steps).await? {
            SequenceCompileOutcome::Supported(compiled_steps) => compiled_steps,
            SequenceCompileOutcome::Unsupported(blocker) => {
                Log::debug(&format!(
                    "[ executor ] Sequence 快路径跳过: step=[{}], reason={}",
                    blocker.step_label, blocker.reason
                ));
                return Ok(None);
            }
        };

        if compiled_steps.is_empty() {
            return Ok(Some(ControlFlow::Next));
        }

        if let Some(timeout_flow) = self
            .executor
            .record_progress_evidence(
                "sequence.prepare",
                format!("准备执行动作序列，共 {} 条设备动作", compiled_steps.len()),
            )
            .await?
        {
            return Ok(Some(timeout_flow));
        }

        let operations = compiled_steps
            .iter()
            .map(|step| step.operation.clone())
            .collect::<Vec<_>>();

        get_device_ctx()
            .execute_sequence(&operations)
            .await
            .map_err(|error| {
                ScriptExecutor::execute_error(
                    "sequence.prepare",
                    format!("执行动作序列失败: {}", error),
                )
            })?;

        for compiled_step in compiled_steps {
            Log::debug(&format!(
                "[ executor ] Sequence 快路径已执行: {}",
                compiled_step.debug_label
            ));
            if let Some(action_trace) = compiled_step.trace {
                self.executor.record_action_trace(action_trace);
            }
        }
        Ok(Some(ControlFlow::Next))
    }

    async fn dispatch_action(
        &mut self,
        action: &Action,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        match action {
            Action::Capture { output_var } => {
                let image = Arc::new(
                    self.executor
                        .capture_device_screenshot("action.capture")
                        .await?,
                );
                self.executor
                    .activate_image_context("action.capture", image, Some(output_var))
                    .await?;
                Ok((ControlFlow::Next, None))
            }
            Action::Click {
                mode,
                offset_x,
                offset_y,
            } => self.execute_click(mode, *offset_x, *offset_y).await,
            Action::Swipe { duration, mode } => self.execute_swipe(mode, *duration).await,
            Action::LongClick {
                mode,
                offset_x,
                offset_y,
            } => self.execute_long_click(mode, *offset_x, *offset_y).await,
            Action::Reboot => self
                .execute_simple_device_operation(
                    "action.reboot",
                    "设备重启",
                    DeviceOperation::Reboot,
                    PolicyActionKind::Reboot,
                    20_000,
                )
                .await,
            Action::Back => self
                .execute_simple_device_operation(
                    "action.back",
                    "返回键",
                    DeviceOperation::Back,
                    PolicyActionKind::Back,
                    DEVICE_EXTERNAL_TIMEOUT_MS,
                )
                .await,
            Action::Home => self
                .execute_simple_device_operation(
                    "action.home",
                    "主页键",
                    DeviceOperation::Home,
                    PolicyActionKind::Home,
                    DEVICE_EXTERNAL_TIMEOUT_MS,
                )
                .await,
            Action::InputText { text } => self
                .execute_simple_device_operation(
                    "action.inputText",
                    "输入文本",
                    DeviceOperation::InputText(text.clone()),
                    PolicyActionKind::Input,
                    DEVICE_EXTERNAL_TIMEOUT_MS,
                )
                .await,
            Action::PosAdd { target } => {
                self.executor.adjust_policy_click_pos(*target, 1).await?;
                Ok((ControlFlow::Next, None))
            }
            Action::PosMinus { target } => {
                self.executor.adjust_policy_click_pos(*target, -1).await?;
                Ok((ControlFlow::Next, None))
            }
            Action::DropSetNext { task, variable_id } => {
                self.executor.execute_drop_set_next(*task, variable_id).await?;
                Ok((ControlFlow::Next, None))
            }
            Action::LaunchApp {
                pkg_name,
                activity_name,
            } => {
                if pkg_name.trim().is_empty() || activity_name.trim().is_empty() {
                    return Err(ScriptExecutor::execute_error(
                        "action.launchApp",
                        "LaunchApp 需要同时提供 pkg_name 和 activity_name".to_string(),
                    ));
                }
                self.execute_simple_device_operation(
                    "action.launchApp",
                    "启动应用",
                    DeviceOperation::LaunchApp {
                        pkg_name: pkg_name.clone(),
                        activity_name: activity_name.clone(),
                    },
                    PolicyActionKind::StartApp,
                    DEVICE_EXTERNAL_TIMEOUT_MS,
                )
                .await
            }
            Action::StopApp { pkg_name } => self
                .execute_simple_device_operation(
                    "action.stopApp",
                    "停止应用",
                    DeviceOperation::StopApp {
                        pkg_name: pkg_name.clone(),
                    },
                    PolicyActionKind::StopApp,
                    DEVICE_EXTERNAL_TIMEOUT_MS,
                )
                .await,
        }
    }

    async fn execute_long_click(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        let plan = self
            .executor
            .plan_long_click_action(mode, offset_x, offset_y)
            .await?;
        self.executor
            .execute_planned_device_action("action.longClick", "长按", plan)
            .await
    }

    async fn execute_click(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        let plan = self.executor.plan_click_action(mode, offset_x, offset_y).await?;
        self.executor
            .execute_planned_device_action("action.click", "设备点击", plan)
            .await
    }

    async fn execute_swipe(
        &mut self,
        mode: &SwipeMode,
        duration: u64,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        let plan = self.executor.plan_swipe_action(mode, duration).await?;
        self.executor
            .execute_planned_device_action("action.swipe", "设备滑动", plan)
            .await
    }
}
