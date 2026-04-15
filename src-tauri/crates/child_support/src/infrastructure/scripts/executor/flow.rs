impl ScriptExecutor {
    async fn execute_flow_control_step(
        &mut self,
        cur_exec_num: u32,
        max_exec_num: u32,
        flow: &FlowControl,
    ) -> ExecuteResult<ControlFlow> {
        if cur_exec_num > max_exec_num {
            return Ok(ControlFlow::Next);
        }

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
            FlowControl::Link { target } => Err(Self::execute_error(
                "flow.link",
                format!("跳转任务[{}]尚未接入调度器切换逻辑", target),
            )),
            FlowControl::AddPolicies { .. } => Err(Self::execute_error(
                "flow.addPolicies",
                "动态策略集拼装尚未接入运行时".to_string(),
            )),
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
            TaskControl::GetState { target, status: _ } => Err(Self::execute_error(
                "taskControl.getState",
                format!(
                    "GetState 只应用于条件节点 TaskStatus，不应作为步骤执行[target={}]",
                    Self::state_target_label(target)
                ),
            )),
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
                ConditionNode::ExecNumCompare { .. } => Err(Self::execute_error(
                    "condition.execNumCompare",
                    "执行次数条件尚未定义比较阈值，当前不执行隐式推断".to_string(),
                )),
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
            TaskControl::GetState { target, status } | TaskControl::SetState { target, status } => {
                (target, status)
            }
        };

        let ctx = self.runtime_ctx.read().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx
                    .execution
                    .task_states
                    .get(id)
                    .cloned()
                    .unwrap_or_else(TaskState::default);
                Ok(match status {
                    StateStatus::Enabled { value } => state.enabled_flag == *value,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                })
            }
            StateTarget::Policy { id } => {
                let state = ctx
                    .execution
                    .policy_states
                    .get(id)
                    .cloned()
                    .unwrap_or_default();
                Ok(match status {
                    StateStatus::Enabled { .. } => false,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                })
            }
        }
    }
}
