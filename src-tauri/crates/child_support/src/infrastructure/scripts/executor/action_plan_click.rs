impl ScriptExecutor {
    fn select_positioned_owned<T: Clone>(items: Vec<T>, position: Option<u16>) -> Vec<T> {
        if items.is_empty() {
            return items;
        }
        let index = match position {
            Some(999) => items.len() - 1,
            Some(value) => usize::from(value).min(items.len() - 1),
            None => 0,
        };
        vec![items[index].clone()]
    }

    async fn resolve_active_policy_click_pos(&self) -> Option<u16> {
        let context = self.active_policy_context.as_ref()?;
        let ctx = self.runtime_ctx.read().await;
        Some(
            ctx.execution
                .policy_states
                .get(&context.policy_id)
                .and_then(|state| state.click_pos)
                .unwrap_or(context.base_click_pos),
        )
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
            trace: Some(ActionTraceBuilder::build_action_trace(
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
            trace: Some(ActionTraceBuilder::build_action_trace(
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
                    targets: vec![ActionTraceBuilder::build_point_target(
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
                    targets: vec![ActionTraceBuilder::build_point_target(
                        PolicyActionTargetRole::Primary,
                        point,
                    )],
                })
            }
            ClickMode::Txt {
                input_var,
                txt,
                txt_expr,
                enable_filter,
            } => {
                let target_text =
                    self.resolve_optional_text(txt.as_deref(), txt_expr.as_deref(), step_type)?;
                let items = self
                    .resolve_ocr_target_items(step_type, input_var, target_text.as_deref())
                    .await?;
                let items = if *enable_filter {
                    Self::select_positioned_owned(items, self.resolve_active_policy_click_pos().await)
                } else {
                    items
                };
                let mut points = Vec::new();
                let mut targets = Vec::new();
                for item in items {
                    let point =
                        Self::bounding_box_center_to_point(step_type, label, &item.bounding_box)?;
                    points.push(point);
                    targets.push(ActionTraceBuilder::build_ocr_target(
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
            ClickMode::LabelIdx {
                input_var,
                idx,
                enable_filter,
            } => {
                let items = self.resolve_det_target_items(step_type, input_var, *idx).await?;
                let items = if *enable_filter {
                    Self::select_positioned_owned(items, self.resolve_active_policy_click_pos().await)
                } else {
                    items
                };
                let mut points = Vec::new();
                let mut targets = Vec::new();
                for item in items {
                    let point =
                        Self::bounding_box_center_to_point(step_type, label, &item.bounding_box)?;
                    points.push(point);
                    targets.push(ActionTraceBuilder::build_det_target(
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
}
