impl ScriptExecutor {
    async fn resolve_swipe_ocr_endpoint(
        &self,
        step_type: &str,
        input_var: &str,
        target_text: Option<&str>,
        target_label: &str,
        role: PolicyActionTargetRole,
    ) -> ExecuteResult<(Point<u16>, PolicyActionTarget)> {
        let items = self
            .read_runtime_result_vec::<OcrResult>(input_var, step_type, "OCR")
            .await?;
        let item = Self::select_ocr_result(&items, target_text).ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!(
                    "输入变量[{}]里未找到{}: {}",
                    input_var,
                    target_label,
                    target_text.unwrap_or_default()
                ),
            )
        })?;
        let point = Self::bounding_box_center_to_point(step_type, target_label, &item.bounding_box)?;
        Ok((point, ActionTraceBuilder::build_ocr_target(role, point, item)))
    }

    async fn resolve_swipe_det_endpoint(
        &self,
        step_type: &str,
        input_var: &str,
        target_idx: u32,
        target_label: &str,
        role: PolicyActionTargetRole,
    ) -> ExecuteResult<(Point<u16>, PolicyActionTarget)> {
        let items = self
            .read_runtime_result_vec::<DetResult>(input_var, step_type, "检测")
            .await?;
        let item = Self::select_det_result(&items, Some(target_idx)).ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!("输入变量[{}]里未找到{}: {}", input_var, target_label, target_idx),
            )
        })?;
        let point = Self::bounding_box_center_to_point(step_type, target_label, &item.bounding_box)?;
        Ok((point, ActionTraceBuilder::build_det_target(role, point, item)))
    }

    async fn resolve_swipe_target(
        &mut self,
        step_type: &str,
        target: &SwipeTarget,
        role: PolicyActionTargetRole,
    ) -> ExecuteResult<(Point<u16>, PolicyActionTarget)> {
        match target {
            SwipeTarget::Txt {
                input_var,
                value,
                value_expr,
            } => {
                let target_text =
                    self.resolve_optional_text(value.as_deref(), value_expr.as_deref(), step_type)?;
                let item = self
                    .resolve_ocr_target_items(step_type, input_var, target_text.as_deref())
                    .await?
                    .into_iter()
                    .next()
                    .ok_or_else(|| {
                        Self::execute_error(
                            step_type,
                            format!(
                                "输入变量[{}]里未找到滑动文字目标: {}",
                                input_var,
                                target_text.unwrap_or_default()
                            ),
                        )
                    })?;
                let point = Self::bounding_box_center_to_point(
                    step_type,
                    "滑动文字目标",
                    &item.bounding_box,
                )?;
                Ok((point, ActionTraceBuilder::build_ocr_target(role, point, &item)))
            }
            SwipeTarget::LabelIdx { input_var, idx } => {
                let item = self
                    .resolve_det_target_items(step_type, input_var, Some(u32::from(*idx)))
                    .await?
                    .into_iter()
                    .next()
                    .ok_or_else(|| {
                        Self::execute_error(
                            step_type,
                            format!("输入变量[{}]里未找到滑动标签目标: {}，请检查标签是否输入有误！", input_var, idx),
                        )
                    })?;
                let point = Self::bounding_box_center_to_point(
                    step_type,
                    "滑动标签目标",
                    &item.bounding_box,
                )?;
                Ok((point, ActionTraceBuilder::build_det_target(role, point, &item)))
            }
        }
    }

    async fn resolve_ocr_target_items(
        &self,
        step_type: &str,
        input_var: &str,
        target_text: Option<&str>,
    ) -> ExecuteResult<Vec<OcrResult>> {
        if let Ok(hits) = self
            .read_runtime_result_vec::<SearchHit>(input_var, step_type, "SearchHit")
            .await
        {
            let items = Self::collect_ocr_results_from_hits(&hits);
            return Ok(Self::select_ocr_results(&items, target_text)
                .into_iter()
                .cloned()
                .collect());
        }

        let items = self
            .read_runtime_result_vec::<OcrResult>(input_var, step_type, "OCR")
            .await?;
        Ok(Self::select_ocr_results(&items, target_text)
            .into_iter()
            .cloned()
            .collect())
    }

    async fn resolve_det_target_items(
        &self,
        step_type: &str,
        input_var: &str,
        target_idx: Option<u32>,
    ) -> ExecuteResult<Vec<DetResult>> {
        let items = self
            .read_runtime_result_vec::<DetResult>(input_var, step_type, "检测")
            .await?;
        Ok(Self::select_det_results(&items, target_idx)
            .into_iter()
            .cloned()
            .collect())
    }
}
