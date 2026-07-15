#[derive(Debug, Clone)]
struct CompiledSequenceOperation {
    operation: DeviceOperation,
    trace: Option<PolicyActionTrace>,
    debug_label: String,
}

#[derive(Debug, Clone)]
enum SequenceCompileOutcome {
    Supported(Vec<CompiledSequenceOperation>),
    Unsupported(SequenceCompileBlocker),
}

#[derive(Debug, Clone)]
struct SequenceCompileBlocker {
    step_label: String,
    reason: String,
}

struct SequenceOperationCompiler<'a> {
    executor: &'a mut ScriptExecutor,
}

impl ScriptExecutor {
    async fn compile_sequence_operations(
        &mut self,
        steps: &[Step],
    ) -> ExecuteResult<SequenceCompileOutcome> {
        SequenceOperationCompiler { executor: self }
            .compile(steps)
            .await
    }
}

impl<'a> SequenceOperationCompiler<'a> {
    async fn compile(
        &mut self,
        steps: &[Step],
    ) -> ExecuteResult<SequenceCompileOutcome> {
        let mut operations = Vec::new();
        for step in steps {
            if step.skip_flag {
                continue;
            }

            let next_operations = match self.compile_step(step).await? {
                Ok(next_operations) => next_operations,
                Err(reason) => {
                    return Ok(SequenceCompileOutcome::Unsupported(
                        SequenceCompileBlocker {
                            step_label: self.describe_step(step),
                            reason,
                        },
                    ));
                }
            };
            operations.extend(next_operations);
        }
        Ok(SequenceCompileOutcome::Supported(operations))
    }

