impl ScriptExecutor {
    async fn execute_task_control_step(
        &mut self,
        task_control: &TaskControl,
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence("taskControl.setState", "TaskControl 状态写入")
            .await?
        {
            return Ok(timeout_flow);
        }
        match task_control {
            TaskControl::SetState {
                target,
                targets,
                status,
            } => {
                if targets.is_empty() {
                    self.set_state_value(target, status).await?;
                } else {
                    for target in targets {
                        self.set_state_value(target, status).await?;
                    }
                }
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_vision_step(&mut self, vision: &VisionNode) -> ExecuteResult<ControlFlow> {
        match vision {
            VisionNode::Detect { input_var, out_var } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "vision.detect",
                        format!("Vision Detect {} -> {}", input_var, out_var),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                self.execute_detect_step("vision.detect", input_var, out_var)
                    .await?;
                Ok(ControlFlow::Next)
            }
            VisionNode::Ocr { input_var, out_var } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "vision.ocr",
                        format!("Vision OCR {} -> {}", input_var, out_var),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                self.execute_ocr_step("vision.ocr", input_var, out_var).await?;
                Ok(ControlFlow::Next)
            }
            VisionNode::CountCompare {
                input_var,
                out_var,
                target_value,
                op,
                expected_count,
                then_steps,
            } => {
                let matched = self
                    .execute_vision_count_compare_step(
                        "vision.countCompare",
                        input_var,
                        out_var,
                        target_value.as_deref(),
                        op,
                        *expected_count,
                    )
                    .await?;
                if matched && !then_steps.is_empty() {
                    return self.execute(then_steps).await;
                }
                Ok(ControlFlow::Next)
            }
            VisionNode::VisionSearch {
                rule,
                out_var,
                then_steps,
            } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "vision.search",
                        format!("VisionSearch 搜索并写入 {}", out_var),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
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

    async fn execute_vision_count_compare_step(
        &mut self,
        step_type: &str,
        input_var: &str,
        out_var: &str,
        target_value: Option<&str>,
        op: &CompareOp,
        expected_count: i32,
    ) -> ExecuteResult<bool> {
        let value = self.read_runtime_var(input_var).await.ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!("输入变量[{}]不存在，无法统计视觉结果数量", input_var),
            )
        })?;
        let target_value = target_value
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let actual_count = if let Ok(items) = Self::deserialize_dynamic_value::<Vec<DetResult>>(&value)
        {
            Self::count_det_items(&items, target_value)
        } else if let Ok(items) = Self::deserialize_dynamic_value::<Vec<OcrResult>>(&value) {
            Self::count_ocr_items(&items, target_value)
        } else {
            return Err(Self::execute_error(
                step_type,
                format!("输入变量[{}]不是兼容的检测结果或 OCR 结果集", input_var),
            ));
        };
        let matched = Self::compare_dynamic(
            &Dynamic::from_int(actual_count.into()),
            op,
            &Dynamic::from_int(expected_count.into()),
        );
        self.set_runtime_var(out_var, Dynamic::from_bool(matched)).await?;
        Ok(matched)
    }

    fn count_det_items(items: &[DetResult], target_value: Option<&str>) -> i32 {
        let Some(target_value) = target_value else {
            return items.len() as i32;
        };
        items.iter()
            .filter(|item| item.label.trim() == target_value || item.label.contains(target_value))
            .count() as i32
    }

    fn count_ocr_items(items: &[OcrResult], target_value: Option<&str>) -> i32 {
        let Some(target_value) = target_value else {
            return items.len() as i32;
        };
        let exact_count = items
            .iter()
            .filter(|item| item.txt.trim() == target_value)
            .count() as i32;
        if exact_count > 0 {
            return exact_count;
        }
        items.iter()
            .filter(|item| item.txt.contains(target_value))
            .count() as i32
    }
}
