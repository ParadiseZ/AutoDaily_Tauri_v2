impl ScriptExecutor {
    fn evaluate_condition<'a>(
        &'a mut self,
        condition: &'a ConditionNode,
    ) -> Pin<Box<dyn Future<Output = ExecuteResult<bool>> + 'a>> {
        Box::pin(async move {
            match condition {
                ConditionNode::RawExpr { expr } => self.eval_bool(expr, "condition.rawExpr"),
                ConditionNode::Group { op, items } => match op {
                    crate::domain::vision::ocr_search::LogicOp::And => {
                        for item in items {
                            if !self.evaluate_condition(item).await? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    crate::domain::vision::ocr_search::LogicOp::Or => {
                        for item in items {
                            if self.evaluate_condition(item).await? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }
                    crate::domain::vision::ocr_search::LogicOp::Not => {
                        if let Some(first) = items.first() {
                            Ok(!self.evaluate_condition(first).await?)
                        } else {
                            Ok(true)
                        }
                    }
                },
                ConditionNode::VarCompare {
                    var_name,
                    op,
                    value,
                } => {
                    let Some(lhs) = self.read_runtime_var(var_name).await else {
                        return Ok(false);
                    };
                    let rhs = Self::var_value_to_dynamic(value);
                    Ok(Self::compare_dynamic(&lhs, op, &rhs))
                }
                ConditionNode::VisionCountCompare {
                    input_var,
                    target_value,
                    op,
                    expected_count,
                } => {
                    self.match_vision_count_compare(
                        "condition.visionCountCompare",
                        input_var,
                        target_value.as_deref(),
                        op,
                        *expected_count,
                    )
                    .await
                }
                ConditionNode::TaskStatus { a } => self.match_state_status(a).await,
                ConditionNode::CurrentTaskIn { op, items, targets } => {
                    Ok(self.match_current_task_rule(op, items, targets).await)
                }
                ConditionNode::ExecNumCompare { target, op } => {
                    self.match_exec_num_compare(target, op).await
                }
                ConditionNode::ColorCompare { .. } => Err(Self::execute_error(
                    "condition.colorCompare",
                    "颜色比较请通过数据处理 ColorCompare 筛选结果集后再判断".to_string(),
                )),
                ConditionNode::PolicySetResult {
                    result_var,
                    field,
                    op,
                    value_bool,
                    value_id,
                } => self
                    .match_policy_set_result(
                        result_var,
                        field,
                        op,
                        *value_bool,
                        value_id.as_str(),
                    )
                    .await,
            }
        })
    }

    async fn match_policy_set_result(
        &self,
        result_var: &str,
        field: &PolicySetResultField,
        op: &PolicySetResultCompareOp,
        value_bool: bool,
        value_id: &str,
    ) -> ExecuteResult<bool> {
        let Some(value) = self.read_runtime_var(result_var).await else {
            return Ok(false);
        };
        let result = Self::deserialize_dynamic_value::<PolicyExecutionResult>(&value).map_err(
            |error| {
                Self::execute_error(
                    "condition.policySetResult",
                    format!("变量[{}]不是兼容的策略执行结果: {}", result_var, error),
                )
            },
        )?;

        Ok(match field {
            PolicySetResultField::Matched => Self::compare_bool(result.matched, op, value_bool),
            PolicySetResultField::PolicySetId => {
                Self::compare_optional_id(result.policy_set_id, op, value_id)
            }
            PolicySetResultField::PolicyGroupId => {
                Self::compare_optional_id(result.policy_group_id, op, value_id)
            }
            PolicySetResultField::PolicyId => {
                Self::compare_optional_id(result.policy_id, op, value_id)
            }
        })
    }

    async fn set_state_value(
        &mut self,
        target: &StateTarget,
        status: &StateStatus,
    ) -> ExecuteResult<()> {
        let mut ctx = self.runtime_ctx.write().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx.execution.task_states.entry(*id).or_default();
                match status {
                    StateStatus::Enabled { value } => state.enabled_flag = *value,
                    StateStatus::Skip { value } => state.skip_flag = *value,
                    StateStatus::Done { value } => state.done_flag = *value,
                }
            }
            StateTarget::Policy { id } => {
                let state = ctx.execution.policy_states.entry(*id).or_default();
                match status {
                    StateStatus::Enabled { .. } => {
                        return Err(Self::execute_error(
                            "taskControl.setState",
                            format!("策略[{}]不支持 enabled 状态", id),
                        ));
                    }
                    StateStatus::Skip { value } => state.skip_flag = *value,
                    StateStatus::Done { value } => state.done_flag = *value,
                }
            }
        }
        Ok(())
    }

    async fn match_state_status(&mut self, task_control: &TaskControl) -> ExecuteResult<bool> {
        let (target, targets, status) = match task_control {
            TaskControl::SetState {
                target,
                targets,
                status,
            } => (target, targets, status),
        };

        if targets.is_empty() {
            return Ok(self.match_state_value(target, status).await);
        }

        for target in targets {
            if !self.match_state_value(target, status).await {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn match_current_task_rule(
        &self,
        op: &crate::domain::vision::ocr_search::LogicOp,
        items: &[CurrentTaskRule],
        legacy_targets: &[TaskId],
    ) -> bool {
        let current_task_id = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.current_task.as_ref().map(|task| task.id)
        };

        if !items.is_empty() {
            return Self::evaluate_current_task_rule_items(current_task_id, op, items);
        }

        current_task_id.is_some_and(|task_id| legacy_targets.contains(&task_id))
    }

    fn evaluate_current_task_rule_items(
        current_task_id: Option<TaskId>,
        op: &crate::domain::vision::ocr_search::LogicOp,
        items: &[CurrentTaskRule],
    ) -> bool {
        match op {
            crate::domain::vision::ocr_search::LogicOp::And => items
                .iter()
                .all(|item| Self::evaluate_current_task_rule(current_task_id, item)),
            crate::domain::vision::ocr_search::LogicOp::Or => items
                .iter()
                .any(|item| Self::evaluate_current_task_rule(current_task_id, item)),
            crate::domain::vision::ocr_search::LogicOp::Not => items
                .first()
                .is_none_or(|item| !Self::evaluate_current_task_rule(current_task_id, item)),
        }
    }

    fn evaluate_current_task_rule(
        current_task_id: Option<TaskId>,
        rule: &CurrentTaskRule,
    ) -> bool {
        match rule {
            CurrentTaskRule::Task { target } => current_task_id.is_some_and(|task_id| task_id == *target),
            CurrentTaskRule::Group { op, items } => {
                Self::evaluate_current_task_rule_items(current_task_id, op, items)
            }
        }
    }

    async fn match_state_value(&self, target: &StateTarget, status: &StateStatus) -> bool {
        let ctx = self.runtime_ctx.read().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx
                    .execution
                    .task_states
                    .get(id)
                    .cloned()
                    .unwrap_or_else(TaskState::default);
                match status {
                    StateStatus::Enabled { value } => state.enabled_flag == *value,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                }
            }
            StateTarget::Policy { id } => {
                let state = ctx
                    .execution
                    .policy_states
                    .get(id)
                    .cloned()
                    .unwrap_or_default();
                match status {
                    StateStatus::Enabled { .. } => false,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                }
            }
        }
    }

    async fn bind_policy_set(
        &self,
        source: PolicySetId,
        target: PolicySetId,
        top: bool,
        reverse: bool,
    ) {
        let binding = PolicySetBindingOp {
            source: PolicySetBindingSource::PolicySet(source),
            top,
            reverse,
        };
        let mut ctx = self.runtime_ctx.write().await;
        let entry = ctx.execution.policy_set_bindings.entry(target).or_default();
        entry.retain(|item| item.source != binding.source);
        entry.push(binding);
        Log::info(&format!(
            "[ executor ] 绑定：策略集➡️策略集,source_set={}, target_set={}, top={}, reverse={}",
            source, target, top, reverse
        ));
    }

    async fn invalidate_policy_set_candidate_cache(&self) {
        let mut ctx = self.runtime_ctx.write().await;
        ctx.execution.policy_set_candidate_cache_ready = false;
        ctx.execution.policy_set_candidate_cache.clear();
    }

    async fn bind_policy_group_to_set(
        &self,
        source: PolicyGroupId,
        target: PolicySetId,
        top: bool,
        reverse: bool,
    ) {
        let binding = PolicySetBindingOp {
            source: PolicySetBindingSource::PolicyGroup(source),
            top,
            reverse,
        };
        let mut ctx = self.runtime_ctx.write().await;
        let entry = ctx.execution.policy_set_bindings.entry(target).or_default();
        entry.retain(|item| item.source != binding.source);
        entry.push(binding);
        Log::info(&format!(
            "[ executor ] 绑定：策略组➡️策略集,source_group={}, target_set={}, top={}, reverse={}",
            source, target, top, reverse
        ));
    }

    async fn remove_policy_set_from_set(&self, source: PolicySetId, target: PolicySetId) {
        let mut ctx = self.runtime_ctx.write().await;
        if let Some(entry) = ctx.execution.policy_set_bindings.get_mut(&target) {
            entry.retain(|item| item.source != PolicySetBindingSource::PolicySet(source));
            if entry.is_empty() {
                ctx.execution.policy_set_bindings.remove(&target);
            }
        }
        Log::info(&format!(
            "[ executor ] 移除：策略集➡️策略集,source_set={}, target_set={}",
            source, target
        ));
    }

    async fn remove_policy_group_from_set(&self, source: PolicyGroupId, target: PolicySetId) {
        let mut ctx = self.runtime_ctx.write().await;
        if let Some(entry) = ctx.execution.policy_set_bindings.get_mut(&target) {
            entry.retain(|item| item.source != PolicySetBindingSource::PolicyGroup(source));
            if entry.is_empty() {
                ctx.execution.policy_set_bindings.remove(&target);
            }
        }
        Log::info(&format!(
            "[ executor ] 移除：策略组➡️策略集,source_group={}, target_set={}",
            source, target
        ));
    }

    async fn bind_policy_to_group(
        &self,
        source: PolicyId,
        target: PolicyGroupId,
        top: bool,
        reverse: bool,
    ) {
        let binding = PolicyGroupBindingOp {
            source: PolicyGroupBindingSource::Policy(source),
            top,
            reverse,
        };
        let mut ctx = self.runtime_ctx.write().await;
        let entry = ctx.execution.policy_group_bindings.entry(target).or_default();
        entry.retain(|item| item.source != binding.source);
        entry.push(binding);
        Log::info(&format!(
            "[ executor ] 绑定：策略➡️策略组,source_policy={}, target_group={}, top={}, reverse={}",
            source, target, top, reverse
        ));
    }

    async fn unload_policy_from_group(&self, source: PolicyId, target: PolicyGroupId) {
        let mut ctx = self.runtime_ctx.write().await;
        if let Some(entry) = ctx.execution.policy_group_bindings.get_mut(&target) {
            entry.retain(|item| item.source != PolicyGroupBindingSource::Policy(source));
            if entry.is_empty() {
                ctx.execution.policy_group_bindings.remove(&target);
            }
        }
        Log::info(&format!(
            "[ executor ] 卸载：策略➡️策略组,source_policy={}, target_group={}",
            source, target
        ));
    }

    async fn add_policy_group_to_group(
        &self,
        source: PolicyGroupId,
        target: PolicyGroupId,
        top: bool,
        reverse: bool,
    ) {
        let binding = PolicyGroupBindingOp {
            source: PolicyGroupBindingSource::PolicyGroup(source),
            top,
            reverse,
        };
        let mut ctx = self.runtime_ctx.write().await;
        let entry = ctx.execution.policy_group_bindings.entry(target).or_default();
        entry.retain(|item| item.source != binding.source);
        entry.push(binding);
        Log::info(&format!(
            "[ executor ] 追加：策略组➡️策略组,source_group={}, target_group={}, top={}, reverse={}",
            source, target, top, reverse
        ));
    }

    async fn unload_policy_group_from_group(&self, source: PolicyGroupId, target: PolicyGroupId) {
        let mut ctx = self.runtime_ctx.write().await;
        if let Some(entry) = ctx.execution.policy_group_bindings.get_mut(&target) {
            entry.retain(|item| item.source != PolicyGroupBindingSource::PolicyGroup(source));
            if entry.is_empty() {
                ctx.execution.policy_group_bindings.remove(&target);
            }
        }
        Log::info(&format!(
            "[ executor ] 卸载：策略组➡️策略组,source_group={}, target_group={}",
            source, target
        ));
    }

    async fn execute_add_policies_binding(
        &self,
        source: PolicySetId,
        target: PolicySetId,
        top: bool,
        reverse: bool,
    ) -> ExecuteResult<()> {
        if source == target {
            return Err(Self::execute_error(
                "flow.addPolicies",
                format!("源策略集[{}]与目标策略集[{}]不能相同", source, target),
            ));
        }

        let bundle = self.load_policy_bundle("flow.addPolicies").await?;
        if !bundle.policy_sets.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.addPolicies",
                format!("源策略集[{}]不存在", source),
            ));
        }
        if !bundle.policy_sets.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.addPolicies",
                format!("目标策略集[{}]不存在", target),
            ));
        }

        Log::info(&format!(
            "[ executor ] 执行策略集绑定: source_set={}, target_set={}, top={}, reverse={}",
            source, target, top, reverse
        ));
        self.bind_policy_set(source, target, top, reverse).await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_remove_policies_step(
        &self,
        source: PolicySetId,
        target: PolicySetId,
    ) -> ExecuteResult<()> {
        let bundle = self.load_policy_bundle("flow.removePolicies").await?;
        if !bundle.policy_sets.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.removePolicies",
                format!("源策略集[{}]不存在", source),
            ));
        }
        if !bundle.policy_sets.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.removePolicies",
                format!("目标策略集[{}]不存在", target),
            ));
        }

        self.remove_policy_set_from_set(source, target).await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_bind_policy_group_step(
        &self,
        source: PolicyGroupId,
        target: PolicySetId,
        top: bool,
        reverse: bool,
    ) -> ExecuteResult<()> {
        let bundle = self.load_policy_bundle("flow.bindPolicyGroup").await?;
        if !bundle.policy_groups.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.bindPolicyGroup",
                format!("源策略组[{}]不存在", source),
            ));
        }
        if !bundle.policy_sets.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.bindPolicyGroup",
                format!("目标策略集[{}]不存在", target),
            ));
        }

        Log::info(&format!(
            "[ executor ] 执行策略组绑定: source_group={}, target_set={}, top={}, reverse={}",
            source, target, top, reverse
        ));
        self.bind_policy_group_to_set(source, target, top, reverse)
            .await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_remove_policy_group_step(
        &self,
        source: PolicyGroupId,
        target: PolicySetId,
    ) -> ExecuteResult<()> {
        let bundle = self.load_policy_bundle("flow.removePolicyGroup").await?;
        if !bundle.policy_groups.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.removePolicyGroup",
                format!("源策略组[{}]不存在", source),
            ));
        }
        if !bundle.policy_sets.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.removePolicyGroup",
                format!("目标策略集[{}]不存在", target),
            ));
        }

        self.remove_policy_group_from_set(source, target).await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_add_policy_groups_step(
        &self,
        source: PolicyGroupId,
        target: PolicyGroupId,
        top: bool,
        reverse: bool,
    ) -> ExecuteResult<()> {
        if source == target {
            return Err(Self::execute_error(
                "flow.addPolicyGroups",
                format!("源策略组[{}]与目标策略组[{}]不能相同", source, target),
            ));
        }

        let bundle = self.load_policy_bundle("flow.addPolicyGroups").await?;
        if !bundle.policy_groups.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.addPolicyGroups",
                format!("源策略组[{}]不存在", source),
            ));
        }
        if !bundle.policy_groups.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.addPolicyGroups",
                format!("目标策略组[{}]不存在", target),
            ));
        }

        Log::info(&format!(
            "[ executor ] 执行追加策略组: source_group={}, target_group={}, top={}, reverse={}",
            source, target, top, reverse
        ));
        self.add_policy_group_to_group(source, target, top, reverse)
            .await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_unload_policy_group_step(
        &self,
        source: PolicyGroupId,
        target: PolicyGroupId,
    ) -> ExecuteResult<()> {
        let bundle = self.load_policy_bundle("flow.unloadPolicyGroup").await?;
        if !bundle.policy_groups.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.unloadPolicyGroup",
                format!("源策略组[{}]不存在", source),
            ));
        }
        if !bundle.policy_groups.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.unloadPolicyGroup",
                format!("目标策略组[{}]不存在", target),
            ));
        }

        self.unload_policy_group_from_group(source, target).await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_bind_policy_step(
        &self,
        source: PolicyId,
        target: PolicyGroupId,
        top: bool,
        reverse: bool,
    ) -> ExecuteResult<()> {
        let bundle = self.load_policy_bundle("flow.bindPolicy").await?;
        if !bundle.policies.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.bindPolicy",
                format!("源策略[{}]不存在", source),
            ));
        }
        if !bundle.policy_groups.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.bindPolicy",
                format!("目标策略组[{}]不存在", target),
            ));
        }

        Log::info(&format!(
            "[ executor ] 执行绑定策略: source_policy={}, target_group={}, top={}, reverse={}",
            source, target, top, reverse
        ));
        self.bind_policy_to_group(source, target, top, reverse).await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn execute_unload_policy_step(
        &self,
        source: PolicyId,
        target: PolicyGroupId,
    ) -> ExecuteResult<()> {
        let bundle = self.load_policy_bundle("flow.unloadPolicy").await?;
        if !bundle.policies.iter().any(|item| item.id == source) {
            return Err(Self::execute_error(
                "flow.unloadPolicy",
                format!("源策略[{}]不存在", source),
            ));
        }
        if !bundle.policy_groups.iter().any(|item| item.id == target) {
            return Err(Self::execute_error(
                "flow.unloadPolicy",
                format!("目标策略组[{}]不存在", target),
            ));
        }

        self.unload_policy_from_group(source, target).await;
        self.invalidate_policy_set_candidate_cache().await;
        Ok(())
    }

    async fn match_exec_num_compare(
        &self,
        target: &StateTarget,
        op: &CompareOp,
    ) -> ExecuteResult<bool> {
        let exec_cur = self.current_exec_count(target).await;
        let exec_max = self.resolve_exec_limit(target).await?;
        Ok(Self::compare_exec_count(exec_cur, op, exec_max))
    }

    async fn current_exec_count(&self, target: &StateTarget) -> u32 {
        let ctx = self.runtime_ctx.read().await;
        match target {
            StateTarget::Task { id } => ctx
                .execution
                .task_states
                .get(id)
                .map(|state| state.exec_cur)
                .unwrap_or(0),
            StateTarget::Policy { id } => ctx
                .execution
                .policy_states
                .get(id)
                .map(|state| state.exec_cur)
                .unwrap_or(0),
        }
    }

    async fn resolve_exec_limit(&self, target: &StateTarget) -> ExecuteResult<Option<u32>> {
        let script_id = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.script_id
        };
        let snapshot = get_script_bundle_snapshot(script_id).await.ok_or_else(|| {
            Self::execute_error(
                "condition.execNumCompare",
                format!("当前 session 中不存在脚本[{}]的 bundle", script_id),
            )
        })?;

        match target {
            StateTarget::Task { id } => {
                let tasks: Vec<ScriptTaskTable> =
                    Self::parse_bundle_json("condition.execNumCompare", "tasks_json", &snapshot.tasks_json)?;
                let task = tasks.into_iter().find(|task| task.id == *id).ok_or_else(|| {
                    Self::execute_error(
                        "condition.execNumCompare",
                        format!("目标任务[{}]不存在", id),
                    )
                })?;
                Ok((task.exec_max > 0).then_some(task.exec_max))
            }
            StateTarget::Policy { id } => {
                let policies: Vec<PolicyTable> = Self::parse_bundle_json(
                    "condition.execNumCompare",
                    "policies_json",
                    &snapshot.policies_json,
                )?;
                let policy = policies.into_iter().find(|policy| policy.id == *id).ok_or_else(|| {
                    Self::execute_error(
                        "condition.execNumCompare",
                        format!("目标策略[{}]不存在", id),
                    )
                })?;
                Ok((policy.data.0.exec_max > 0).then_some(u32::from(policy.data.0.exec_max)))
            }
        }
    }

    fn compare_exec_count(exec_cur: u32, op: &CompareOp, exec_max: Option<u32>) -> bool {
        match exec_max {
            Some(exec_max) => match op {
                CompareOp::Eq => exec_cur == exec_max,
                CompareOp::Ne => exec_cur != exec_max,
                CompareOp::Lt => exec_cur < exec_max,
                CompareOp::Le => exec_cur <= exec_max,
                CompareOp::Gt => exec_cur > exec_max,
                CompareOp::Ge => exec_cur >= exec_max,
                CompareOp::Contains | CompareOp::NotContains => false,
            },
            None => match op {
                CompareOp::Eq => false,
                CompareOp::Ne => true,
                CompareOp::Lt => true,
                CompareOp::Le => true,
                CompareOp::Gt => false,
                CompareOp::Ge => false,
                CompareOp::Contains | CompareOp::NotContains => false,
            },
        }
    }
}
