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
                    }
                }

                Ok(ControlFlow::Next)
            }
            FlowControl::Continue => Ok(ControlFlow::Continue),
            FlowControl::Break => Ok(ControlFlow::Break),
            FlowControl::WaitMs { ms } => {
                if let Some(timeout_flow) = self
                    .sleep_with_progress_timeout(
                        *ms,
                        "flow.waitMs",
                        format!("WaitMs 等待 {}ms", ms),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                Ok(ControlFlow::Next)
            }
            FlowControl::Link { target } => Ok(ControlFlow::Link(*target)),
            FlowControl::AddPolicies { source, target } => {
                self.add_policy_overlay(*source, *target).await;
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

    async fn execute_data_handling_step(
        &mut self,
        data: &DataHanding,
    ) -> ExecuteResult<ControlFlow> {
        match data {
            DataHanding::SetVar { name, val, expr } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "data.setVar",
                        format!("SetVar 写入变量 {}", name),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                let value =
                    if let Some(expr) = expr.as_ref().filter(|value| !value.trim().is_empty()) {
                        self.eval_dynamic(expr, "data.setVar")?
                    } else if let Some(val) = val {
                        Self::var_value_to_dynamic(val)
                    } else {
                        Dynamic::UNIT
                    };
                self.set_runtime_var(name, value).await?;
                Ok(ControlFlow::Next)
            }
            DataHanding::GetVar { name, default_val } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "data.getVar",
                        format!("GetVar 读取变量 {}", name),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                if self.read_runtime_var(name).await.is_none() {
                    if let Some(default_val) = default_val {
                        self.set_runtime_var(name, Self::var_value_to_dynamic(default_val))
                            .await?;
                    }
                }
                Ok(ControlFlow::Next)
            }
            DataHanding::Filter {
                input_var,
                out_name,
                mode,
                logic_expr,
                then_steps,
            } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "data.filter",
                        format!("Filter 准备处理输入变量 {}", input_var),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                let Some(input) = self.read_runtime_var(input_var).await else {
                    self.set_runtime_var(out_name, Dynamic::from(Array::new()))
                        .await?;
                    return Ok(ControlFlow::Next);
                };

                let Some(items) = input.clone().try_cast::<Array>() else {
                    return Err(Self::execute_error(
                        "data.filter",
                        format!("输入变量[{}]不是数组，无法执行过滤", input_var),
                    ));
                };

                let mut output = Array::new();
                for (index, item) in items.into_iter().enumerate() {
                    if let Some(timeout_flow) = self
                        .record_progress_evidence(
                            "data.filter.item",
                            format!("Filter 处理条目 {} -> {}", input_var, index),
                        )
                        .await?
                    {
                        return Ok(timeout_flow);
                    }
                    self.scope.set_value(FILTER_ITEM_VAR, item.clone());
                    self.scope.set_value(ITEM_VAR, item.clone());
                    self.scope.set_value(FILTER_INDEX_VAR, index as i64);
                    self.scope.set_value(ITEM_INDEX_VAR, index as i64);

                    let matched = if logic_expr.trim().is_empty() {
                        true
                    } else {
                        self.eval_bool(logic_expr, "data.filter.logicExpr")?
                    };

                    if !matched {
                        continue;
                    }

                    if !then_steps.is_empty() {
                        match self.execute(then_steps).await? {
                            ControlFlow::Next => {}
                            ControlFlow::Continue => continue,
                            ControlFlow::Break => break,
                            ControlFlow::Link(target) => return Ok(ControlFlow::Link(target)),
                            ControlFlow::Return => return Ok(ControlFlow::Return),
                        }
                    }

                    match mode {
                        FilterMode::Filter => output.push(item),
                        FilterMode::Map => {
                            let current = self
                                .scope
                                .get_value::<Dynamic>(ITEM_VAR)
                                .unwrap_or_else(|| Dynamic::UNIT);
                            output.push(current);
                        }
                    }
                }

                self.set_runtime_var(out_name, Dynamic::from(output))
                    .await?;
                Ok(ControlFlow::Next)
            }
            DataHanding::ColorCompare {
                input_var,
                out_var,
                target_text,
                is_font,
                target_color,
                method,
            } => {
                self.execute_color_compare_step(
                    input_var,
                    out_var,
                    target_text.as_deref(),
                    *is_font,
                    target_color,
                    method,
                )
                .await
            }
        }
    }

    async fn execute_color_compare_step(
        &mut self,
        input_var: &str,
        out_var: &str,
        target_text: Option<&str>,
        is_font: bool,
        target_color: &ColorRgb,
        method: &ColorCompareMethod,
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence(
                "data.colorCompare",
                format!(
                    "ColorCompare 比较输入变量 {} 到输出变量 {}",
                    input_var, out_var
                ),
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

        let items = Self::deserialize_dynamic_value::<Vec<OcrResult>>(&input).map_err(|error| {
            Self::execute_error(
                "data.colorCompare",
                format!("输入变量[{}]不是兼容的 OCR 结果集: {}", input_var, error),
            )
        })?;
        let capture = self.require_last_capture_image("data.colorCompare").await?;
        let target_lab = Self::rgb_to_oklab(target_color);
        let candidates = Self::filter_ocr_items_for_color_compare(&items, target_text);
        let matched: Vec<OcrResult> = candidates
            .into_iter()
            .filter(|item| {
                Self::ocr_item_matches_color(
                    capture.as_ref(),
                    item,
                    is_font,
                    target_lab,
                    method,
                )
            })
            .cloned()
            .collect();

        self.set_runtime_var(
            out_var,
            to_dynamic(matched).map_err(|error| {
                Self::execute_error(
                    "data.colorCompare",
                    format!("颜色比较结果写入变量失败: {}", error),
                )
            })?,
        )
        .await?;
        Ok(ControlFlow::Next)
    }

    async fn require_last_capture_image(&self, step_type: &str) -> ExecuteResult<Arc<RgbaImage>> {
        let ctx = self.runtime_ctx.read().await;
        ctx.observation
            .last_capture_image
            .clone()
            .ok_or_else(|| {
                Self::execute_error(
                    step_type,
                    "当前运行时没有可用截图，请先执行 Capture 或激活图像上下文".to_string(),
                )
            })
    }

    fn filter_ocr_items_for_color_compare<'a>(
        items: &'a [OcrResult],
        target_text: Option<&str>,
    ) -> Vec<&'a OcrResult> {
        let target_text = target_text
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let Some(target_text) = target_text else {
            return items.iter().collect();
        };

        let exact: Vec<_> = items.iter().filter(|item| item.txt.trim() == target_text).collect();
        if !exact.is_empty() {
            return exact;
        }

        items.iter()
            .filter(|item| item.txt.contains(target_text))
            .collect()
    }

    fn ocr_item_matches_color(
        image: &RgbaImage,
        item: &OcrResult,
        is_font: bool,
        target_lab: OklabColor,
        method: &ColorCompareMethod,
    ) -> bool {
        let Some(inner_rect) = Self::clamp_bbox_to_image(&item.bounding_box, image) else {
            return false;
        };
        let ring_rect = Self::expand_rect_for_ring(&inner_rect, image.width(), image.height());
        let threshold = Self::color_compare_threshold(method);

        if is_font {
            let candidate_colors =
                Self::extract_font_candidate_colors(image, inner_rect, ring_rect);
            return candidate_colors.into_iter().any(|candidate| {
                Self::oklab_distance(candidate, target_lab) <= threshold
            });
        }

        Self::extract_background_color(image, inner_rect, ring_rect)
            .map(|candidate| Self::oklab_distance(candidate, target_lab) <= threshold)
            .unwrap_or(false)
    }

    fn color_compare_threshold(method: &ColorCompareMethod) -> f32 {
        match method {
            ColorCompareMethod::OklabDistance { threshold } => threshold.max(0.0),
        }
    }

    fn extract_font_candidate_colors(
        image: &RgbaImage,
        inner_rect: BoundingBox,
        ring_rect: BoundingBox,
    ) -> Vec<OklabColor> {
        let inner_pixels = Self::collect_rect_pixels(image, &inner_rect);
        if inner_pixels.is_empty() {
            return Vec::new();
        }

        let background_color = Self::extract_background_color(image, inner_rect, ring_rect);
        let filtered_pixels = if let Some(background_color) = background_color {
            let separated: Vec<_> = inner_pixels
                .iter()
                .copied()
                .filter(|pixel| Self::oklab_distance(*pixel, background_color) >= 0.05)
                .collect();
            if separated.len() >= 12 {
                separated
            } else {
                inner_pixels.clone()
            }
        } else {
            inner_pixels.clone()
        };

        Self::cluster_colors(&filtered_pixels, 3)
            .into_iter()
            .take(3)
            .map(|cluster| cluster.center)
            .collect()
    }

    fn extract_background_color(
        image: &RgbaImage,
        inner_rect: BoundingBox,
        ring_rect: BoundingBox,
    ) -> Option<OklabColor> {
        let ring_pixels = Self::collect_ring_pixels(image, ring_rect, inner_rect);
        if ring_pixels.is_empty() {
            return None;
        }

        Self::cluster_colors(&ring_pixels, 3)
            .into_iter()
            .next()
            .map(|cluster| cluster.center)
    }

    fn clamp_bbox_to_image(bbox: &BoundingBox, image: &RgbaImage) -> Option<BoundingBox> {
        let max_x = image.width().saturating_sub(1) as i32;
        let max_y = image.height().saturating_sub(1) as i32;
        let x1 = bbox.x1.clamp(0, max_x);
        let y1 = bbox.y1.clamp(0, max_y);
        let x2 = bbox.x2.clamp(0, max_x);
        let y2 = bbox.y2.clamp(0, max_y);
        (x1 < x2 && y1 < y2).then_some(BoundingBox::new(x1, y1, x2, y2))
    }

    fn expand_rect_for_ring(rect: &BoundingBox, image_width: u32, image_height: u32) -> BoundingBox {
        let width = (rect.x2 - rect.x1).max(1);
        let height = (rect.y2 - rect.y1).max(1);
        let pad_x = ((width as f32 * 0.15).round() as i32).max(2);
        let pad_y = ((height as f32 * 0.15).round() as i32).max(2);
        BoundingBox::new(
            (rect.x1 - pad_x).clamp(0, image_width.saturating_sub(1) as i32),
            (rect.y1 - pad_y).clamp(0, image_height.saturating_sub(1) as i32),
            (rect.x2 + pad_x).clamp(0, image_width.saturating_sub(1) as i32),
            (rect.y2 + pad_y).clamp(0, image_height.saturating_sub(1) as i32),
        )
    }

    fn collect_rect_pixels(image: &RgbaImage, rect: &BoundingBox) -> Vec<OklabColor> {
        let mut pixels = Vec::new();
        for y in rect.y1..=rect.y2 {
            for x in rect.x1..=rect.x2 {
                let pixel = image.get_pixel(x as u32, y as u32);
                pixels.push(Self::rgb_to_oklab(&ColorRgb {
                    r: pixel[0],
                    g: pixel[1],
                    b: pixel[2],
                }));
            }
        }
        pixels
    }

    fn collect_ring_pixels(
        image: &RgbaImage,
        outer_rect: BoundingBox,
        inner_rect: BoundingBox,
    ) -> Vec<OklabColor> {
        let mut pixels = Vec::new();
        for y in outer_rect.y1..=outer_rect.y2 {
            for x in outer_rect.x1..=outer_rect.x2 {
                if x >= inner_rect.x1
                    && x <= inner_rect.x2
                    && y >= inner_rect.y1
                    && y <= inner_rect.y2
                {
                    continue;
                }

                let pixel = image.get_pixel(x as u32, y as u32);
                pixels.push(Self::rgb_to_oklab(&ColorRgb {
                    r: pixel[0],
                    g: pixel[1],
                    b: pixel[2],
                }));
            }
        }
        pixels
    }

    fn cluster_colors(points: &[OklabColor], limit: usize) -> Vec<ColorCluster> {
        if points.is_empty() || limit == 0 {
            return Vec::new();
        }

        let k = limit.min(points.len());
        let mut centers = Vec::with_capacity(k);
        centers.push(points[0]);
        while centers.len() < k {
            let next = points
                .iter()
                .copied()
                .max_by(|left, right| {
                    let left_distance = centers
                        .iter()
                        .map(|center| Self::oklab_distance(*left, *center))
                        .fold(f32::MAX, f32::min);
                    let right_distance = centers
                        .iter()
                        .map(|center| Self::oklab_distance(*right, *center))
                        .fold(f32::MAX, f32::min);
                    left_distance.total_cmp(&right_distance)
                })
                .unwrap_or(points[0]);
            if centers.iter().any(|center| Self::oklab_distance(*center, next) < 0.001) {
                break;
            }
            centers.push(next);
        }

        for _ in 0..8 {
            let mut groups: Vec<Vec<OklabColor>> = vec![Vec::new(); centers.len()];
            for point in points {
                let closest = centers
                    .iter()
                    .enumerate()
                    .min_by(|(_, left), (_, right)| {
                        Self::oklab_distance(*point, **left)
                            .total_cmp(&Self::oklab_distance(*point, **right))
                    })
                    .map(|(index, _)| index)
                    .unwrap_or(0);
                groups[closest].push(*point);
            }

            let mut moved = false;
            for (index, group) in groups.into_iter().enumerate() {
                if group.is_empty() {
                    continue;
                }
                let next_center = Self::mean_color(&group);
                if Self::oklab_distance(centers[index], next_center) > 0.001 {
                    centers[index] = next_center;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }

        let mut groups: Vec<Vec<OklabColor>> = vec![Vec::new(); centers.len()];
        for point in points {
            let closest = centers
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    Self::oklab_distance(*point, **left)
                        .total_cmp(&Self::oklab_distance(*point, **right))
                })
                .map(|(index, _)| index)
                .unwrap_or(0);
            groups[closest].push(*point);
        }

        let mut clusters = Vec::new();
        for (center, members) in centers.into_iter().zip(groups.into_iter()) {
            if members.is_empty() {
                continue;
            }
            let mean_distance =
                members.iter().map(|member| Self::oklab_distance(*member, center)).sum::<f32>()
                    / members.len() as f32;
            clusters.push(ColorCluster {
                center,
                count: members.len(),
                mean_distance,
            });
        }

        clusters.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.mean_distance.total_cmp(&right.mean_distance))
        });
        clusters
    }

    fn mean_color(points: &[OklabColor]) -> OklabColor {
        let mut sum = OklabColor::default();
        for point in points {
            sum.l += point.l;
            sum.a += point.a;
            sum.b += point.b;
        }
        let count = points.len() as f32;
        OklabColor {
            l: sum.l / count,
            a: sum.a / count,
            b: sum.b / count,
        }
    }

    fn oklab_distance(left: OklabColor, right: OklabColor) -> f32 {
        let dl = left.l - right.l;
        let da = left.a - right.a;
        let db = left.b - right.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    fn rgb_to_oklab(color: &ColorRgb) -> OklabColor {
        let srgb = |value: u8| value as f32 / 255.0;
        let to_linear = |value: f32| {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };

        let r = to_linear(srgb(color.r));
        let g = to_linear(srgb(color.g));
        let b = to_linear(srgb(color.b));

        let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

        let l_cbrt = l.cbrt();
        let m_cbrt = m.cbrt();
        let s_cbrt = s.cbrt();

        OklabColor {
            l: 0.210_454_26 * l_cbrt + 0.793_617_8 * m_cbrt - 0.004_072_047 * s_cbrt,
            a: 1.977_998_5 * l_cbrt - 2.428_592_2 * m_cbrt + 0.450_593_7 * s_cbrt,
            b: 0.025_904_037 * l_cbrt + 0.782_771_77 * m_cbrt - 0.808_675_77 * s_cbrt,
        }
    }

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

    fn evaluate_condition<'a>(
        &'a mut self,
        condition: &'a ConditionNode,
    ) -> Pin<Box<dyn Future<Output = ExecuteResult<bool>> + 'a>> {
        Box::pin(async move {
            match condition {
                ConditionNode::RawExpr { expr } => self.eval_bool(expr, "condition.rawExpr"),
                ConditionNode::Group { op, items } => match op {
                    crate::domain::vision::ocr_search::LogicOp::And => {
                        for item in items {
                            if !self.evaluate_condition(item).await? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    crate::domain::vision::ocr_search::LogicOp::Or => {
                        for item in items {
                            if self.evaluate_condition(item).await? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }
                    crate::domain::vision::ocr_search::LogicOp::Not => {
                        if let Some(first) = items.first() {
                            Ok(!self.evaluate_condition(first).await?)
                        } else {
                            Ok(true)
                        }
                    }
                },
                ConditionNode::VarCompare {
                    var_name,
                    op,
                    value,
                } => {
                    let Some(lhs) = self.read_runtime_var(var_name).await else {
                        return Ok(false);
                    };
                    let rhs = Self::var_value_to_dynamic(value);
                    Ok(Self::compare_dynamic(&lhs, op, &rhs))
                }
                ConditionNode::TaskStatus { a } => self.match_state_status(a).await,
                ConditionNode::CurrentTaskIn { targets } => Ok(self.current_task_in(targets).await),
                ConditionNode::PolicyCondition { input_var, rule } => {
                    if let Some(input_var) =
                        input_var.as_deref().map(str::trim).filter(|value| !value.is_empty())
                    {
                        if let Err(error) = self
                            .activate_image_var("condition.policyCondition", input_var)
                            .await
                        {
                            Log::debug(&format!(
                                "[ executor ] PolicyCondition 输入图像不可用，按 false 处理: {}",
                                error
                            ));
                            return Ok(false);
                        }
                    }

                    let ctx = self.runtime_ctx.read().await;
                    if let Some(snapshot) = ctx.observation.last_snapshot.as_ref() {
                        Ok(rule.evaluate(snapshot))
                    } else {
                        Log::debug("[ executor ] PolicyCondition 未找到可用视觉快照，按 false 处理");
                        Ok(false)
                    }
                }
                ConditionNode::ExecNumCompare { target, op } => {
                    self.match_exec_num_compare(target, op).await
                }
                ConditionNode::ColorCompare { .. } => Err(Self::execute_error(
                    "condition.colorCompare",
                    "颜色比较尚未接入视觉颜色分析".to_string(),
                )),
                ConditionNode::PolicySetResult {
                    result_var,
                    field,
                    op,
                    value_bool,
                    value_id,
                } => self
                    .match_policy_set_result(
                        result_var,
                        field,
                        op,
                        *value_bool,
                        value_id.as_str(),
                    )
                    .await,
            }
        })
    }

    async fn match_policy_set_result(
        &self,
        result_var: &str,
        field: &PolicySetResultField,
        op: &PolicySetResultCompareOp,
        value_bool: bool,
        value_id: &str,
    ) -> ExecuteResult<bool> {
        let Some(value) = self.read_runtime_var(result_var).await else {
            return Ok(false);
        };
        let result = Self::deserialize_dynamic_value::<PolicyExecutionResult>(&value).map_err(
            |error| {
                Self::execute_error(
                    "condition.policySetResult",
                    format!("变量[{}]不是兼容的策略执行结果: {}", result_var, error),
                )
            },
        )?;

        Ok(match field {
            PolicySetResultField::Matched => Self::compare_bool(result.matched, op, value_bool),
            PolicySetResultField::PolicySetId => {
                Self::compare_optional_id(result.policy_set_id, op, value_id)
            }
            PolicySetResultField::PolicyGroupId => {
                Self::compare_optional_id(result.policy_group_id, op, value_id)
            }
            PolicySetResultField::PolicyId => {
                Self::compare_optional_id(result.policy_id, op, value_id)
            }
        })
    }

    async fn set_state_value(
        &mut self,
        target: &StateTarget,
        status: &StateStatus,
    ) -> ExecuteResult<()> {
        let mut ctx = self.runtime_ctx.write().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx.execution.task_states.entry(*id).or_default();
                match status {
                    StateStatus::Enabled { value } => state.enabled_flag = *value,
                    StateStatus::Skip { value } => state.skip_flag = *value,
                    StateStatus::Done { value } => state.done_flag = *value,
                }
            }
            StateTarget::Policy { id } => {
                let state = ctx.execution.policy_states.entry(*id).or_default();
                match status {
                    StateStatus::Enabled { .. } => {
                        return Err(Self::execute_error(
                            "taskControl.setState",
                            format!("策略[{}]不支持 enabled 状态", id),
                        ));
                    }
                    StateStatus::Skip { value } => state.skip_flag = *value,
                    StateStatus::Done { value } => state.done_flag = *value,
                }
            }
        }
        Ok(())
    }

    async fn match_state_status(&mut self, task_control: &TaskControl) -> ExecuteResult<bool> {
        let (target, targets, status) = match task_control {
            TaskControl::SetState {
                target,
                targets,
                status,
            } => (target, targets, status),
        };

        if targets.is_empty() {
            return Ok(self.match_state_value(target, status).await);
        }

        for target in targets {
            if !self.match_state_value(target, status).await {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn current_task_in(&self, targets: &[TaskId]) -> bool {
        if targets.is_empty() {
            return false;
        }

        let ctx = self.runtime_ctx.read().await;
        ctx.execution
            .current_task
            .as_ref()
            .is_some_and(|task| targets.contains(&task.id))
    }

    async fn match_state_value(&self, target: &StateTarget, status: &StateStatus) -> bool {
        let ctx = self.runtime_ctx.read().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx
                    .execution
                    .task_states
                    .get(id)
                    .cloned()
                    .unwrap_or_else(TaskState::default);
                match status {
                    StateStatus::Enabled { value } => state.enabled_flag == *value,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                }
            }
            StateTarget::Policy { id } => {
                let state = ctx
                    .execution
                    .policy_states
                    .get(id)
                    .cloned()
                    .unwrap_or_default();
                match status {
                    StateStatus::Enabled { .. } => false,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                }
            }
        }
    }

    async fn add_policy_overlay(&self, source: PolicySetId, target: PolicySetId) {
        let mut ctx = self.runtime_ctx.write().await;
        let entry = ctx.execution.policy_set_overlays.entry(target).or_default();
        if !entry.contains(&source) {
            entry.push(source);
        }
    }

    async fn match_exec_num_compare(
        &self,
        target: &StateTarget,
        op: &CompareOp,
    ) -> ExecuteResult<bool> {
        let exec_cur = self.current_exec_count(target).await;
        let exec_max = self.resolve_exec_limit(target).await?;
        Ok(Self::compare_exec_count(exec_cur, op, exec_max))
    }

    async fn current_exec_count(&self, target: &StateTarget) -> u32 {
        let ctx = self.runtime_ctx.read().await;
        match target {
            StateTarget::Task { id } => ctx
                .execution
                .task_states
                .get(id)
                .map(|state| state.exec_cur)
                .unwrap_or(0),
            StateTarget::Policy { id } => ctx
                .execution
                .policy_states
                .get(id)
                .map(|state| state.exec_cur)
                .unwrap_or(0),
        }
    }

    async fn resolve_exec_limit(&self, target: &StateTarget) -> ExecuteResult<Option<u32>> {
        let script_id = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.script_id
        };
        let snapshot = get_script_bundle_snapshot(script_id).await.ok_or_else(|| {
            Self::execute_error(
                "condition.execNumCompare",
                format!("当前 session 中不存在脚本[{}]的 bundle", script_id),
            )
        })?;

        match target {
            StateTarget::Task { id } => {
                let tasks: Vec<ScriptTaskTable> =
                    Self::parse_bundle_json("condition.execNumCompare", "tasks_json", &snapshot.tasks_json)?;
                let task = tasks.into_iter().find(|task| task.id == *id).ok_or_else(|| {
                    Self::execute_error(
                        "condition.execNumCompare",
                        format!("目标任务[{}]不存在", id),
                    )
                })?;
                Ok((task.exec_max > 0).then_some(task.exec_max))
            }
            StateTarget::Policy { id } => {
                let policies: Vec<PolicyTable> = Self::parse_bundle_json(
                    "condition.execNumCompare",
                    "policies_json",
                    &snapshot.policies_json,
                )?;
                let policy = policies.into_iter().find(|policy| policy.id == *id).ok_or_else(|| {
                    Self::execute_error(
                        "condition.execNumCompare",
                        format!("目标策略[{}]不存在", id),
                    )
                })?;
                Ok((policy.data.0.exec_max > 0).then_some(u32::from(policy.data.0.exec_max)))
            }
        }
    }

    fn compare_exec_count(exec_cur: u32, op: &CompareOp, exec_max: Option<u32>) -> bool {
        match exec_max {
            Some(exec_max) => match op {
                CompareOp::Eq => exec_cur == exec_max,
                CompareOp::Ne => exec_cur != exec_max,
                CompareOp::Lt => exec_cur < exec_max,
                CompareOp::Le => exec_cur <= exec_max,
                CompareOp::Gt => exec_cur > exec_max,
                CompareOp::Ge => exec_cur >= exec_max,
                CompareOp::Contains | CompareOp::NotContains => false,
            },
            None => match op {
                CompareOp::Eq => false,
                CompareOp::Ne => true,
                CompareOp::Lt => true,
                CompareOp::Le => true,
                CompareOp::Gt => false,
                CompareOp::Ge => false,
                CompareOp::Contains | CompareOp::NotContains => false,
            },
        }
    }
}
