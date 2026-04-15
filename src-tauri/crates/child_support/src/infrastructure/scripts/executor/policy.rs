impl ScriptExecutor {
    async fn execute_handle_policy_set(
        &mut self,
        target: &[PolicySetId],
        input_var: &str,
        out_var: &str,
    ) -> ExecuteResult<ControlFlow> {
        self.activate_image_var("flow.handlePolicySet", input_var)
            .await?;
        let bundle = self.load_policy_bundle("flow.handlePolicySet").await?;
        let candidates = Self::resolve_policy_set_candidates(&bundle, target)?;
        self.execute_policy_candidates("flow.handlePolicySet", candidates, out_var)
            .await
    }

    async fn execute_handle_policy(
        &mut self,
        target: &[PolicyId],
        input_var: &str,
        out_var: &str,
    ) -> ExecuteResult<ControlFlow> {
        self.activate_image_var("flow.handlePolicy", input_var).await?;
        let bundle = self.load_policy_bundle("flow.handlePolicy").await?;
        let candidates = Self::resolve_policy_candidates(&bundle, target)?;
        self.execute_policy_candidates("flow.handlePolicy", candidates, out_var)
            .await
    }

    async fn execute_policy_candidates(
        &mut self,
        step_type: &str,
        candidates: Vec<PolicyCandidate>,
        out_var: &str,
    ) -> ExecuteResult<ControlFlow> {
        let match_flags = {
            let ctx = self.runtime_ctx.read().await;
            ctx.observation
                .last_snapshot
                .as_ref()
                .map(|snapshot| Self::build_policy_match_flags(snapshot, &candidates))
        };
        let Some(match_flags) = match_flags else {
            let result = PolicyExecutionResult {
                matched: false,
                policy_set_id: None,
                policy_group_id: None,
                policy_id: None,
                rounds: Vec::new(),
            };
            self.set_runtime_var(
                out_var,
                Self::results_to_dynamic(step_type, "策略执行", &result)?,
            )
            .await?;
            return Ok(ControlFlow::Next);
        };

        let mut result = PolicyExecutionResult {
            matched: false,
            policy_set_id: None,
            policy_group_id: None,
            policy_id: None,
            rounds: Vec::new(),
        };

        for (candidate, matched) in candidates.into_iter().zip(match_flags.into_iter()) {
            let skipped = self.policy_is_skipped(candidate.policy.id).await;
            let mut round = PolicyExecutionRound {
                matched: matched && !skipped,
                policy_set_id: candidate.policy_set_id,
                policy_group_id: candidate.policy_group_id,
                policy_id: Some(candidate.policy.id),
                page_fingerprints: Vec::new(),
                action_signatures: Vec::new(),
                actions: Vec::new(),
            };

            if !matched || skipped {
                result.rounds.push(round);
                continue;
            }

            let policy_name = candidate.policy.data.0.name.clone();
            let before_action = candidate.policy.data.0.before_action.clone();
            let after_action = candidate.policy.data.0.after_action.clone();
            self.begin_active_policy_round_trace().await;
            let execute_result = async {
                self.execute_policy_steps(
                    step_type,
                    &policy_name,
                    "before_action",
                    before_action.as_slice(),
                )
                .await?;
                self.execute_policy_steps(
                    step_type,
                    &policy_name,
                    "after_action",
                    after_action.as_slice(),
                )
                .await?;
                Ok::<(), crate::infrastructure::scripts::script_error::ScriptError>(())
            }
            .await;
            let trace = self.take_active_policy_round_trace();
            execute_result?;

            self.mark_policy_succeeded(candidate.policy.id).await;
            round.matched = true;
            round.page_fingerprints = trace.page_fingerprints;
            round.action_signatures = trace.action_signatures;
            round.actions = trace.actions;
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

    fn build_policy_match_flags(
        snapshot: &VisionSnapshot,
        candidates: &[PolicyCandidate],
    ) -> Vec<bool> {
        if candidates.is_empty() {
            return Vec::new();
        }

        let rules: Vec<_> = candidates
            .iter()
            .map(|candidate| candidate.policy.data.0.cond.clone())
            .collect();
        let searcher = OcrSearcher::new(&rules);
        let hits = searcher.search(snapshot);

        candidates
            .iter()
            .map(|candidate| {
                candidate
                    .policy
                    .data
                    .0
                    .cond
                    .evaluate(&hits, &snapshot.det_items)
            })
            .collect()
    }

    async fn load_policy_bundle(&self, step_type: &str) -> ExecuteResult<PolicyBundle> {
        let script_id = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.script_id
        };
        let snapshot = get_script_bundle_snapshot(script_id).await.ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!("当前 session 中不存在脚本[{}]的 bundle", script_id),
            )
        })?;

        Ok(PolicyBundle {
            policies: Self::parse_bundle_json(step_type, "policies_json", &snapshot.policies_json)?,
            policy_groups: Self::parse_bundle_json(
                step_type,
                "policy_groups_json",
                &snapshot.policy_groups_json,
            )?,
            policy_sets: Self::parse_bundle_json(
                step_type,
                "policy_sets_json",
                &snapshot.policy_sets_json,
            )?,
            group_policies: Self::parse_bundle_json(
                step_type,
                "group_policies_json",
                &snapshot.group_policies_json,
            )?,
            set_groups: Self::parse_bundle_json(
                step_type,
                "set_groups_json",
                &snapshot.set_groups_json,
            )?,
        })
    }

    fn parse_bundle_json<T>(step_type: &str, field: &str, json: &str) -> ExecuteResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(json).map_err(|error| {
            Self::execute_error(
                step_type,
                format!("解析 bundle 字段 {} 失败: {}", field, error),
            )
        })
    }

    fn resolve_policy_set_candidates(
        bundle: &PolicyBundle,
        target: &[PolicySetId],
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let policy_map: HashMap<PolicyId, PolicyTable> = bundle
            .policies
            .iter()
            .cloned()
            .map(|policy| (policy.id, policy))
            .collect();
        let group_map: HashMap<PolicyGroupId, PolicyGroupTable> = bundle
            .policy_groups
            .iter()
            .cloned()
            .map(|group| (group.id, group))
            .collect();
        let set_map: HashMap<PolicySetId, PolicySetTable> = bundle
            .policy_sets
            .iter()
            .cloned()
            .map(|set| (set.id, set))
            .collect();

        let mut candidates = Vec::new();
        for set_id in target {
            if !set_map.contains_key(set_id) {
                return Err(Self::execute_error(
                    "flow.handlePolicySet",
                    format!("目标策略集[{}]不存在", set_id),
                ));
            }

            let mut group_relations: Vec<_> = bundle
                .set_groups
                .iter()
                .filter(|relation| relation.set_id == *set_id)
                .cloned()
                .collect();
            group_relations.sort_by_key(|relation| relation.order_index);

            for group_relation in group_relations {
                if !group_map.contains_key(&group_relation.group_id) {
                    Log::warn(&format!(
                        "[ executor ] 策略集[{}]引用的策略组[{}]不存在，已跳过",
                        set_id, group_relation.group_id
                    ));
                    continue;
                }

                let mut policy_relations: Vec<_> = bundle
                    .group_policies
                    .iter()
                    .filter(|relation| relation.group_id == group_relation.group_id)
                    .cloned()
                    .collect();
                policy_relations.sort_by_key(|relation| relation.order_index);

                for policy_relation in policy_relations {
                    let Some(policy) = policy_map.get(&policy_relation.policy_id) else {
                        Log::warn(&format!(
                            "[ executor ] 策略组[{}]引用的策略[{}]不存在，已跳过",
                            group_relation.group_id, policy_relation.policy_id
                        ));
                        continue;
                    };

                    candidates.push(PolicyCandidate {
                        policy_set_id: Some(*set_id),
                        policy_group_id: Some(group_relation.group_id),
                        policy: policy.clone(),
                    });
                }
            }
        }

        Ok(candidates)
    }

    fn resolve_policy_candidates(
        bundle: &PolicyBundle,
        target: &[PolicyId],
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let policy_map: HashMap<PolicyId, PolicyTable> = bundle
            .policies
            .iter()
            .cloned()
            .map(|policy| (policy.id, policy))
            .collect();

        let mut candidates = Vec::new();
        for policy_id in target {
            let Some(policy) = policy_map.get(policy_id) else {
                return Err(Self::execute_error(
                    "flow.handlePolicy",
                    format!("目标策略[{}]不存在", policy_id),
                ));
            };

            candidates.push(PolicyCandidate {
                policy_set_id: None,
                policy_group_id: None,
                policy: policy.clone(),
            });
        }

        Ok(candidates)
    }

    async fn begin_active_policy_round_trace(&mut self) {
        self.active_policy_round = Some(ActivePolicyRoundTrace::default());
        if let Some(fingerprint) = self.current_page_fingerprint().await {
            self.push_active_policy_page_fingerprint(fingerprint);
        }
    }

    fn take_active_policy_round_trace(&mut self) -> ActivePolicyRoundTrace {
        self.active_policy_round.take().unwrap_or_default()
    }
}
