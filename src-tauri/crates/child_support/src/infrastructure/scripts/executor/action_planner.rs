struct PlannedDeviceAction {
    operations: Vec<DeviceOperation>,
    trace: Option<PolicyActionTrace>,
}

struct ResolvedPrimaryTargets {
    points: Vec<Point<u16>>,
    source: PolicyActionSource,
    targets: Vec<PolicyActionTarget>,
}

impl ScriptExecutor {
    async fn execute_planned_device_action(
        &self,
        step_type: &str,
        label: &str,
        plan: PlannedDeviceAction,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        if plan.operations.is_empty() {
            return Ok((ControlFlow::Next, plan.trace));
        }

        Self::await_device_result_with_timeout(
            step_type,
            label,
            DEVICE_EXTERNAL_TIMEOUT_MS,
            get_device_ctx().execute_operations(&plan.operations),
        )
        .await?;

        Ok((ControlFlow::Next, plan.trace))
    }

    async fn plan_click_action(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<PlannedDeviceAction> {
        let ResolvedPrimaryTargets {
            points,
            source,
            mut targets,
        } = self
            .resolve_primary_click_targets("action.click", "点击目标", mode)
            .await?;

        if points.is_empty() {
            return Ok(PlannedDeviceAction {
                operations: Vec::new(),
                trace: None,
            });
        }

        let mut fixed_points = Vec::with_capacity(points.len());
        let mut operations = Vec::with_capacity(points.len());
        for point in points {
            let fixed_point = self.apply_click_fixed_offset(point, offset_x, offset_y).await?;
            let click_point = self.apply_click_random_offset(fixed_point).await?;
            fixed_points.push(fixed_point);
            operations.push(DeviceOperation::Click(click_point));
        }

        for (target, point) in targets.iter_mut().zip(fixed_points) {
            target.point = Some(PointU16 {
                x: point.x,
                y: point.y,
            });
        }

        Ok(PlannedDeviceAction {
            operations,
            trace: Some(Self::build_action_trace(
                PolicyActionKind::Click,
                source,
                targets,
            )),
        })
    }

    async fn plan_long_click_action(
        &mut self,
        mode: &ClickMode,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<PlannedDeviceAction> {
        let ResolvedPrimaryTargets {
            points,
            source,
            targets,
        } = self
            .resolve_primary_click_targets("action.longClick", "长按目标", mode)
            .await?;

        if points.is_empty() {
            return Ok(PlannedDeviceAction {
                operations: Vec::new(),
                trace: None,
            });
        }

        let mut operations = Vec::with_capacity(points.len());
        for point in points {
            let fixed_point = self.apply_click_fixed_offset(point, offset_x, offset_y).await?;
            let click_point = self.apply_click_random_offset(fixed_point).await?;
            operations.push(DeviceOperation::LongClick(click_point));
        }

        Ok(PlannedDeviceAction {
            operations,
            trace: Some(Self::build_action_trace(
                PolicyActionKind::Press,
                source,
                targets,
            )),
        })
    }

    async fn resolve_primary_click_targets(
        &mut self,
        step_type: &str,
        label: &str,
        mode: &ClickMode,
    ) -> ExecuteResult<ResolvedPrimaryTargets> {
        match mode {
            ClickMode::Point { p } => {
                let point = Self::point_to_absolute(p);
                Ok(ResolvedPrimaryTargets {
                    points: vec![point],
                    source: PolicyActionSource::Fixed,
                    targets: vec![Self::build_point_target(
                        PolicyActionTargetRole::Primary,
                        point,
                    )],
                })
            }
            ClickMode::Percent { p } => {
                let screen_size = self.ensure_screen_size().await?;
                let point = Self::percent_point_to_absolute(p, screen_size)?;
                Ok(ResolvedPrimaryTargets {
                    points: vec![point],
                    source: PolicyActionSource::Fixed,
                    targets: vec![Self::build_point_target(
                        PolicyActionTargetRole::Primary,
                        point,
                    )],
                })
            }
            ClickMode::Txt {
                input_var,
                txt,
                txt_expr,
            } => {
                let target_text =
                    self.resolve_optional_text(txt.as_deref(), txt_expr.as_deref(), step_type)?;
                let items = self
                    .resolve_ocr_target_items(step_type, input_var, target_text.as_deref())
                    .await?;
                let mut points = Vec::new();
                let mut targets = Vec::new();
                for item in items {
                    let point =
                        Self::bounding_box_center_to_point(step_type, label, &item.bounding_box)?;
                    points.push(point);
                    targets.push(Self::build_ocr_target(
                        PolicyActionTargetRole::Primary,
                        point,
                        &item,
                    ));
                }
                Ok(ResolvedPrimaryTargets {
                    points,
                    source: PolicyActionSource::Ocr,
                    targets,
                })
            }
            ClickMode::LabelIdx { input_var, idx } => {
                let items = self.resolve_det_target_items(step_type, input_var, *idx).await?;
                let mut points = Vec::new();
                let mut targets = Vec::new();
                for item in items {
                    let point =
                        Self::bounding_box_center_to_point(step_type, label, &item.bounding_box)?;
                    points.push(point);
                    targets.push(Self::build_det_target(
                        PolicyActionTargetRole::Primary,
                        point,
                        &item,
                    ));
                }
                Ok(ResolvedPrimaryTargets {
                    points,
                    source: PolicyActionSource::Det,
                    targets,
                })
            }
        }
    }

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
                    Self::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Fixed,
                        vec![
                            Self::build_point_target(PolicyActionTargetRole::Start, from_point),
                            Self::build_point_target(PolicyActionTargetRole::End, to_point),
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
                    Self::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Fixed,
                        vec![
                            Self::build_point_target(PolicyActionTargetRole::Start, from_point),
                            Self::build_point_target(PolicyActionTargetRole::End, to_point),
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
                    Self::build_action_trace(
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
                    Self::build_action_trace(
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
                    Self::build_action_trace(
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