    async fn compile_step(
        &mut self,
        step: &Step,
    ) -> ExecuteResult<Result<Vec<CompiledSequenceOperation>, String>> {
        match &step.kind {
            StepKind::Action { exec_max, a } => {
                if *exec_max > 0 {
                    return Ok(Err(
                        "包含 exec_max 限制，必须走普通动作执行路径".to_string(),
                    ));
                }
                self.compile_action(a)
                    .await
                    .map(|result| result.map(|operation| vec![operation]))
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
                    return Ok(Err(
                        "WaitMs 依赖输入变量或运行时变量，不能合并进固定 Sequence".to_string(),
                    ));
                }
                if *ms == 0 {
                    return Ok(Ok(Vec::new()));
                }
                Ok(Ok(vec![CompiledSequenceOperation {
                    operation: DeviceOperation::Delay(*ms),
                    trace: None,
                    debug_label: format!("等待 {}ms", ms),
                }]))
            }
            StepKind::FlowControl { .. } => Ok(Err(
                "Sequence 只支持固定等待，不支持其它流程控制".to_string(),
            )),
            StepKind::DataHanding { .. } => Ok(Err(
                "Sequence 不支持数据处理步骤".to_string(),
            )),
            StepKind::TaskControl { .. } => Ok(Err(
                "Sequence 不支持任务状态控制步骤".to_string(),
            )),
            StepKind::Vision { .. } => Ok(Err(
                "Sequence 不支持视觉步骤".to_string(),
            )),
            StepKind::Sequence { .. } => Ok(Err(
                "Sequence 不支持嵌套 Sequence".to_string(),
            )),
        }
    }

    async fn compile_action(
        &mut self,
        action: &Action,
    ) -> ExecuteResult<Result<CompiledSequenceOperation, String>> {
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
            Action::Capture { .. } => Ok(Err(
                "截图需要单独结果通道，不能合并进 Sequence".to_string(),
            )),
            Action::Reboot => Ok(Err(
                "重启动作依赖独立 ADB 指令通道，不能合并进 Sequence".to_string(),
            )),
            Action::Back => Ok(Ok(CompiledSequenceOperation {
                operation: DeviceOperation::Back,
                trace: Some(ActionTraceBuilder::build_simple_action_trace(
                    PolicyActionKind::Back,
                )),
                debug_label: "返回键".to_string(),
            })),
            Action::Home => Ok(Ok(CompiledSequenceOperation {
                operation: DeviceOperation::Home,
                trace: Some(ActionTraceBuilder::build_simple_action_trace(
                    PolicyActionKind::Home,
                )),
                debug_label: "主页键".to_string(),
            })),
            Action::InputText { text } => Ok(Ok(CompiledSequenceOperation {
                operation: DeviceOperation::InputText(text.clone()),
                trace: Some(ActionTraceBuilder::build_simple_action_trace(
                    PolicyActionKind::Input,
                )),
                debug_label: format!("输入文本({})", text),
            })),
            Action::PosAdd { .. } | Action::PosMinus { .. } => Ok(Err(
                "策略点击索引调整依赖运行时策略状态，不能合并进 Sequence".to_string(),
            )),
            Action::DropSetNext { .. } => Ok(Err(
                "UI 变量下一个需要读写模板变量，不能合并进 Sequence".to_string(),
            )),
            Action::LaunchApp {
                pkg_name,
                pkg_name_expr,
                activity_name,
                activity_name_expr,
            } => {
                if pkg_name_expr
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                    || activity_name_expr
                        .as_deref()
                        .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(Err(
                        "LaunchApp 依赖变量绑定，不能合并进固定 Sequence".to_string(),
                    ));
                }
                if pkg_name.trim().is_empty() || activity_name.trim().is_empty() {
                    return Ok(Err(
                        "LaunchApp 缺少 pkg_name 或 activity_name".to_string(),
                    ));
                }
                Ok(Ok(CompiledSequenceOperation {
                    operation: DeviceOperation::LaunchApp {
                        pkg_name: pkg_name.clone(),
                        activity_name: activity_name.clone(),
                    },
                    trace: Some(ActionTraceBuilder::build_simple_action_trace(
                        PolicyActionKind::StartApp,
                    )),
                    debug_label: format!("启动应用({}/{})", pkg_name, activity_name),
                }))
            }
            Action::StopApp {
                pkg_name,
                pkg_name_expr,
            } => {
                if pkg_name_expr
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(Err(
                        "StopApp 依赖变量绑定，不能合并进固定 Sequence".to_string(),
                    ));
                }
                Ok(Ok(CompiledSequenceOperation {
                    operation: DeviceOperation::StopApp {
                        pkg_name: pkg_name.clone(),
                    },
                    trace: Some(ActionTraceBuilder::build_simple_action_trace(
                        PolicyActionKind::StopApp,
                    )),
                    debug_label: format!("停止应用({})", pkg_name),
                }))
            }
        }
    }

    async fn compile_click(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<Result<CompiledSequenceOperation, String>> {
        let point = match mode {
            ClickMode::Point { p, p_expr } => {
                if p_expr
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(Err(
                        "点击依赖变量点位，不能合并进固定 Sequence".to_string(),
                    ));
                }
                ScriptExecutor::point_to_absolute(p)
            }
            ClickMode::Percent { p, p_expr } => {
                if p_expr
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(Err(
                        "点击依赖变量点位，不能合并进固定 Sequence".to_string(),
                    ));
                }
                let screen_size = self.executor.ensure_screen_size().await?;
                ScriptExecutor::percent_point_to_absolute(p, screen_size)?
            }
            ClickMode::Txt { .. } | ClickMode::LabelIdx { .. } => {
                return Ok(Err(
                    "点击依赖 OCR/检测结果，不能合并进固定 Sequence".to_string(),
                ));
            }
        };
        let point = self
            .executor
            .apply_click_fixed_offset(point, offset_x, offset_y)
            .await?;
        let point = self.executor.apply_click_random_offset(point).await?;
        Ok(Ok(CompiledSequenceOperation {
            operation: DeviceOperation::Click(point),
            trace: Some(ActionTraceBuilder::build_action_trace(
                PolicyActionKind::Click,
                PolicyActionSource::Fixed,
                vec![ActionTraceBuilder::build_point_target(
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
    ) -> ExecuteResult<Result<CompiledSequenceOperation, String>> {
        let point = match mode {
            ClickMode::Point { p, p_expr } => {
                if p_expr
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(Err(
                        "长按依赖变量点位，不能合并进固定 Sequence".to_string(),
                    ));
                }
                ScriptExecutor::point_to_absolute(p)
            }
            ClickMode::Percent { p, p_expr } => {
                if p_expr
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                {
                    return Ok(Err(
                        "长按依赖变量点位，不能合并进固定 Sequence".to_string(),
                    ));
                }
                let screen_size = self.executor.ensure_screen_size().await?;
                ScriptExecutor::percent_point_to_absolute(p, screen_size)?
            }
            ClickMode::Txt { .. } | ClickMode::LabelIdx { .. } => {
                return Ok(Err(
                    "长按依赖 OCR/检测结果，不能合并进固定 Sequence".to_string(),
                ));
            }
        };
        let point = self
            .executor
            .apply_click_fixed_offset(point, offset_x, offset_y)
            .await?;
        let point = self.executor.apply_click_random_offset(point).await?;
        Ok(Ok(CompiledSequenceOperation {
            operation: DeviceOperation::LongClick(point),
            trace: Some(ActionTraceBuilder::build_action_trace(
                PolicyActionKind::Press,
                PolicyActionSource::Fixed,
                vec![ActionTraceBuilder::build_point_target(
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
    ) -> ExecuteResult<Result<CompiledSequenceOperation, String>> {
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
                return Ok(Err(
                    "滑动依赖 OCR/检测结果，不能合并进固定 Sequence".to_string(),
                ));
            }
        };
        Ok(Ok(CompiledSequenceOperation {
            operation: DeviceOperation::Swipe { from, to, duration },
            trace: Some(ActionTraceBuilder::build_action_trace(
                PolicyActionKind::Swipe,
                PolicyActionSource::Fixed,
                vec![
                    ActionTraceBuilder::build_point_target(PolicyActionTargetRole::Start, from),
                    ActionTraceBuilder::build_point_target(PolicyActionTargetRole::End, to),
                ],
            )),
            debug_label: format!(
                "滑动(({}, {}) -> ({}, {}), {}ms)",
                from.x, from.y, to.x, to.y, duration
            ),
        }))
    }

    fn describe_step(&self, step: &Step) -> String {
        step.label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| match &step.kind {
                StepKind::Action { .. } => "未命名动作".to_string(),
                StepKind::FlowControl { .. } => "未命名流程".to_string(),
                StepKind::DataHanding { .. } => "未命名数据处理".to_string(),
                StepKind::TaskControl { .. } => "未命名任务控制".to_string(),
                StepKind::Vision { .. } => "未命名视觉步骤".to_string(),
                StepKind::Sequence { .. } => "未命名 Sequence".to_string(),
            })
    }
}
