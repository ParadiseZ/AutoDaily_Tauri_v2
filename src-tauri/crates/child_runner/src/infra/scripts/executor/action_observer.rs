impl ScriptExecutor {
    fn record_action_trace(&mut self, mut action_trace: PolicyActionTrace) {
        if let Some(round) = self.active_policy_round.as_mut() {
            action_trace.action_index = round.actions.len() as u16;
            round.action_signatures.push(action_trace.signature.clone());
            round.actions.push(action_trace);
        }
    }

    fn push_active_policy_page_fingerprint(&mut self, fingerprint: String) {
        let Some(round) = self.active_policy_round.as_mut() else {
            return;
        };

        if round.page_fingerprints.last() != Some(&fingerprint) {
            round.page_fingerprints.push(fingerprint);
        }
    }

    async fn current_page_fingerprint(&self) -> Option<String> {
        let ctx = self.runtime_ctx.read().await;
        ctx.observation
            .last_snapshot
            .as_ref()
            .map(Self::build_page_fingerprint)
    }

    fn should_attach_page_fingerprint(evidence_signature: &str) -> bool {
        evidence_signature.starts_with("vision.")
            || evidence_signature.starts_with("flow.handlePolicy")
            || evidence_signature.starts_with("data.relativeFilter")
            || evidence_signature.starts_with("data.colorCompare")
            || evidence_signature.starts_with("debug.policy")
    }

    async fn resolve_evidence_page_fingerprint(&self, evidence_signature: &str) -> Option<String> {
        if Self::should_attach_page_fingerprint(evidence_signature) {
            self.current_page_fingerprint().await
        } else {
            None
        }
    }

    async fn observe_refresh_hook(
        &self,
        action: &Action,
        action_trace: Option<&PolicyActionTrace>,
    ) {
        let Some(action_trace) = action_trace else {
            return;
        };

        let (script_name, task_name, step_name) = self.current_execution_names().await;
        let page_fingerprint = self.current_page_fingerprint().await;
        Log::debug(&format!(
            "[ executor ] 动作后 observe hook: script={}, task={}, step={}, action={:?}, signature={}, page_fingerprint={}",
            script_name,
            task_name,
            step_name,
            action,
            action_trace.signature,
            page_fingerprint.unwrap_or_else(|| "<none>".to_string())
        ));
    }

    async fn evaluate_action_progress_timeout(
        &mut self,
        runtime_policy: &runner_protocol::message::RuntimeExecutionPolicy,
        action: &Action,
        action_trace: Option<&PolicyActionTrace>,
    ) -> ExecuteResult<Option<ControlFlow>> {
        let Some(action_trace) = action_trace else {
            return Ok(None);
        };

        self.evaluate_progress_probe(
            runtime_policy,
            None,
            action_trace.signature.clone(),
            format!("动作后观测点: {:?}", action),
        )
        .await
    }

    async fn evaluate_progress_probe(
        &mut self,
        runtime_policy: &runner_protocol::message::RuntimeExecutionPolicy,
        page_fingerprint: Option<String>,
        evidence_signature: String,
        detail: String,
    ) -> ExecuteResult<Option<ControlFlow>> {
        let (_execution_id, _assignment_id, _script_id, task_id, step_id) =
            self.current_execution_locator().await;
        let now = Instant::now();
        let previous_probe = self.last_progress_probe.clone();
        let same_probe_chain = previous_probe
            .as_ref()
            .filter(|previous| previous.page_fingerprint == page_fingerprint)
            .filter(|previous| previous.evidence_signature == evidence_signature)
            .filter(|previous| previous.task_id == task_id)
            .filter(|previous| previous.step_id == step_id);
        let stagnant_since = same_probe_chain
            .map(|previous| previous.stagnant_since)
            .unwrap_or(now);
        let already_notified = same_probe_chain
            .map(|previous| previous.notified)
            .unwrap_or(false);

        let mut probe = ProgressProbe {
            page_fingerprint: page_fingerprint.clone(),
            evidence_signature: evidence_signature.clone(),
            task_id,
            step_id,
            stagnant_since,
            notified: already_notified,
        };

        if !runtime_policy.progress_timeout_enabled || runtime_policy.progress_timeout_ms == 0 {
            self.last_progress_probe = Some(probe);
            return Ok(None);
        }

        let timeout_elapsed = now.duration_since(probe.stagnant_since)
            >= Duration::from_millis(runtime_policy.progress_timeout_ms);

        if !timeout_elapsed || already_notified {
            self.last_progress_probe = Some(probe);
            return Ok(None);
        }
        probe.notified = true;
        self.last_progress_probe = Some(probe);

        let message = format!(
            "检测到长时间无进展: task={:?}, step={:?}, evidence={}, page={}, stagnantFor={}ms, threshold={}ms, detail={}",
            task_id,
            step_id,
            evidence_signature,
            page_fingerprint
                .clone()
                .unwrap_or_else(|| "<none>".to_string()),
            now.duration_since(stagnant_since).as_millis(),
            runtime_policy.progress_timeout_ms,
            detail
        );
        self.emit_timeout_signals(
            runtime_policy.timeout_action.clone(),
            runtime_policy.timeout_notify_channels.clone(),
            page_fingerprint,
            Some(evidence_signature),
            message.clone(),
        )
        .await;
        let timeout_result = self.handle_timeout_action(runtime_policy, message).await;
        self.reset_progress_probe();
        timeout_result
    }

    async fn record_progress_evidence(
        &mut self,
        evidence_signature: impl Into<String>,
        detail: impl Into<String>,
    ) -> ExecuteResult<Option<ControlFlow>> {
        let Some(runtime_policy) = get_runtime_execution_policy().await else {
            return Ok(None);
        };
        let evidence_signature = evidence_signature.into();
        let page_fingerprint = self
            .resolve_evidence_page_fingerprint(&evidence_signature)
            .await;

        self.evaluate_progress_probe(
            &runtime_policy,
            page_fingerprint,
            evidence_signature,
            detail.into(),
        )
        .await
    }

    async fn sleep_with_progress_timeout(
        &mut self,
        ms: u64,
        evidence_signature: impl Into<String>,
        detail: impl Into<String>,
    ) -> ExecuteResult<Option<ControlFlow>> {
        let evidence_signature = evidence_signature.into();
        let detail = detail.into();

        if let Some(flow) = self
            .record_progress_evidence(evidence_signature.clone(), detail.clone())
            .await?
        {
            return Ok(Some(flow));
        }

        let mut remaining = Duration::from_millis(ms);
        while !remaining.is_zero() {
            if let Some(flow) = Self::stop_requested_flow() {
                return Ok(Some(flow));
            }
            let slice = remaining.min(Duration::from_millis(WAIT_TIMEOUT_CHECK_SLICE_MS));
            tokio::time::sleep(slice).await;
            remaining = remaining.saturating_sub(slice);

            if let Some(flow) = self
                .record_progress_evidence(evidence_signature.clone(), detail.clone())
                .await?
            {
                return Ok(Some(flow));
            }
        }

        #[cfg(feature = "testkit")]
        if let Some(test_hooks) = self.test_hooks.as_ref() {
            test_hooks
                .record_operation(&domain_device::DeviceOperation::Delay(ms), None)
                .await;
        }
        Ok(None)
    }

    async fn handle_timeout_action(
        &mut self,
        runtime_policy: &runner_protocol::message::RuntimeExecutionPolicy,
        message: String,
    ) -> ExecuteResult<Option<ControlFlow>> {
        match runtime_policy.timeout_action {
            TimeoutAction::SkipCurrentTask => {
                self.mark_current_task_skipped().await;
                Ok(Some(ControlFlow::Return))
            }
            TimeoutAction::RunRecoveryTask => {
                let recovery_task_id = self.recovery_task_id().await.ok_or_else(|| {
                    Self::execute_error(
                        "action.timeout",
                        "当前脚本未配置 recovery_task_id，无法执行 RunRecoveryTask".to_string(),
                    )
                })?;
                Ok(Some(ControlFlow::Link(recovery_task_id)))
            }
            TimeoutAction::StopExecution => {
                crate::infra::context::runtime_control::request_stop_execution();
                crate::infra::context::runtime_control::set_running_status(
                    crate::infra::context::runtime_control::RunningStatus::Stopping,
                );
                emit_progress_event(
                    RuntimeProgressPhase::Stopping,
                    None,
                    None,
                    None,
                    None,
                    Some(message.clone()),
                );
                emit_lifecycle_event(RuntimeLifecyclePhase::Stopping, Some(message.clone()));
                Err(Self::execute_error("action.timeout", message))
            }
        }
    }

    async fn emit_timeout_signals(
        &self,
        timeout_action: TimeoutAction,
        notify_channels: Vec<domain_device::TimeoutNotifyChannel>,
        page_fingerprint: Option<String>,
        action_signature: Option<String>,
        message: String,
    ) {
        let (_execution_id, assignment_id, script_id, task_id, step_id) =
            self.current_execution_locator().await;
        let timeout_message = format!(
            "[timeout] action={:?}; page={}; signature={}; {}",
            timeout_action,
            page_fingerprint
                .clone()
                .unwrap_or_else(|| "<none>".to_string()),
            action_signature
                .clone()
                .unwrap_or_else(|| "<none>".to_string()),
            message
        );
        emit_progress_event(
            RuntimeProgressPhase::Executing,
            assignment_id,
            script_id,
            task_id,
            step_id,
            Some(timeout_message),
        );

        if notify_channels.iter().any(|channel| {
            matches!(
                channel,
                domain_device::TimeoutNotifyChannel::SystemNotification
            )
        }) {
            emit_progress_event(
                RuntimeProgressPhase::Executing,
                assignment_id,
                script_id,
                task_id,
                step_id,
                Some(format!("[timeout_notify] {}", message)),
            );
        }

        if notify_channels.iter().any(|channel| {
            matches!(
                channel,
                domain_device::TimeoutNotifyChannel::Email
            )
        }) {
            emit_progress_event(
                RuntimeProgressPhase::Executing,
                assignment_id,
                script_id,
                task_id,
                step_id,
                Some(format!("[timeout_email] {}", message)),
            );
        }
    }

    async fn current_execution_locator(
        &self,
    ) -> (
        Option<ExecutionId>,
        Option<AssignmentId>,
        Option<ScriptId>,
        Option<TaskId>,
        Option<StepId>,
    ) {
        let ctx = self.runtime_ctx.read().await;
        (
            ctx.execution.current_execution_id,
            ctx.execution.current_assignment_id,
            Some(ctx.execution.script_id),
            ctx.execution.current_task.as_ref().map(|task| task.id),
            ctx.execution.current_step_id,
        )
    }

    async fn current_execution_names(&self) -> (String, String, String) {
        let ctx = self.runtime_ctx.read().await;
        let script_name = ctx
            .execution
            .script_info
            .as_ref()
            .map(|script| script.name.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("<none>")
            .to_string();
        let task_name = ctx
            .execution
            .current_task
            .as_ref()
            .map(|task| task.name.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("<none>")
            .to_string();
        let step_name = ctx
            .execution
            .current_step_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("<none>")
            .to_string();
        (script_name, task_name, step_name)
    }

    async fn mark_current_task_skipped(&self) {
        let Some(task_id) = ({
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.current_task.as_ref().map(|task| task.id)
        }) else {
            return;
        };

        let mut ctx = self.runtime_ctx.write().await;
        let state = ctx.execution.task_states.entry(task_id).or_default();
        state.skip_flag = true;
    }

    async fn recovery_task_id(&self) -> Option<TaskId> {
        let ctx = self.runtime_ctx.read().await;
        ctx.execution
            .script_info
            .as_ref()
            .and_then(|info| info.runtime_settings.recovery_task_id)
    }

    fn reset_progress_probe(&mut self) {
        self.last_progress_probe = None;
    }
}
