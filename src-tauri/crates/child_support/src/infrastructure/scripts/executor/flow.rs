impl ScriptExecutor {
    async fn execute_flow_control_step(
        &mut self,
        flow: &FlowControl,
    ) -> ExecuteResult<ControlFlow> {
        match flow {
            FlowControl::If {
                con,
                then,
                else_steps,
            } => {
                if self.evaluate_condition(con).await? {
                    self.execute(then).await
                } else if let Some(else_steps) = else_steps {
                    self.execute(else_steps).await
                } else {
                    Ok(ControlFlow::Next)
                }
            }
            FlowControl::While { con, flow } => {
                let mut iteration = 0usize;
                while self.evaluate_condition(con).await? {
                    iteration += 1;
                    if iteration > MAX_LOOP_ITERATIONS {
                        return Err(Self::execute_error(
                            "flow.loop",
                            format!("循环次数超过上限 {}", MAX_LOOP_ITERATIONS),
                        ));
                    }

                    match self.execute(flow).await? {
                        ControlFlow::Next => continue,
                        ControlFlow::Continue => continue,
                        ControlFlow::Break => break,
                        ControlFlow::Link(target) => return Ok(ControlFlow::Link(target)),
                        ControlFlow::Return => return Ok(ControlFlow::Return),
                    }
                }
                Ok(ControlFlow::Next)
            }
            FlowControl::ForEach {
                input_var,
                item_var,
                index_var,
                flow,
            } => {
                let Some(input) = self.read_runtime_var(input_var).await else {
                    return Ok(ControlFlow::Next);
                };

                let Some(items) = input.clone().try_cast::<Array>() else {
                    return Err(Self::execute_error(
                        "flow.forEach",
                        format!("输入变量[{}]不是数组，无法执行遍历", input_var),
                    ));
                };

                for (index, item) in items.into_iter().enumerate() {
                    if !item_var.trim().is_empty() {
                        self.set_runtime_var(item_var, item).await?;
                    }
                    if !index_var.trim().is_empty() {
                        self.set_runtime_var(index_var, Dynamic::from_int(index as INT))
                            .await?;
                    }

                    match self.execute(flow).await? {
                        ControlFlow::Next => continue,
                        ControlFlow::Continue => continue,
                        ControlFlow::Break => break,
                        ControlFlow::Link(target) => return Ok(ControlFlow::Link(target)),
                        ControlFlow::Return => return Ok(ControlFlow::Return),
                    }
                }

                Ok(ControlFlow::Next)
            }
            FlowControl::Continue => Ok(ControlFlow::Continue),
            FlowControl::Break => Ok(ControlFlow::Break),
            FlowControl::WaitMs { ms } => {
                tokio::time::sleep(Duration::from_millis(*ms)).await;
                Ok(ControlFlow::Next)
            }
            FlowControl::Link { target } => Ok(ControlFlow::Link(*target)),
            FlowControl::AddPolicies { source, target } => {
                self.add_policy_overlay(*source, *target).await;
                Ok(ControlFlow::Next)
            }
            FlowControl::HandlePolicySet {
                target,
                input_var,
                out_var,
            } => self.execute_handle_policy_set(target, input_var, out_var).await,
            FlowControl::HandlePolicy {
                target,
                input_var,
                out_var,
            } => self.execute_handle_policy(target, input_var, out_var).await,
        }
    }

    async fn execute_data_handling_step(
        &mut self,
        data: &DataHanding,
    ) -> ExecuteResult<ControlFlow> {
        match data {
            DataHanding::SetVar { name, val, expr } => {
                let value =
                    if let Some(expr) = expr.as_ref().filter(|value| !value.trim().is_empty()) {
                        self.eval_dynamic(expr, "data.setVar")?
                    } else if let Some(val) = val {
                        Self::var_value_to_dynamic(val)
                    } else {
                        Dynamic::UNIT
                    };
                self.set_runtime_var(name, value).await?;
                Ok(ControlFlow::Next)
            }
            DataHanding::GetVar { name, default_val } => {
                if self.read_runtime_var(name).await.is_none() {
                    if let Some(default_val) = default_val {
                        self.set_runtime_var(name, Self::var_value_to_dynamic(default_val))
                            .await?;
                    }
                }
                Ok(ControlFlow::Next)
            }
            DataHanding::Filter {
                input_var,
                out_name,
                mode,
                logic_expr,
                then_steps,
            } => {
                let Some(input) = self.read_runtime_var(input_var).await else {
                    self.set_runtime_var(out_name, Dynamic::from(Array::new()))
                        .await?;
                    return Ok(ControlFlow::Next);
                };

                let Some(items) = input.clone().try_cast::<Array>() else {
                    return Err(Self::execute_error(
                        "data.filter",
                        format!("输入变量[{}]不是数组，无法执行过滤", input_var),
                    ));
                };

                let mut output = Array::new();
                for (index, item) in items.into_iter().enumerate() {
                    self.scope.set_value(FILTER_ITEM_VAR, item.clone());
                    self.scope.set_value(ITEM_VAR, item.clone());
                    self.scope.set_value(FILTER_INDEX_VAR, index as i64);
                    self.scope.set_value(ITEM_INDEX_VAR, index as i64);

                    let matched = if logic_expr.trim().is_empty() {
                        true
                    } else {
                        self.eval_bool(logic_expr, "data.filter.logicExpr")?
                    };

                    if !matched {
                        continue;
                    }

                    if !then_steps.is_empty() {
                        match self.execute(then_steps).await? {
                            ControlFlow::Next => {}
                            ControlFlow::Continue => continue,
                            ControlFlow::Break => break,
                            ControlFlow::Link(target) => return Ok(ControlFlow::Link(target)),
                            ControlFlow::Return => return Ok(ControlFlow::Return),
                        }
                    }

                    match mode {
                        FilterMode::Filter => output.push(item),
                        FilterMode::Map => {
                            let current = self
                                .scope
                                .get_value::<Dynamic>(ITEM_VAR)
                                .unwrap_or_else(|| Dynamic::UNIT);
                            output.push(current);
                        }
                    }
                }

                self.set_runtime_var(out_name, Dynamic::from(output))
                    .await?;
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_task_control_step(
        &mut self,
        task_control: &TaskControl,
    ) -> ExecuteResult<ControlFlow> {
        match task_control {
            TaskControl::SetState { target, status } => {
                self.set_state_value(target, status).await?;
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_vision_step(&mut self, vision: &VisionNode) -> ExecuteResult<ControlFlow> {
        match vision {
            VisionNode::VisionSearch {
                rule,
                out_var,
                then_steps,
            } => {
                let (hits, matched) = {
                    let ctx = self.runtime_ctx.read().await;
                    if let Some(snapshot) = ctx.observation.last_snapshot.as_ref() {
                        let searcher = OcrSearcher::new(std::slice::from_ref(rule));
                        let hits = searcher.search(snapshot);
                        let matched = rule.evaluate(&hits, &snapshot.det_items);
                        (hits, matched)
                    } else {
                        (Vec::new(), false)
                    }
                };

                {
                    let mut ctx = self.runtime_ctx.write().await;
                    ctx.observation.last_hits = hits.clone();
                }

                self.set_runtime_var(out_var, Self::search_hits_to_dynamic(&hits))
                    .await?;

                if matched && !then_steps.is_empty() {
                    return self.execute(then_steps).await;
                }

                Ok(ControlFlow::Next)
            }
        }
    }

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
                    "颜色比较尚未接入视觉颜色分析".to_string(),
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
        let (target, status) = match task_control {
            TaskControl::SetState { target, status } => (target, status),
        };

        Ok(self.match_state_value(target, status).await)
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
