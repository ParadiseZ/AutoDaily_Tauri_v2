impl ScriptExecutor {
    async fn execute_action_step(
        &mut self,
        cur_exec_num: u32,
        max_exec_num: u32,
        action: &Action,
    ) -> ExecuteResult<ControlFlow> {
        if cur_exec_num > max_exec_num {
            return Ok(ControlFlow::Next);
        }

        self.before_action(action).await?;
        let (flow, action_trace) = self.dispatch_action(action).await?;
        if let Some(action_trace) = action_trace.as_ref() {
            self.record_action_trace(action_trace.clone());
        }
        let post_action_flow = self.after_action(action, action_trace.as_ref()).await?;
        Ok(post_action_flow.unwrap_or(flow))
    }

    async fn before_action(&mut self, _action: &Action) -> ExecuteResult<()> {
        Ok(())
    }

    async fn after_action(
        &mut self,
        action: &Action,
        action_trace: Option<&PolicyActionTrace>,
    ) -> ExecuteResult<Option<ControlFlow>> {
        if !Self::action_requires_wait(action) {
            return Ok(None);
        }

        let Some(runtime_policy) = get_runtime_execution_policy().await else {
            return Ok(None);
        };

        if runtime_policy.action_wait_ms > 0 {
            tokio::time::sleep(Duration::from_millis(runtime_policy.action_wait_ms)).await;
        }

        self.observe_refresh_hook(action, action_trace).await;
        self.evaluate_progress_timeout(&runtime_policy, action_trace)
            .await
    }

    async fn dispatch_action(
        &mut self,
        action: &Action,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        match action {
            Action::Capture { output_var } => {
                let image = Arc::new(get_device_ctx().get_screenshot().await.ok_or_else(|| {
                    Self::execute_error("action.capture", "获取设备截图失败".to_string())
                })?);
                self.activate_image_context("action.capture", image, Some(output_var))
                    .await?;
                Ok((ControlFlow::Next, None))
            }
            Action::Click { mode } => self.execute_click(mode).await,
            Action::Swipe { duration, mode } => self.execute_swipe(mode, *duration).await,
            Action::Reboot => {
                get_device_ctx()
                    .reboot()
                    .await
                    .map_err(|e| Self::execute_error("action.reboot", e))?;
                Ok((
                    ControlFlow::Next,
                    Some(Self::build_simple_action_trace(PolicyActionKind::Reboot)),
                ))
            }
            Action::LaunchApp {
                pkg_name,
                activity_name,
            } => {
                if pkg_name.trim().is_empty() || activity_name.trim().is_empty() {
                    return Err(Self::execute_error(
                        "action.launchApp",
                        "LaunchApp 需要同时提供 pkg_name 和 activity_name".to_string(),
                    ));
                }
                get_device_ctx()
                    .launch_app(pkg_name, activity_name)
                    .await
                    .map_err(|e| Self::execute_error("action.launchApp", e))?;
                Ok((
                    ControlFlow::Next,
                    Some(Self::build_simple_action_trace(PolicyActionKind::StartApp)),
                ))
            }
            Action::StopApp { pkg_name } => {
                get_device_ctx()
                    .stop_app(pkg_name)
                    .await
                    .map_err(|e| Self::execute_error("action.stopApp", e))?;
                Ok((
                    ControlFlow::Next,
                    Some(Self::build_simple_action_trace(PolicyActionKind::StopApp)),
                ))
            }
        }
    }

    async fn execute_click(
        &mut self,
        mode: &ClickMode,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        let (point, trace) = match mode {
            ClickMode::Point { p } => {
                let point = Self::point_to_absolute(p);
                let target = Self::build_point_target(PolicyActionTargetRole::Primary, point);
                (
                    point,
                    Self::build_action_trace(
                        PolicyActionKind::Click,
                        PolicyActionSource::Fixed,
                        vec![target],
                    ),
                )
            }
            ClickMode::Percent { p } => {
                let screen_size = self.ensure_screen_size().await?;
                let point = Self::percent_point_to_absolute(p, screen_size)?;
                let target = Self::build_point_target(PolicyActionTargetRole::Primary, point);
                (
                    point,
                    Self::build_action_trace(
                        PolicyActionKind::Click,
                        PolicyActionSource::Fixed,
                        vec![target],
                    ),
                )
            }
            ClickMode::Txt { input_var, txt } => {
                let (point, item) = self
                    .resolve_ocr_target_point("action.click", input_var, txt.as_deref(), "点击目标")
                    .await?;
                let target = Self::build_ocr_target(
                    PolicyActionTargetRole::Primary,
                    point,
                    &item,
                );
                (
                    point,
                    Self::build_action_trace(
                        PolicyActionKind::Click,
                        PolicyActionSource::Ocr,
                        vec![target],
                    ),
                )
            }
            ClickMode::LabelIdx { input_var, idx } => {
                let (point, item) = self
                    .resolve_det_target_point("action.click", input_var, *idx, "点击目标")
                    .await?;
                let target = Self::build_det_target(
                    PolicyActionTargetRole::Primary,
                    point,
                    &item,
                );
                (
                    point,
                    Self::build_action_trace(
                        PolicyActionKind::Click,
                        PolicyActionSource::Det,
                        vec![target],
                    ),
                )
            }
        };
        get_device_ctx()
            .click(point)
            .await
            .map_err(|e| Self::execute_error("action.click", e))?;
        Ok((ControlFlow::Next, Some(trace)))
    }

    async fn execute_swipe(
        &mut self,
        mode: &SwipeMode,
        duration: u64,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
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
            } => {
                let items = self
                    .read_runtime_result_vec::<OcrResult>(input_var, "action.swipe", "OCR")
                    .await?;
                let from_item =
                    Self::select_ocr_result(&items, from.as_deref()).ok_or_else(|| {
                        Self::execute_error(
                            "action.swipe",
                            format!(
                                "输入变量[{}]里未找到文字滑动起点: {}",
                                input_var,
                                from.clone().unwrap_or_default()
                            ),
                        )
                    })?;
                let to_item = Self::select_ocr_result(&items, to.as_deref()).ok_or_else(|| {
                    Self::execute_error(
                        "action.swipe",
                        format!(
                            "输入变量[{}]里未找到文字滑动终点: {}",
                            input_var,
                            to.clone().unwrap_or_default()
                        ),
                    )
                })?;
                (
                    Self::bounding_box_center_to_point(
                        "action.swipe",
                        "文字滑动起点",
                        &from_item.bounding_box,
                    )?,
                    Self::bounding_box_center_to_point(
                        "action.swipe",
                        "文字滑动终点",
                        &to_item.bounding_box,
                    )?,
                    Self::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Ocr,
                        vec![
                            Self::build_ocr_target(
                                PolicyActionTargetRole::Start,
                                Self::bounding_box_center_to_point(
                                    "action.swipe",
                                    "文字滑动起点",
                                    &from_item.bounding_box,
                                )?,
                                from_item,
                            ),
                            Self::build_ocr_target(
                                PolicyActionTargetRole::End,
                                Self::bounding_box_center_to_point(
                                    "action.swipe",
                                    "文字滑动终点",
                                    &to_item.bounding_box,
                                )?,
                                to_item,
                            ),
                        ],
                    ),
                )
            }
            SwipeMode::LabelIdx {
                input_var,
                from,
                to,
            } => {
                let items = self
                    .read_runtime_result_vec::<DetResult>(input_var, "action.swipe", "检测")
                    .await?;
                let from_item = Self::select_det_result(&items, Some(u32::from(*from)))
                    .ok_or_else(|| {
                        Self::execute_error(
                            "action.swipe",
                            format!("输入变量[{}]里未找到标签滑动起点: {}", input_var, from),
                        )
                    })?;
                let to_item =
                    Self::select_det_result(&items, Some(u32::from(*to))).ok_or_else(|| {
                        Self::execute_error(
                            "action.swipe",
                            format!("输入变量[{}]里未找到标签滑动终点: {}", input_var, to),
                        )
                    })?;
                (
                    Self::bounding_box_center_to_point(
                        "action.swipe",
                        "标签滑动起点",
                        &from_item.bounding_box,
                    )?,
                    Self::bounding_box_center_to_point(
                        "action.swipe",
                        "标签滑动终点",
                        &to_item.bounding_box,
                    )?,
                    Self::build_action_trace(
                        PolicyActionKind::Swipe,
                        PolicyActionSource::Det,
                        vec![
                            Self::build_det_target(
                                PolicyActionTargetRole::Start,
                                Self::bounding_box_center_to_point(
                                    "action.swipe",
                                    "标签滑动起点",
                                    &from_item.bounding_box,
                                )?,
                                from_item,
                            ),
                            Self::build_det_target(
                                PolicyActionTargetRole::End,
                                Self::bounding_box_center_to_point(
                                    "action.swipe",
                                    "标签滑动终点",
                                    &to_item.bounding_box,
                                )?,
                                to_item,
                            ),
                        ],
                    ),
                )
            }
        };
        get_device_ctx()
            .swipe(from, to, duration)
            .await
            .map_err(|e| Self::execute_error("action.swipe", e))?;
        Ok((ControlFlow::Next, Some(trace)))
    }

    async fn resolve_ocr_target_point(
        &self,
        step_type: &str,
        input_var: &str,
        target_text: Option<&str>,
        target_label: &str,
    ) -> ExecuteResult<(Point<u16>, OcrResult)> {
        let items = self
            .read_runtime_result_vec::<OcrResult>(input_var, step_type, "OCR")
            .await?;
        let item = Self::select_ocr_result(&items, target_text).cloned().ok_or_else(|| {
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
        Ok((point, item))
    }

    async fn resolve_det_target_point(
        &self,
        step_type: &str,
        input_var: &str,
        target_idx: Option<u32>,
        target_label: &str,
    ) -> ExecuteResult<(Point<u16>, DetResult)> {
        let items = self
            .read_runtime_result_vec::<DetResult>(input_var, step_type, "检测")
            .await?;
        let item = Self::select_det_result(&items, target_idx).cloned().ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!(
                    "输入变量[{}]里未找到{}标签: {}",
                    input_var,
                    target_label,
                    target_idx
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "<empty>".to_string())
                ),
            )
        })?;
        let point = Self::bounding_box_center_to_point(step_type, target_label, &item.bounding_box)?;
        Ok((point, item))
    }

    async fn build_capture_observation(
        &self,
        image: &RgbaImage,
    ) -> ExecuteResult<(Vec<DetResult>, Vec<OcrResult>, VisionSnapshot)> {
        let (
            grid_size,
            img_det_service,
            ocr_service,
            img_det_model_json,
            txt_det_model_json,
            txt_rec_model_json,
            has_img_det_model,
            has_txt_det_model,
            has_txt_rec_model,
        ) = {
            let ctx = self.runtime_ctx.read().await;
            let Some(script_info) = ctx.execution.script_info.as_ref() else {
                return Err(Self::execute_error(
                    "action.capture",
                    "当前运行时缺少 script_info，无法生成视觉快照".to_string(),
                ));
            };

            (
                ctx.observation.vision_signature_grid_size,
                ctx.img_det_service.clone(),
                ctx.ocr_service.clone(),
                Self::serialize_json("action.capture", "img_det_model", &script_info.img_det_model)?,
                Self::serialize_json("action.capture", "txt_det_model", &script_info.txt_det_model)?,
                Self::serialize_json("action.capture", "txt_rec_model", &script_info.txt_rec_model)?,
                script_info.img_det_model.is_some(),
                script_info.txt_det_model.is_some(),
                script_info.txt_rec_model.is_some(),
            )
        };

        if has_txt_det_model != has_txt_rec_model {
            return Err(Self::execute_error(
                "action.capture",
                "OCR 链路需要同时配置文字检测模型和文字识别模型".to_string(),
            ));
        }

        let cache_key = Self::build_capture_cache_key(
            image,
            grid_size,
            &img_det_model_json,
            &txt_det_model_json,
            &txt_rec_model_json,
        );

        if let Some(entry) = {
            let mut ctx = self.runtime_ctx.write().await;
            ctx.observation.vision_text_cache.find_entry(&cache_key)
        } {
            match VisionSnapshot::new(
                entry.ocr_results.clone(),
                entry.det_results.clone(),
                None,
                grid_size,
            ) {
                Ok(snapshot) => {
                    return Ok((entry.det_results, entry.ocr_results, snapshot));
                }
                Err(error) => {
                    Log::warn(&format!(
                        "[ executor ] OCR 缓存命中但重建视觉快照失败，已回退实时推理: {}",
                        error
                    ));
                }
            }
        }

        let capture_image = DynamicImage::ImageRgba8(image.clone());
        let det_results = if has_img_det_model {
            let mut service = img_det_service.lock().await;
            service.detect(&capture_image).map_err(|error| {
                Self::execute_error("action.capture", format!("目标检测执行失败: {}", error))
            })?
        } else {
            Vec::new()
        };
        let ocr_results = if has_txt_det_model {
            let mut service = ocr_service.lock().await;
            service.ocr_batch(&capture_image).map_err(|error| {
                Self::execute_error("action.capture", format!("OCR 执行失败: {}", error))
            })?
        } else {
            Vec::new()
        };

        let snapshot =
            VisionSnapshot::new(ocr_results.clone(), det_results.clone(), None, grid_size)
                .map_err(|error| {
                    Self::execute_error("action.capture", format!("构建视觉快照失败: {}", error))
                })?;

        {
            let mut ctx = self.runtime_ctx.write().await;
            if let Err(error) = ctx.observation.vision_text_cache.record_entry(
                cache_key,
                det_results.clone(),
                ocr_results.clone(),
            ) {
                Log::warn(&format!(
                    "[ executor ] 写入 OCR 文字缓存失败，已忽略本次缓存: {}",
                    error
                ));
            }
        }

        Ok((det_results, ocr_results, snapshot))
    }

    async fn activate_image_context(
        &mut self,
        step_type: &str,
        image: Arc<RgbaImage>,
        output_var: Option<&str>,
    ) -> ExecuteResult<()> {
        let (det_results, ocr_results, snapshot) =
            self.build_capture_observation(image.as_ref()).await?;
        let fingerprint = Self::build_page_fingerprint(&snapshot);
        let screen_size = (image.width(), image.height());

        if let Some(output_var) = output_var {
            self.set_runtime_var(output_var, Dynamic::from(image.clone()))
                .await?;
        }
        self.set_runtime_var("runtime.latestCapture", Dynamic::from(image.clone()))
            .await?;
        self.set_runtime_var(
            "runtime.detResults",
            Self::results_to_dynamic(step_type, "检测", &det_results)?,
        )
        .await?;
        self.set_runtime_var(
            "runtime.ocrResults",
            Self::results_to_dynamic(step_type, "OCR", &ocr_results)?,
        )
        .await?;

        let mut ctx = self.runtime_ctx.write().await;
        ctx.observation.last_capture_image = Some(image);
        ctx.observation.screen_size = screen_size;
        ctx.observation.last_snapshot = Some(snapshot);
        ctx.observation.last_hits.clear();
        drop(ctx);

        self.push_active_policy_page_fingerprint(fingerprint);
        Ok(())
    }

    async fn activate_image_var(&mut self, step_type: &str, input_var: &str) -> ExecuteResult<()> {
        let image = self.read_runtime_image_var(input_var, step_type).await?;
        self.activate_image_context(step_type, image, None).await
    }

    async fn read_runtime_image_var(
        &self,
        input_var: &str,
        step_type: &str,
    ) -> ExecuteResult<Arc<RgbaImage>> {
        let value = self.read_runtime_var(input_var).await.ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!("输入变量[{}]不存在，无法读取图像", input_var),
            )
        })?;

        value.try_cast::<Arc<RgbaImage>>().ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!("输入变量[{}]不是图像变量，无法构建视觉快照", input_var),
            )
        })
    }

    async fn ensure_screen_size(&self) -> ExecuteResult<(u32, u32)> {
        let cached = {
            let ctx = self.runtime_ctx.read().await;
            if ctx.observation.screen_size.0 > 0 && ctx.observation.screen_size.1 > 0 {
                return Ok(ctx.observation.screen_size);
            }
            ctx.observation
                .last_capture_image
                .as_ref()
                .map(|image| (image.width(), image.height()))
        };
        if let Some(screen_size) = cached {
            let mut ctx = self.runtime_ctx.write().await;
            ctx.observation.screen_size = screen_size;
            return Ok(screen_size);
        }
        let image = get_device_ctx().get_screenshot().await.ok_or_else(|| {
            Self::execute_error("action.screenSize", "获取屏幕尺寸失败".to_string())
        })?;
        let screen_size = (image.width(), image.height());
        let mut ctx = self.runtime_ctx.write().await;
        ctx.observation.last_capture_image = Some(Arc::new(image));
        ctx.observation.screen_size = screen_size;
        Ok(screen_size)
    }

    fn point_to_absolute(point: &PointU16) -> Point<u16> {
        Point::new(point.x, point.y)
    }

    fn bounding_box_center_to_point(
        step_type: &str,
        target_label: &str,
        bounding_box: &BoundingBox,
    ) -> ExecuteResult<Point<u16>> {
        let center = bounding_box.center();
        let x = u16::try_from(center.x).map_err(|_| {
            Self::execute_error(
                step_type,
                format!("{}中心点 x 坐标越界: {}", target_label, center.x),
            )
        })?;
        let y = u16::try_from(center.y).map_err(|_| {
            Self::execute_error(
                step_type,
                format!("{}中心点 y 坐标越界: {}", target_label, center.y),
            )
        })?;
        Ok(Point::new(x, y))
    }

    fn select_ocr_result<'a>(
        items: &'a [OcrResult],
        target_text: Option<&str>,
    ) -> Option<&'a OcrResult> {
        let target_text = target_text.map(str::trim).filter(|value| !value.is_empty());
        match target_text {
            Some(target) => items
                .iter()
                .find(|item| item.txt.trim() == target)
                .or_else(|| items.iter().find(|item| item.txt.contains(target))),
            None => items.first(),
        }
    }

    fn select_det_result(items: &[DetResult], target_idx: Option<u32>) -> Option<&DetResult> {
        match target_idx {
            Some(target) => items.iter().find(|item| item.index == target as i32),
            None => items.first(),
        }
    }

    fn serialize_json<T>(
        step_type: &str,
        field: &str,
        value: &T,
    ) -> ExecuteResult<String>
    where
        T: Serialize,
    {
        serde_json::to_string(value).map_err(|error| {
            Self::execute_error(
                step_type,
                format!("序列化 {} 失败，无法构建视觉缓存键: {}", field, error),
            )
        })
    }

    fn results_to_dynamic<T>(
        step_type: &str,
        result_label: &str,
        value: &T,
    ) -> ExecuteResult<Dynamic>
    where
        T: Serialize,
    {
        to_dynamic(value).map_err(|error| {
            Self::execute_error(
                step_type,
                format!("序列化{}结果集到运行时变量失败: {}", result_label, error),
            )
        })
    }

    fn build_capture_cache_key(
        image: &RgbaImage,
        signature_grid_size: u16,
        img_det_model_json: &str,
        txt_det_model_json: &str,
        txt_rec_model_json: &str,
    ) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"capture:v1");
        hasher.write(&image.width().to_le_bytes());
        hasher.write(&image.height().to_le_bytes());
        hasher.write(&signature_grid_size.max(1).to_le_bytes());
        Self::write_hash_segment(&mut hasher, img_det_model_json.as_bytes());
        Self::write_hash_segment(&mut hasher, txt_det_model_json.as_bytes());
        Self::write_hash_segment(&mut hasher, txt_rec_model_json.as_bytes());
        Self::write_hash_segment(&mut hasher, image.as_raw());
        format!("capture:v1:{:016x}", hasher.finish())
    }

    fn write_hash_segment(hasher: &mut XxHash3_64, bytes: &[u8]) {
        hasher.write(&(bytes.len() as u64).to_le_bytes());
        hasher.write(bytes);
    }

    async fn read_runtime_result_vec<T>(
        &self,
        input_var: &str,
        step_type: &str,
        result_label: &str,
    ) -> ExecuteResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let value = self.read_runtime_var(input_var).await.ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!(
                    "输入变量[{}]不存在，无法读取{}结果集",
                    input_var, result_label
                ),
            )
        })?;

        Self::deserialize_dynamic_value::<Vec<T>>(&value).map_err(|error| {
            Self::execute_error(
                step_type,
                format!(
                    "输入变量[{}]不是兼容的{}结果集，无法执行动作: {}",
                    input_var, result_label, error
                ),
            )
        })
    }

    fn record_action_trace(&mut self, mut action_trace: PolicyActionTrace) {
        if let Some(round) = self.active_policy_round.as_mut() {
            action_trace.action_index = round.actions.len() as u16;
            round.action_signatures.push(action_trace.signature.clone());
            round.actions.push(action_trace);
        }
    }

    fn push_active_policy_page_fingerprint(&mut self, fingerprint: String) {
        let Some(round) = self.active_policy_round.as_mut() else {
            return;
        };

        if round.page_fingerprints.last() != Some(&fingerprint) {
            round.page_fingerprints.push(fingerprint);
        }
    }

    async fn current_page_fingerprint(&self) -> Option<String> {
        let ctx = self.runtime_ctx.read().await;
        ctx.observation
            .last_snapshot
            .as_ref()
            .map(Self::build_page_fingerprint)
    }

    async fn observe_refresh_hook(
        &self,
        action: &Action,
        action_trace: Option<&PolicyActionTrace>,
    ) {
        let Some(action_trace) = action_trace else {
            return;
        };

        let page_fingerprint = self.current_page_fingerprint().await;
        Log::debug(&format!(
            "[ executor ] 动作后 observe hook: action={:?}, signature={}, page_fingerprint={}",
            action,
            action_trace.signature,
            page_fingerprint.unwrap_or_else(|| "<none>".to_string())
        ));
    }

    async fn evaluate_progress_timeout(
        &mut self,
        runtime_policy: &crate::infrastructure::ipc::message::RuntimeExecutionPolicy,
        action_trace: Option<&PolicyActionTrace>,
    ) -> ExecuteResult<Option<ControlFlow>> {
        let Some(action_trace) = action_trace else {
            self.last_progress_probe = None;
            return Ok(None);
        };

        let Some(page_fingerprint) = self.current_page_fingerprint().await else {
            self.last_progress_probe = None;
            return Ok(None);
        };

        let probe = ActionProgressProbe {
            page_fingerprint: page_fingerprint.clone(),
            action_signature: action_trace.signature.clone(),
            recorded_at: Instant::now(),
        };

        if !runtime_policy.progress_timeout_enabled || runtime_policy.progress_timeout_ms == 0 {
            self.last_progress_probe = Some(probe);
            return Ok(None);
        }

        let timeout_elapsed = self
            .last_progress_probe
            .as_ref()
            .filter(|previous| previous.page_fingerprint == probe.page_fingerprint)
            .filter(|previous| previous.action_signature == probe.action_signature)
            .map(|previous| {
                probe.recorded_at.duration_since(previous.recorded_at)
                    >= Duration::from_millis(runtime_policy.progress_timeout_ms)
            })
            .unwrap_or(false);

        self.last_progress_probe = Some(probe);
        if !timeout_elapsed {
            return Ok(None);
        }

        let message = format!(
            "检测到动作长时间无进展: action={}, page={}, threshold={}ms",
            action_trace.signature,
            page_fingerprint,
            runtime_policy.progress_timeout_ms
        );
        self.emit_timeout_signals(
            runtime_policy.timeout_action.clone(),
            runtime_policy.timeout_notify_channels.clone(),
            Some(page_fingerprint.clone()),
            Some(action_trace.signature.clone()),
            message.clone(),
        )
        .await;

        self.handle_timeout_action(runtime_policy, message).await
    }

    async fn handle_timeout_action(
        &mut self,
        runtime_policy: &crate::infrastructure::ipc::message::RuntimeExecutionPolicy,
        message: String,
    ) -> ExecuteResult<Option<ControlFlow>> {
        match runtime_policy.timeout_action {
            TimeoutAction::NotifyOnly => Ok(None),
            TimeoutAction::SkipCurrentTask => {
                self.mark_current_task_skipped().await;
                Ok(Some(ControlFlow::Return))
            }
            TimeoutAction::RunRecoveryTask => {
                let recovery_task_id = self.recovery_task_id().await.ok_or_else(|| {
                    Self::execute_error(
                        "action.timeout",
                        "当前脚本未配置 recovery_task_id，无法执行 RunRecoveryTask".to_string(),
                    )
                })?;
                Ok(Some(ControlFlow::Link(recovery_task_id)))
            }
            TimeoutAction::StopExecution => {
                self.prepare_timeout_checkpoint_if_needed().await;
                crate::infrastructure::context::child_process_sec::set_running_status(
                    crate::infrastructure::context::child_process_sec::RunningStatus::Idle,
                );
                emit_progress_event(
                    RuntimeProgressPhase::Idle,
                    None,
                    None,
                    None,
                    None,
                    Some(message.clone()),
                );
                emit_lifecycle_event(RuntimeLifecyclePhase::Idle, Some(message.clone()));
                Err(Self::execute_error("action.timeout", message))
            }
            TimeoutAction::PauseExecution => {
                self.prepare_timeout_checkpoint_if_needed().await;
                crate::infrastructure::context::child_process_sec::set_running_status(
                    crate::infrastructure::context::child_process_sec::RunningStatus::Paused,
                );
                emit_progress_event(
                    RuntimeProgressPhase::Paused,
                    None,
                    None,
                    None,
                    None,
                    Some(message.clone()),
                );
                emit_lifecycle_event(RuntimeLifecyclePhase::Paused, Some(message.clone()));
                Err(Self::execute_error("action.timeout", message))
            }
            TimeoutAction::RestartApp => {
                self.prepare_timeout_checkpoint_if_needed().await;
                let (script_name, pkg_name, activity_name) =
                    self.current_script_launch_target().await?;
                if let Err(error) = get_device_ctx().stop_app(&pkg_name).await {
                    Log::warn(&format!(
                        "[ executor ] timeout RestartApp 停止应用失败，继续尝试拉起: script={}, pkg={}, error={}",
                        script_name, pkg_name, error
                    ));
                }
                get_device_ctx()
                    .launch_app(&pkg_name, &activity_name)
                    .await
                    .map_err(|error| {
                        Self::execute_error(
                            "action.timeout",
                            format!(
                                "{}；执行 RestartApp 失败: script={}, pkg={}, activity={}, error={}",
                                message, script_name, pkg_name, activity_name, error
                            ),
                        )
                    })?;
                Err(Self::execute_error(
                    "action.timeout",
                    format!(
                        "{}；已执行 RestartApp: script={}, pkg={}, activity={}。当前执行不会自动恢复到目标页，如需继续编排，请改用 RunRecoveryTask。",
                        message, script_name, pkg_name, activity_name
                    ),
                ))
            }
        }
    }

    async fn emit_timeout_signals(
        &self,
        timeout_action: TimeoutAction,
        notify_channels: Vec<crate::infrastructure::ipc::message::TimeoutNotifyChannel>,
        page_fingerprint: Option<String>,
        action_signature: Option<String>,
        message: String,
    ) {
        let (_execution_id, assignment_id, script_id, task_id, step_id) =
            self.current_execution_locator().await;
        let timeout_message = format!(
            "[timeout] action={:?}; page={}; signature={}; {}",
            timeout_action,
            page_fingerprint.clone().unwrap_or_else(|| "<none>".to_string()),
            action_signature.clone().unwrap_or_else(|| "<none>".to_string()),
            message
        );
        emit_progress_event(
            RuntimeProgressPhase::Executing,
            assignment_id,
            script_id,
            task_id,
            step_id,
            Some(timeout_message),
        );

        if notify_channels.iter().any(|channel| {
            matches!(
                channel,
                crate::infrastructure::ipc::message::TimeoutNotifyChannel::SystemNotification
            )
        }) {
            emit_progress_event(
                RuntimeProgressPhase::Executing,
                assignment_id,
                script_id,
                task_id,
                step_id,
                Some(format!("[timeout_notify] {}", message)),
            );
        }
    }

    async fn current_execution_locator(
        &self,
    ) -> (
        Option<ExecutionId>,
        Option<ScheduleId>,
        Option<crate::infrastructure::core::ScriptId>,
        Option<TaskId>,
        Option<StepId>,
    ) {
        let ctx = self.runtime_ctx.read().await;
        (
            ctx.execution.current_execution_id,
            ctx.execution.current_assignment_id,
            Some(ctx.execution.script_id),
            ctx.execution.current_task.as_ref().map(|task| task.id),
            ctx.execution.current_step_id,
        )
    }

    async fn mark_current_task_skipped(&self) {
        let Some(task_id) = ({
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.current_task.as_ref().map(|task| task.id)
        }) else {
            return;
        };

        let mut ctx = self.runtime_ctx.write().await;
        let state = ctx.execution.task_states.entry(task_id).or_default();
        state.skip_flag = true;
    }

    async fn recovery_task_id(&self) -> Option<TaskId> {
        let ctx = self.runtime_ctx.read().await;
        ctx.execution
            .script_info
            .as_ref()
            .and_then(|info| info.runtime_settings.recovery_task_id)
    }

    async fn current_script_launch_target(&self) -> ExecuteResult<(String, String, String)> {
        let ctx = self.runtime_ctx.read().await;
        let script_info = ctx.execution.script_info.as_ref().ok_or_else(|| {
            Self::execute_error(
                "action.timeout",
                "当前运行时缺少 script_info，无法执行 RestartApp".to_string(),
            )
        })?;

        let pkg_name = script_info
            .pkg_name
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                Self::execute_error(
                    "action.timeout",
                    format!(
                        "脚本[{}]未配置全局 pkg_name，无法执行 RestartApp",
                        script_info.name
                    ),
                )
            })?;
        let activity_name = script_info
            .activity_name
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                Self::execute_error(
                    "action.timeout",
                    format!(
                        "脚本[{}]未配置全局 activity_name，无法执行 RestartApp",
                        script_info.name
                    ),
                )
            })?;

        Ok((script_info.name.clone(), pkg_name, activity_name))
    }

    async fn prepare_timeout_checkpoint_if_needed(&self) {
        if let Err(error) = prepare_and_persist_checkpoint(SessionCheckpointReason::Manual).await {
            Log::warn(&format!(
                "[ executor ] timeout 后写入 checkpoint 失败，已忽略: {}",
                error
            ));
        }
    }

    fn build_page_fingerprint(snapshot: &VisionSnapshot) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"page:v1");
        hasher.write(&snapshot.signature_grid_size.to_le_bytes());
        for item in &snapshot.layout_items {
            hasher.write(&[match item.source {
                crate::domain::vision::ocr_search::VisionLayoutSource::Ocr => 1,
                crate::domain::vision::ocr_search::VisionLayoutSource::Det => 2,
            }]);
            hasher.write(&(item.item_index as u64).to_le_bytes());
            hasher.write(&item.stable_box.x1.to_le_bytes());
            hasher.write(&item.stable_box.y1.to_le_bytes());
            hasher.write(&item.stable_box.x2.to_le_bytes());
            hasher.write(&item.stable_box.y2.to_le_bytes());
            hasher.write(&item.stable_center.x.to_le_bytes());
            hasher.write(&item.stable_center.y.to_le_bytes());
            Self::write_hash_segment(&mut hasher, item.text.as_deref().unwrap_or("").as_bytes());
            Self::write_hash_segment(&mut hasher, item.label.as_deref().unwrap_or("").as_bytes());
            hasher.write(&item.label_index.unwrap_or_default().to_le_bytes());
        }
        format!("page:v1:{:016x}", hasher.finish())
    }

    fn build_simple_action_trace(kind: PolicyActionKind) -> PolicyActionTrace {
        Self::build_action_trace(kind, PolicyActionSource::Custom, Vec::new())
    }

    fn build_action_trace(
        kind: PolicyActionKind,
        source: PolicyActionSource,
        targets: Vec<PolicyActionTarget>,
    ) -> PolicyActionTrace {
        let signature = Self::build_action_signature(&kind, &source, &targets);
        PolicyActionTrace {
            action_index: 0,
            kind,
            source,
            signature,
            targets,
        }
    }

    fn build_action_signature(
        kind: &PolicyActionKind,
        source: &PolicyActionSource,
        targets: &[PolicyActionTarget],
    ) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"policy-action:v1");
        hasher.write(&[Self::action_kind_code(kind)]);
        hasher.write(&[Self::action_source_code(source)]);
        for target in targets {
            hasher.write(&[Self::target_role_code(&target.role)]);
            let (x, y) = target
                .point
                .as_ref()
                .map(|point| (point.x, point.y))
                .unwrap_or_default();
            hasher.write(&x.to_le_bytes());
            hasher.write(&y.to_le_bytes());
            if let Some(box_area) = target.box_area.as_ref() {
                hasher.write(&box_area.x1.to_le_bytes());
                hasher.write(&box_area.y1.to_le_bytes());
                hasher.write(&box_area.x2.to_le_bytes());
                hasher.write(&box_area.y2.to_le_bytes());
            } else {
                hasher.write(&0i32.to_le_bytes());
                hasher.write(&0i32.to_le_bytes());
                hasher.write(&0i32.to_le_bytes());
                hasher.write(&0i32.to_le_bytes());
            }
            Self::write_hash_segment(&mut hasher, target.text.as_deref().unwrap_or("").as_bytes());
            hasher.write(&target.label_id.unwrap_or_default().to_le_bytes());
        }
        format!("action:v1:{:016x}", hasher.finish())
    }

    fn build_point_target(role: PolicyActionTargetRole, point: Point<u16>) -> PolicyActionTarget {
        PolicyActionTarget {
            role,
            point: Some(PointU16 { x: point.x, y: point.y }),
            box_area: None,
            text: None,
            label_id: None,
        }
    }

    fn build_ocr_target(
        role: PolicyActionTargetRole,
        point: Point<u16>,
        item: &OcrResult,
    ) -> PolicyActionTarget {
        PolicyActionTarget {
            role,
            point: Some(PointU16 { x: point.x, y: point.y }),
            box_area: Some(item.bounding_box.clone()),
            text: Some(item.txt.clone()),
            label_id: None,
        }
    }

    fn build_det_target(
        role: PolicyActionTargetRole,
        point: Point<u16>,
        item: &DetResult,
    ) -> PolicyActionTarget {
        PolicyActionTarget {
            role,
            point: Some(PointU16 { x: point.x, y: point.y }),
            box_area: Some(item.bounding_box.clone()),
            text: Some(item.label.clone()),
            label_id: Some(item.index),
        }
    }

    fn action_kind_code(kind: &PolicyActionKind) -> u8 {
        match kind {
            PolicyActionKind::Unknown => 0,
            PolicyActionKind::Click => 1,
            PolicyActionKind::Swipe => 2,
            PolicyActionKind::Input => 3,
            PolicyActionKind::Press => 4,
            PolicyActionKind::Reboot => 5,
            PolicyActionKind::StartApp => 6,
            PolicyActionKind::StopApp => 7,
            PolicyActionKind::Back => 8,
            PolicyActionKind::Home => 9,
            PolicyActionKind::Menu => 10,
            PolicyActionKind::None => 11,
        }
    }

    fn action_source_code(source: &PolicyActionSource) -> u8 {
        match source {
            PolicyActionSource::Ocr => 1,
            PolicyActionSource::Det => 2,
            PolicyActionSource::Label => 3,
            PolicyActionSource::Fixed => 4,
            PolicyActionSource::Text => 5,
            PolicyActionSource::Custom => 6,
        }
    }

    fn target_role_code(role: &PolicyActionTargetRole) -> u8 {
        match role {
            PolicyActionTargetRole::Primary => 1,
            PolicyActionTargetRole::Secondary => 2,
            PolicyActionTargetRole::Start => 3,
            PolicyActionTargetRole::End => 4,
            PolicyActionTargetRole::Path => 5,
        }
    }

    fn action_requires_wait(action: &Action) -> bool {
        matches!(
            action,
            Action::Click { .. }
                | Action::Swipe { .. }
                | Action::Reboot
                | Action::LaunchApp { .. }
                | Action::StopApp { .. }
        )
    }

    fn percent_point_to_absolute(
        point: &PointF32,
        screen_size: (u32, u32),
    ) -> ExecuteResult<Point<u16>> {
        let (width, height) = screen_size;
        if width == 0 || height == 0 {
            return Err(Self::execute_error(
                "action.percentPoint",
                "屏幕尺寸无效，无法换算百分比坐标".to_string(),
            ));
        }
        let max_x = width.saturating_sub(1) as f32;
        let max_y = height.saturating_sub(1) as f32;
        let x = (point.x.clamp(0.0, 1.0) * max_x).round() as u16;
        let y = (point.y.clamp(0.0, 1.0) * max_y).round() as u16;
        Ok(Point::new(x, y))
    }
}
