impl ScriptExecutor {
    fn policy_set_candidate_cache_key(target: &[PolicySetId]) -> String {
        target
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("|")
    }

    async fn resolve_cached_policy_set_candidates(
        &self,
        bundle: &PolicyBundle,
        target: &[PolicySetId],
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let cache_key = Self::policy_set_candidate_cache_key(target);
        {
            let ctx = self.runtime_ctx.read().await;
            if ctx.execution.policy_set_candidate_cache_ready {
                if let Some(cached) = ctx.execution.policy_set_candidate_cache.get(&cache_key) {
                    Log::debug(&format!(
                        "[ executor ] HandlePolicySet 复用候选缓存: key={}, candidate_count={}",
                        cache_key,
                        cached.len()
                    ));
                    return Ok(cached.clone());
                }
            }
        }

        let candidates = self.resolve_policy_set_candidates(bundle, target).await?;
        let mut ctx = self.runtime_ctx.write().await;
        ctx.execution
            .policy_set_candidate_cache
            .insert(cache_key, candidates.clone());
        ctx.execution.policy_set_candidate_cache_ready = true;
        Ok(candidates)
    }

    async fn execute_handle_policy_set(
        &mut self,
        target: &[PolicySetId],
        det_input_var: &str,
        ocr_input_var: &str,
        out_var: &str,
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence(
                "flow.handlePolicySet",
                format!("HandlePolicySet 开始匹配，target_count={}", target.len()),
            )
            .await?
        {
            return Ok(timeout_flow);
        }
        self.activate_runtime_results_context(
            "flow.handlePolicySet",
            det_input_var,
            ocr_input_var,
        )
            .await?;
        let bundle = self.load_policy_bundle("flow.handlePolicySet").await?;
        let candidates = self
            .resolve_cached_policy_set_candidates(&bundle, target)
            .await?;
        Log::debug(&format!(
            "[ executor ] HandlePolicySet 候选展开完成: target_count={}, candidate_count={}, out_var={}",
            target.len(),
            candidates.len(),
            out_var
        ));
        self.execute_policy_candidates("flow.handlePolicySet", candidates, out_var)
            .await
    }

    async fn debug_execute_policy_candidates(
        &mut self,
        debug_target_label: &str,
        candidates: Vec<PolicyCandidate>,
    ) -> ExecuteResult<PolicyExecutionResult> {
        self.capture_policy_debug_observation(debug_target_label).await?;
        self.log_policy_debug_observation(debug_target_label).await?;
        Log::info(&format!(
            "[ policy_debug ] {}候选数: {}",
            debug_target_label,
            candidates.len()
        ));
        for (index, candidate) in candidates.iter().enumerate() {
            Log::info(&format!(
                "[ policy_debug ] 候选[{}] set={} group={} policy={}({})",
                index,
                candidate.policy_set_name.as_deref().unwrap_or("<none>"),
                candidate.policy_group_name.as_deref().unwrap_or("<none>"),
                candidate.policy.data.0.name,
                candidate.policy.id
            ));
        }

        self.execute_policy_candidates("debug.policy", candidates, "runtime.policyDebugResult")
            .await?;
        let value = self.read_runtime_var("runtime.policyDebugResult").await.ok_or_else(|| {
            Self::execute_error(
                "debug.policy",
                "策略调试未产出 runtime.policyDebugResult".to_string(),
            )
        })?;
        let result = Self::deserialize_dynamic_value::<PolicyExecutionResult>(&value).map_err(
            |error| {
                Self::execute_error(
                    "debug.policy",
                    format!("解析策略调试结果失败: {}", error),
                )
            },
        )?;
        self.log_policy_debug_result(&result);
        Ok(result)
    }

    async fn execute_handle_policy(
        &mut self,
        target: &[PolicyId],
        input_var: &str,
        out_var: &str,
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence(
                "flow.handlePolicy",
                format!("HandlePolicy 开始匹配，target_count={}", target.len()),
            )
            .await?
        {
            return Ok(timeout_flow);
        }
        self.activate_image_var("flow.handlePolicy", input_var).await?;
        let bundle = self.load_policy_bundle("flow.handlePolicy").await?;
        let candidates = Self::resolve_policy_candidates(&bundle, target)?;
        Log::debug(&format!(
            "[ executor ] HandlePolicy 候选展开完成: target_count={}, candidate_count={}, out_var={}",
            target.len(),
            candidates.len(),
            out_var
        ));
        self.execute_policy_candidates("flow.handlePolicy", candidates, out_var)
            .await
    }

