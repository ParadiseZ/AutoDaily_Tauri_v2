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
                ConditionNode::TaskStatus { a } => self.match_state_status(a).await,
                ConditionNode::CurrentTaskIn { targets } => Ok(self.current_task_in(targets).await),
                ConditionNode::PolicyCondition { input_var, rule } => {
                    if let Some(input_var) =
                        input_var.as_deref().map(str::trim).filter(|value| !value.is_empty())
                    {
                        if let Err(error) = self
                            .activate_image_var("condition.policyCondition", input_var)
                            .await
                        {
                            Log::debug(&format!(
                                "[ executor ] PolicyCondition 输入图像不可用，按 false 处理: {}",
                                error
                            ));
                            return Ok(false);
                        }
                    }

                    let ctx = self.runtime_ctx.read().await;
                    if let Some(snapshot) = ctx.observation.last_snapshot.as_ref() {
                        Ok(rule.evaluate(snapshot))
                    } else {
                        Log::debug("[ executor ] PolicyCondition 未找到可用视觉快照，按 false 处理");
                        Ok(false)
                    }
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

    async fn current_task_in(&self, targets: &[TaskId]) -> bool {
        if targets.is_empty() {
            return false;
        }

        let ctx = self.runtime_ctx.read().await;
        ctx.execution
            .current_task
            .as_ref()
            .is_some_and(|task| targets.contains(&task.id))
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

    async fn add_policy_overlay(&self, source: PolicySetId, target: PolicySetId) {
        let mut ctx = self.runtime_ctx.write().await;
        let entry = ctx.execution.policy_set_overlays.entry(target).or_default();
        if !entry.contains(&source) {
            entry.push(source);
        }
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
