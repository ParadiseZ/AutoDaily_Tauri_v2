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
                        "vision.inference",
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
                        "vision.inference",
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
                det_res_var,
                ocr_res_var,
                rule,
                out_var,
                out_det_var,
                out_ocr_var,
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
                let (hits, filtered_det_results, filtered_ocr_results, matched) = self
                    .execute_vision_search_step(
                        "vision.search",
                        det_res_var.as_deref(),
                        ocr_res_var.as_deref(),
                        rule,
                    )
                    .await?;

                {
                    let mut ctx = self.runtime_ctx.write().await;
                    ctx.observation.last_hits = hits.clone();
                }

                self.set_runtime_var(
                    out_var,
                    to_dynamic(&hits).map_err(|error| {
                        Self::execute_error(
                            "vision.search",
                            format!("序列化 SearchHit 结果失败: {}", error),
                        )
                    })?,
                )
                    .await?;
                if let Some(out_det_var) = out_det_var.as_deref().filter(|value| !value.trim().is_empty()) {
                    self.set_runtime_var(
                        out_det_var,
                        Self::results_to_dynamic("vision.search", "检测", &filtered_det_results)?,
                    )
                    .await?;
                }
                if let Some(out_ocr_var) = out_ocr_var.as_deref().filter(|value| !value.trim().is_empty()) {
                    self.set_runtime_var(
                        out_ocr_var,
                        Self::results_to_dynamic("vision.search", "OCR", &filtered_ocr_results)?,
                    )
                    .await?;
                }

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
        let matched = self
            .match_vision_count_compare(step_type, input_var, target_value, op, expected_count)
            .await?;
        if !out_var.trim().is_empty() {
            self.set_runtime_var(out_var, Dynamic::from_bool(matched)).await?;
        }
        Ok(matched)
    }

    async fn match_vision_count_compare(
        &self,
        step_type: &str,
        input_var: &str,
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
        Ok(Self::compare_dynamic(
            &Dynamic::from_int(actual_count.into()),
            op,
            &Dynamic::from_int(expected_count.into()),
        ))
    }

    async fn execute_vision_search_step(
        &self,
        step_type: &str,
        det_res_var: Option<&str>,
        ocr_res_var: Option<&str>,
        rule: &SearchRule,
    ) -> ExecuteResult<(Vec<SearchHit>, Vec<DetResult>, Vec<OcrResult>, bool)> {
        let (default_ocr, default_det) = {
            let ctx = self.runtime_ctx.read().await;
            (
                ctx.observation
                    .last_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.ocr_items.clone())
                    .unwrap_or_default(),
                ctx.observation
                    .last_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.det_items.clone())
                    .unwrap_or_default(),
            )
        };
        let ocr_results = match ocr_res_var.map(str::trim).filter(|value| !value.is_empty()) {
            Some(input_var) => self
                .read_runtime_result_vec::<OcrResult>(input_var, step_type, "OCR")
                .await?,
            None => default_ocr,
        };
        let det_results = match det_res_var.map(str::trim).filter(|value| !value.is_empty()) {
            Some(input_var) => self
                .read_runtime_result_vec::<DetResult>(input_var, step_type, "检测")
                .await?,
            None => default_det,
        };
        let searcher = OcrSearcher::new(std::slice::from_ref(rule));
        let hits = searcher.search_ocr_items(&ocr_results);
        let matched = rule.evaluate(&hits, &det_results);
        Ok((
            hits.clone(),
            Self::collect_det_results_by_rule(rule, &det_results),
            Self::collect_ocr_results_from_hits(&hits),
            matched,
        ))
    }

    fn collect_ocr_results_from_hits(hits: &[SearchHit]) -> Vec<OcrResult> {
        let mut seen = std::collections::HashSet::new();
        let mut output = Vec::new();
        for hit in hits {
            if seen.insert(hit.ocr_index) {
                output.push(hit.ocr_item.clone());
            }
        }
        output
    }

    fn collect_det_results_by_rule(rule: &SearchRule, det_results: &[DetResult]) -> Vec<DetResult> {
        let mut indices = Vec::new();
        Self::collect_det_label_indices(rule, &mut indices);
        if indices.is_empty() {
            return Vec::new();
        }
        det_results
            .iter()
            .filter(|item| indices.contains(&item.index))
            .cloned()
            .collect()
    }

    fn collect_det_label_indices(rule: &SearchRule, bucket: &mut Vec<i32>) {
        match rule {
            SearchRule::DetLabel { idx } => {
                if !bucket.contains(idx) {
                    bucket.push(*idx);
                }
            }
            SearchRule::Group { items, .. } => {
                for item in items {
                    Self::collect_det_label_indices(item, bucket);
                }
            }
            SearchRule::Txt { .. } => {}
        }
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
