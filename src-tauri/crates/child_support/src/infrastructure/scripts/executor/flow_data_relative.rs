impl ScriptExecutor {
    async fn execute_relative_filter_step(
        &mut self,
        input_var: &str,
        out_var: &str,
        anchor_type: &RelativeAnchorType,
        anchor_text: &str,
        anchor_idx: i32,
        direction: &RelativeDirection,
        target_kind: &RelativeTargetKind,
        max_offset_x: Option<i32>,
        max_offset_y: Option<i32>,
        target_index: Option<usize>,
        then_steps: &[Step],
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence(
                "data.relativeFilter",
                format!("RelativeFilter 筛选输入变量 {} 到 {}", input_var, out_var),
            )
            .await?
        {
            return Ok(timeout_flow);
        }
        let Some(input) = self.read_runtime_var(input_var).await else {
            self.set_runtime_var(out_var, Dynamic::from(Array::new()))
                .await?;
            return Ok(ControlFlow::Next);
        };

        let output = {
            let ctx = self.runtime_ctx.read().await;
            if let Some(snapshot) = ctx.observation.last_snapshot.as_ref() {
                match target_kind {
                    RelativeTargetKind::DetLabel => {
                        let items =
                            Self::deserialize_dynamic_value::<Vec<DetResult>>(&input).map_err(
                                |error| {
                                    Self::execute_error(
                                        "data.relativeFilter",
                                        format!(
                                            "输入变量[{}]不是兼容的检测结果集: {}",
                                            input_var, error
                                        ),
                                    )
                                },
                            )?;
                        let matched = Self::relative_filter_det_items(
                            snapshot,
                            &items,
                            anchor_type,
                            anchor_text,
                            anchor_idx,
                            direction,
                            max_offset_x,
                            max_offset_y,
                            target_index,
                        );
                        to_dynamic(matched).map_err(|error| {
                            Self::execute_error(
                                "data.relativeFilter",
                                format!("相对位置检测结果写入变量失败: {}", error),
                            )
                        })?
                    }
                    RelativeTargetKind::OcrText | RelativeTargetKind::Any => {
                        let items =
                            Self::deserialize_dynamic_value::<Vec<OcrResult>>(&input).map_err(
                                |error| {
                                    Self::execute_error(
                                        "data.relativeFilter",
                                        format!(
                                            "输入变量[{}]不是兼容的 OCR 结果集: {}",
                                            input_var, error
                                        ),
                                    )
                                },
                            )?;
                        let matched = Self::relative_filter_ocr_items(
                            snapshot,
                            &items,
                            anchor_type,
                            anchor_text,
                            anchor_idx,
                            direction,
                            max_offset_x,
                            max_offset_y,
                            target_index,
                        );
                        to_dynamic(matched).map_err(|error| {
                            Self::execute_error(
                                "data.relativeFilter",
                                format!("相对位置 OCR 结果写入变量失败: {}", error),
                            )
                        })?
                    }
                }
            } else {
                Dynamic::from(Array::new())
            }
        };

        let has_output = output.clone().try_cast::<Array>().is_some_and(|items| !items.is_empty());
        self.set_runtime_var(out_var, output).await?;
        if has_output && !then_steps.is_empty() {
            return self.execute(then_steps).await;
        }
        Ok(ControlFlow::Next)
    }

    fn relative_filter_ocr_items(
        snapshot: &VisionSnapshot,
        items: &[OcrResult],
        anchor_type: &RelativeAnchorType,
        anchor_text: &str,
        anchor_idx: i32,
        direction: &RelativeDirection,
        max_offset_x: Option<i32>,
        max_offset_y: Option<i32>,
        target_index: Option<usize>,
    ) -> Vec<OcrResult> {
        let anchors = Self::relative_anchors(snapshot, anchor_type, anchor_text, anchor_idx);
        let mut output = Vec::new();
        for anchor in anchors {
            let mut candidates: Vec<_> = snapshot
                .layout_items
                .iter()
                .filter(|candidate| candidate.source == VisionLayoutSource::Ocr)
                .filter(|candidate| {
                    snapshot
                        .ocr_items
                        .get(candidate.item_index)
                        .is_some_and(|ocr| items.contains(ocr))
                })
                .filter_map(|candidate| {
                    Self::relative_filter_score(
                        anchor,
                        candidate,
                        direction,
                        max_offset_x,
                        max_offset_y,
                    )
                    .map(|score| (score, candidate))
                })
                .collect();
            candidates.sort_by(|left, right| left.0.cmp(&right.0));
            let selected = candidates.get(target_index.unwrap_or(0)).map(|(_, item)| *item);
            if let Some(item) = selected.and_then(|item| snapshot.ocr_items.get(item.item_index)) {
                if !output.contains(item) {
                    output.push(item.clone());
                }
            }
        }
        output
    }

    fn relative_filter_det_items(
        snapshot: &VisionSnapshot,
        items: &[DetResult],
        anchor_type: &RelativeAnchorType,
        anchor_text: &str,
        anchor_idx: i32,
        direction: &RelativeDirection,
        max_offset_x: Option<i32>,
        max_offset_y: Option<i32>,
        target_index: Option<usize>,
    ) -> Vec<DetResult> {
        let anchors = Self::relative_anchors(snapshot, anchor_type, anchor_text, anchor_idx);
        let mut output = Vec::new();
        for anchor in anchors {
            let mut candidates: Vec<_> = snapshot
                .layout_items
                .iter()
                .filter(|candidate| candidate.source == VisionLayoutSource::Det)
                .filter(|candidate| {
                    snapshot
                        .det_items
                        .get(candidate.item_index)
                        .is_some_and(|det| items.contains(det))
                })
                .filter_map(|candidate| {
                    Self::relative_filter_score(
                        anchor,
                        candidate,
                        direction,
                        max_offset_x,
                        max_offset_y,
                    )
                    .map(|score| (score, candidate))
                })
                .collect();
            candidates.sort_by(|left, right| left.0.cmp(&right.0));
            let selected = candidates.get(target_index.unwrap_or(0)).map(|(_, item)| *item);
            if let Some(item) = selected.and_then(|item| snapshot.det_items.get(item.item_index)) {
                if !output.contains(item) {
                    output.push(item.clone());
                }
            }
        }
        output
    }

    fn relative_anchors<'a>(
        snapshot: &'a VisionSnapshot,
        anchor_type: &RelativeAnchorType,
        anchor_text: &str,
        anchor_idx: i32,
    ) -> Vec<&'a VisionLayoutItem> {
        snapshot
            .layout_items
            .iter()
            .filter(|item| match anchor_type {
                RelativeAnchorType::OcrText => {
                    item.source == VisionLayoutSource::Ocr
                        && item
                            .text
                            .as_deref()
                            .is_some_and(|text| text.contains(anchor_text))
                }
                RelativeAnchorType::DetLabel => {
                    item.source == VisionLayoutSource::Det && item.label_index == Some(anchor_idx)
                }
            })
            .collect()
    }

    fn relative_filter_score(
        anchor: &VisionLayoutItem,
        candidate: &VisionLayoutItem,
        direction: &RelativeDirection,
        max_offset_x: Option<i32>,
        max_offset_y: Option<i32>,
    ) -> Option<(i32, i32)> {
        if candidate.item_index == anchor.item_index && candidate.source == anchor.source {
            return None;
        }
        let dx = candidate.stable_center.x - anchor.stable_center.x;
        let dy = candidate.stable_center.y - anchor.stable_center.y;
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        if max_offset_x.is_some_and(|max| abs_dx > max.max(0)) {
            return None;
        }
        if max_offset_y.is_some_and(|max| abs_dy > max.max(0)) {
            return None;
        }
        match direction {
            RelativeDirection::Right if dx > 0 => Some((dx, abs_dy)),
            RelativeDirection::Left if dx < 0 => Some((abs_dx, abs_dy)),
            RelativeDirection::Below if dy > 0 => Some((dy, abs_dx)),
            RelativeDirection::Above if dy < 0 => Some((abs_dy, abs_dx)),
            RelativeDirection::LeftAbove if dx < 0 && dy < 0 => {
                Some((abs_dx + abs_dy, abs_dx))
            }
            RelativeDirection::RightAbove if dx > 0 && dy < 0 => {
                Some((abs_dx + abs_dy, abs_dx))
            }
            RelativeDirection::RightBelow if dx > 0 && dy > 0 => {
                Some((abs_dx + abs_dy, abs_dx))
            }
            RelativeDirection::LeftBelow if dx < 0 && dy > 0 => {
                Some((abs_dx + abs_dy, abs_dx))
            }
            RelativeDirection::Near => Some((abs_dx + abs_dy, abs_dx.min(abs_dy))),
            _ => None,
        }
    }
}
