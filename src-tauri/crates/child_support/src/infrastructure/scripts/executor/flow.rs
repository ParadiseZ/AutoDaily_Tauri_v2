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
                if let Some(timeout_flow) = self
                    .record_progress_evidence("flow.if", "If 条件检查")
                    .await?
                {
                    return Ok(timeout_flow);
                }
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
                loop {
                    if let Some(timeout_flow) = self
                        .record_progress_evidence(
                            "flow.while",
                            "While 条件检查与循环推进",
                        )
                        .await?
                    {
                        return Ok(timeout_flow);
                    }

                    if !self.evaluate_condition(con).await? {
                        break;
                    }

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
                        ControlFlow::StopScript => return Ok(ControlFlow::StopScript),
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
                    if let Some(timeout_flow) = self
                        .record_progress_evidence(
                            "flow.forEach",
                            format!(
                                "ForEach 遍历推进: input_var={}, index={}",
                                input_var, index
                            ),
                        )
                        .await?
                    {
                        return Ok(timeout_flow);
                    }

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
                        ControlFlow::StopScript => return Ok(ControlFlow::StopScript),
                    }
                }

                Ok(ControlFlow::Next)
            }
            FlowControl::Repeat {
                count_expr,
                index_var,
                flow,
            } => {
                let count = self.eval_repeat_count(count_expr, "flow.repeat")?;
                if count > MAX_LOOP_ITERATIONS {
                    return Err(Self::execute_error(
                        "flow.repeat",
                        format!("循环次数超过上限 {}", MAX_LOOP_ITERATIONS),
                    ));
                }

                for index in 0..count {
                    if let Some(timeout_flow) = self
                        .record_progress_evidence(
                            "flow.repeat",
                            format!("Repeat 循环推进: index={}/{}", index + 1, count),
                        )
                        .await?
                    {
                        return Ok(timeout_flow);
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
                        ControlFlow::StopScript => return Ok(ControlFlow::StopScript),
                    }
                }

                Ok(ControlFlow::Next)
            }
            FlowControl::Continue => Ok(ControlFlow::Continue),
            FlowControl::Break => Ok(ControlFlow::Break),
            FlowControl::StopScript => Ok(ControlFlow::StopScript),
            FlowControl::WaitMs {
                ms,
                input_var,
                runtime_var,
            } => {
                let effective_ms = self
                    .resolve_wait_duration_ms(*ms, input_var.as_deref(), runtime_var.as_deref())
                    .await;
                if let Some(timeout_flow) = self
                    .sleep_with_progress_timeout(
                        effective_ms,
                        "flow.waitMs",
                        format!("WaitMs 等待 {}ms", effective_ms),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                Ok(ControlFlow::Next)
            }
            FlowControl::Link { target } => Ok(ControlFlow::Link(*target)),
            FlowControl::AddPolicies {
                source,
                target,
                top,
                reverse,
            } => {
                self.execute_add_policies_binding(*source, *target, *top, *reverse)
                    .await?;
                Ok(ControlFlow::Next)
            }
            FlowControl::BindPolicyGroup {
                source,
                target,
                top,
                reverse,
            } => {
                self.execute_bind_policy_group_step(*source, *target, *top, *reverse)
                    .await?;
                Ok(ControlFlow::Next)
            }
            FlowControl::BindPolicy {
                source,
                target,
                top,
                reverse,
            } => {
                self.execute_bind_policy_step(*source, *target, *top, *reverse)
                    .await?;
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
}