    async fn execute_policy_candidates(
        &mut self,
        step_type: &str,
        candidates: Vec<PolicyCandidate>,
        out_var: &str,
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence(
                format!("{}.scan", step_type),
                format!("{} 扫描候选策略，candidate_count={}", step_type, candidates.len()),
            )
            .await?
        {
            return Ok(timeout_flow);
        }

        let total_candidates = candidates.len();
        let mut result = PolicyExecutionResult {
            matched: false,
            policy_set_id: None,
            policy_group_id: None,
            policy_id: None,
            rounds: Vec::new(),
        };

        for (index, candidate) in candidates.into_iter().enumerate() {
            if let Some(timeout_flow) = self
                .record_progress_evidence(
                    format!("{}.candidate", step_type),
                    format!(
                        "{} 处理候选策略 {}/{}",
                        step_type,
                        index + 1,
                        total_candidates
                    ),
                )
                .await?
            {
                return Ok(timeout_flow);
            }

            let skipped = self.policy_is_skipped(candidate.policy.id).await;
            let mut round = PolicyExecutionRound {
                matched: false,
                policy_set_id: candidate.policy_set_id,
                policy_group_id: candidate.policy_group_id,
                policy_id: Some(candidate.policy.id),
                page_fingerprints: Vec::new(),
                action_signatures: Vec::new(),
                actions: Vec::new(),
            };

            if skipped {
                result.rounds.push(round);
                continue;
            }

            let policy_name = candidate.policy.data.0.name.clone();
            Log::debug(&format!(
                "[ policy_debug ] 命中策略: 策略集={}，策略组={}，策略={}",
                candidate.policy_set_name.as_deref().unwrap_or("<none>"),
                candidate.policy_group_name.as_deref().unwrap_or("<none>"),
                policy_name
            ));
            let before_action = candidate.policy.data.0.before_action.clone();
            let after_action = candidate.policy.data.0.after_action.clone();
            self.begin_active_policy_round_trace(
                candidate.policy.id,
                policy_name.clone(),
                candidate.policy.data.0.cur_pos,
            )
            .await;
            let execute_result = async {
                self.execute_policy_steps(
                    step_type,
                    &policy_name,
                    "before_action",
                    before_action.as_slice(),
                )
                .await?;
                let matched = self
                    .evaluate_policy_match(step_type, &candidate.policy.data.0.cond)
                    .await?;
                if !matched {
                    return Ok::<bool, crate::infrastructure::scripts::script_error::ScriptError>(false);
                }
                self.execute_policy_steps(
                    step_type,
                    &policy_name,
                    "after_action",
                    after_action.as_slice(),
                )
                .await?;
                Ok::<bool, crate::infrastructure::scripts::script_error::ScriptError>(true)
            }
            .await;
            let trace = self.take_active_policy_round_trace();
            let matched = execute_result?;

            round.page_fingerprints = trace.page_fingerprints;
            round.action_signatures = trace.action_signatures;
            round.actions = trace.actions;
            if !matched {
                result.rounds.push(round);
                continue;
            }

            self.mark_policy_succeeded(candidate.policy.id).await;
            round.matched = true;
            result.matched = true;
            result.policy_set_id = candidate.policy_set_id;
            result.policy_group_id = candidate.policy_group_id;
            result.policy_id = Some(candidate.policy.id);
            result.rounds.push(round);

            self.set_runtime_var(
                out_var,
                Self::results_to_dynamic(step_type, "策略执行", &result)?,
            )
            .await?;
            return Ok(ControlFlow::Next);
        }

        self.set_runtime_var(
            out_var,
            Self::results_to_dynamic(step_type, "策略执行", &result)?,
        )
        .await?;
        Ok(ControlFlow::Next)
    }

    async fn evaluate_policy_match(
        &self,
        step_type: &str,
        rule: &SearchRule,
    ) -> ExecuteResult<bool> {
        let ctx = self.runtime_ctx.read().await;
        let snapshot = ctx.observation.last_snapshot.as_ref().ok_or_else(|| {
            Self::execute_error(
                step_type,
                "当前没有可用的视觉快照，无法评估策略命中条件".to_string(),
            )
        })?;
        let searcher = OcrSearcher::new(std::slice::from_ref(rule));
        let hits = searcher.search(snapshot);
        Ok(rule.evaluate(&hits, &snapshot.det_items))
    }

    async fn execute_policy_steps(
        &mut self,
        step_type: &str,
        policy_name: &str,
        phase: &str,
        steps: &[Step],
    ) -> ExecuteResult<()> {
        match self.execute(steps).await? {
            ControlFlow::Next => Ok(()),
            flow => Err(Self::execute_error(
                step_type,
                format!(
                    "策略[{}]的{}返回了不支持的控制流: {:?}",
                    policy_name, phase, flow
                ),
            )),
        }
    }

    async fn policy_is_skipped(&self, policy_id: PolicyId) -> bool {
        let ctx = self.runtime_ctx.read().await;
        ctx.execution
            .policy_states
            .get(&policy_id)
            .map(|state| state.skip_flag)
            .unwrap_or(false)
    }

    async fn mark_policy_succeeded(&self, policy_id: PolicyId) {
        let mut ctx = self.runtime_ctx.write().await;
        let state = ctx.execution.policy_states.entry(policy_id).or_default();
        state.done_flag = true;
        state.exec_cur = state.exec_cur.saturating_add(1);
    }

}
