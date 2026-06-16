impl ScriptExecutor {
    async fn plan_swipe_action(
        &mut self,
        mode: &SwipeMode,
        duration: u64,
    ) -> ExecuteResult<PlannedDeviceAction> {
        let (from, to, trace) = match mode {
            SwipeMode::Point { from, to } => {
                let from_point = Self::point_to_absolute(from);
                let to_point = Self::point_to_absolute(to);
                (
                    from_point,
                    to_point,
                    ActionTraceBuilder::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Fixed,
                        vec![
                            ActionTraceBuilder::build_point_target(
                                PolicyActionTargetRole::Start,
                                from_point,
                            ),
                            ActionTraceBuilder::build_point_target(
                                PolicyActionTargetRole::End,
                                to_point,
                            ),
                        ],
                    ),
                )
            }
            SwipeMode::Percent { from, to } => {
                let screen_size = self.ensure_screen_size().await?;
                let from_point = Self::percent_point_to_absolute(from, screen_size)?;
                let to_point = Self::percent_point_to_absolute(to, screen_size)?;
                (
                    from_point,
                    to_point,
                    ActionTraceBuilder::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Fixed,
                        vec![
                            ActionTraceBuilder::build_point_target(
                                PolicyActionTargetRole::Start,
                                from_point,
                            ),
                            ActionTraceBuilder::build_point_target(
                                PolicyActionTargetRole::End,
                                to_point,
                            ),
                        ],
                    ),
                )
            }
            SwipeMode::Txt {
                input_var,
                from,
                to,
                from_expr,
                to_expr,
            } => {
                let from_text = self.resolve_optional_text(
                    from.as_deref(),
                    from_expr.as_deref(),
                    "action.swipe",
                )?;
                let to_text =
                    self.resolve_optional_text(to.as_deref(), to_expr.as_deref(), "action.swipe")?;
                let (from_point, from_target) = self
                    .resolve_swipe_ocr_endpoint(
                        "action.swipe",
                        input_var,
                        from_text.as_deref(),
                        "文字滑动起点",
                        PolicyActionTargetRole::Start,
                    )
                    .await?;
                let (to_point, to_target) = self
                    .resolve_swipe_ocr_endpoint(
                        "action.swipe",
                        input_var,
                        to_text.as_deref(),
                        "文字滑动终点",
                        PolicyActionTargetRole::End,
                    )
                    .await?;
                (
                    from_point,
                    to_point,
                    ActionTraceBuilder::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Ocr,
                        vec![from_target, to_target],
                    ),
                )
            }
            SwipeMode::LabelIdx {
                input_var,
                from,
                to,
            } => {
                let (from_point, from_target) = self
                    .resolve_swipe_det_endpoint(
                        "action.swipe",
                        input_var,
                        u32::from(*from),
                        "标签滑动起点",
                        PolicyActionTargetRole::Start,
                    )
                    .await?;
                let (to_point, to_target) = self
                    .resolve_swipe_det_endpoint(
                        "action.swipe",
                        input_var,
                        u32::from(*to),
                        "标签滑动终点",
                        PolicyActionTargetRole::End,
                    )
                    .await?;
                (
                    from_point,
                    to_point,
                    ActionTraceBuilder::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Det,
                        vec![from_target, to_target],
                    ),
                )
            }
            SwipeMode::Mixed { from, to } => {
                let (from_point, from_target) = self
                    .resolve_swipe_target("action.swipe", from, PolicyActionTargetRole::Start)
                    .await?;
                let (to_point, to_target) = self
                    .resolve_swipe_target("action.swipe", to, PolicyActionTargetRole::End)
                    .await?;
                (
                    from_point,
                    to_point,
                    ActionTraceBuilder::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Custom,
                        vec![from_target, to_target],
                    ),
                )
            }
        };

        Ok(PlannedDeviceAction {
            operations: vec![DeviceOperation::Swipe { from, to, duration }],
            trace: Some(trace),
        })
    }
}
