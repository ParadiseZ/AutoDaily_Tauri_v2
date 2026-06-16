#[derive(Debug, Clone)]
struct CompiledSequenceOperation {
    operation: DeviceOperation,
    trace: Option<PolicyActionTrace>,
    debug_label: String,
}

struct SequenceOperationCompiler<'a> {
    executor: &'a mut ScriptExecutor,
}

impl ScriptExecutor {
    async fn compile_sequence_operations(
        &mut self,
        steps: &[Step],
    ) -> ExecuteResult<Option<Vec<CompiledSequenceOperation>>> {
        SequenceOperationCompiler { executor: self }
            .compile(steps)
            .await
    }
}

impl<'a> SequenceOperationCompiler<'a> {
    async fn compile(
        &mut self,
        steps: &[Step],
    ) -> ExecuteResult<Option<Vec<CompiledSequenceOperation>>> {
        let mut operations = Vec::new();
        for step in steps {
            if step.skip_flag {
                continue;
            }

            let Some(mut next_operations) = self.compile_step(step).await? else {
                return Ok(None);
            };
            operations.append(&mut next_operations);
        }
        Ok(Some(operations))
    }

    async fn compile_step(
        &mut self,
        step: &Step,
    ) -> ExecuteResult<Option<Vec<CompiledSequenceOperation>>> {
        match &step.kind {
            StepKind::Action { exec_max, a } => {
                if *exec_max > 0 {
                    return Ok(None);
                }
                Ok(self.compile_action(a).await?.map(|operation| vec![operation]))
            }
            StepKind::FlowControl {
                a:
                    FlowControl::WaitMs {
                        ms,
                        input_var,
                        runtime_var,
                    },
            } => {
                if input_var
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                    || runtime_var
                        .as_deref()
                        .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(None);
                }
                if *ms == 0 {
                    return Ok(Some(Vec::new()));
                }
                Ok(Some(vec![CompiledSequenceOperation {
                    operation: DeviceOperation::Delay(*ms),
                    trace: None,
                    debug_label: format!("等待 {}ms", ms),
                }]))
            }
            _ => Ok(None),
        }
    }

    async fn compile_action(
        &mut self,
        action: &Action,
    ) -> ExecuteResult<Option<CompiledSequenceOperation>> {
        match action {
            Action::Click {
                mode,
                offset_x,
                offset_y,
            } => self.compile_click(mode, *offset_x, *offset_y).await,
            Action::Swipe { duration, mode } => self.compile_swipe(mode, *duration).await,
            Action::LongClick {
                mode,
                offset_x,
                offset_y,
            } => self.compile_long_click(mode, *offset_x, *offset_y).await,
            Action::Reboot => Ok(None),
            Action::Back => Ok(Some(CompiledSequenceOperation {
                operation: DeviceOperation::Back,
                trace: Some(ScriptExecutor::build_simple_action_trace(
                    PolicyActionKind::Back,
                )),
                debug_label: "返回键".to_string(),
            })),
            Action::Home => Ok(Some(CompiledSequenceOperation {
                operation: DeviceOperation::Home,
                trace: Some(ScriptExecutor::build_simple_action_trace(
                    PolicyActionKind::Home,
                )),
                debug_label: "主页键".to_string(),
            })),
            Action::InputText { text } => Ok(Some(CompiledSequenceOperation {
                operation: DeviceOperation::InputText(text.clone()),
                trace: Some(ScriptExecutor::build_simple_action_trace(
                    PolicyActionKind::Input,
                )),
                debug_label: format!("输入文本({})", text),
            })),
            Action::LaunchApp {
                pkg_name,
                activity_name,
            } => {
                if pkg_name.trim().is_empty() || activity_name.trim().is_empty() {
                    return Ok(None);
                }
                Ok(Some(CompiledSequenceOperation {
                    operation: DeviceOperation::LaunchApp {
                        pkg_name: pkg_name.clone(),
                        activity_name: activity_name.clone(),
                    },
                    trace: Some(ScriptExecutor::build_simple_action_trace(
                        PolicyActionKind::StartApp,
                    )),
                    debug_label: format!("启动应用({}/{})", pkg_name, activity_name),
                }))
            }
            Action::StopApp { pkg_name } => Ok(Some(CompiledSequenceOperation {
                operation: DeviceOperation::StopApp {
                    pkg_name: pkg_name.clone(),
                },
                trace: Some(ScriptExecutor::build_simple_action_trace(
                    PolicyActionKind::StopApp,
                )),
                debug_label: format!("停止应用({})", pkg_name),
            })),
            _ => Ok(None),
        }
    }

    async fn compile_click(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<Option<CompiledSequenceOperation>> {
        let point = match mode {
            ClickMode::Point { p } => ScriptExecutor::point_to_absolute(p),
            ClickMode::Percent { p } => {
                let screen_size = self.executor.ensure_screen_size().await?;
                ScriptExecutor::percent_point_to_absolute(p, screen_size)?
            }
            ClickMode::Txt { .. } | ClickMode::LabelIdx { .. } => return Ok(None),
        };
        let point = self
            .executor
            .apply_click_fixed_offset(point, offset_x, offset_y)
            .await?;
        let point = self.executor.apply_click_random_offset(point).await?;
        Ok(Some(CompiledSequenceOperation {
            operation: DeviceOperation::Click(point),
            trace: Some(ScriptExecutor::build_action_trace(
                PolicyActionKind::Click,
                PolicyActionSource::Fixed,
                vec![ScriptExecutor::build_point_target(
                    PolicyActionTargetRole::Primary,
                    point,
                )],
            )),
            debug_label: format!("点击({}, {})", point.x, point.y),
        }))
    }

    async fn compile_long_click(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<Option<CompiledSequenceOperation>> {
        let point = match mode {
            ClickMode::Point { p } => ScriptExecutor::point_to_absolute(p),
            ClickMode::Percent { p } => {
                let screen_size = self.executor.ensure_screen_size().await?;
                ScriptExecutor::percent_point_to_absolute(p, screen_size)?
            }
            ClickMode::Txt { .. } | ClickMode::LabelIdx { .. } => return Ok(None),
        };
        let point = self
            .executor
            .apply_click_fixed_offset(point, offset_x, offset_y)
            .await?;
        let point = self.executor.apply_click_random_offset(point).await?;
        Ok(Some(CompiledSequenceOperation {
            operation: DeviceOperation::LongClick(point),
            trace: Some(ScriptExecutor::build_action_trace(
                PolicyActionKind::Press,
                PolicyActionSource::Fixed,
                vec![ScriptExecutor::build_point_target(
                    PolicyActionTargetRole::Primary,
                    point,
                )],
            )),
            debug_label: format!("长按({}, {})", point.x, point.y),
        }))
    }

    async fn compile_swipe(
        &mut self,
        mode: &SwipeMode,
        duration: u64,
    ) -> ExecuteResult<Option<CompiledSequenceOperation>> {
        let (from, to) = match mode {
            SwipeMode::Point { from, to } => (
                ScriptExecutor::point_to_absolute(from),
                ScriptExecutor::point_to_absolute(to),
            ),
            SwipeMode::Percent { from, to } => {
                let screen_size = self.executor.ensure_screen_size().await?;
                (
                    ScriptExecutor::percent_point_to_absolute(from, screen_size)?,
                    ScriptExecutor::percent_point_to_absolute(to, screen_size)?,
                )
            }
            SwipeMode::Txt { .. } | SwipeMode::LabelIdx { .. } | SwipeMode::Mixed { .. } => {
                return Ok(None)
            }
        };
        Ok(Some(CompiledSequenceOperation {
            operation: DeviceOperation::Swipe { from, to, duration },
            trace: Some(ScriptExecutor::build_action_trace(
                PolicyActionKind::Swipe,
                PolicyActionSource::Fixed,
                vec![
                    ScriptExecutor::build_point_target(PolicyActionTargetRole::Start, from),
                    ScriptExecutor::build_point_target(PolicyActionTargetRole::End, to),
                ],
            )),
            debug_label: format!(
                "滑动(({}, {}) -> ({}, {}), {}ms)",
                from.x, from.y, to.x, to.y, duration
            ),
        }))
    }
}
