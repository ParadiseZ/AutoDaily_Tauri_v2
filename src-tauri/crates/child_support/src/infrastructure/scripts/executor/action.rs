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
        let result = self.dispatch_action(action).await;
        self.after_action(action).await?;
        result
    }

    async fn before_action(&mut self, _action: &Action) -> ExecuteResult<()> {
        Ok(())
    }

    async fn after_action(&mut self, action: &Action) -> ExecuteResult<()> {
        if !Self::action_requires_wait(action) {
            return Ok(());
        }

        let Some(runtime_policy) = get_runtime_execution_policy().await else {
            return Ok(());
        };

        if runtime_policy.action_wait_ms > 0 {
            tokio::time::sleep(Duration::from_millis(runtime_policy.action_wait_ms)).await;
        }

        Ok(())
    }

    async fn dispatch_action(&mut self, action: &Action) -> ExecuteResult<ControlFlow> {
        match action {
            Action::Capture { output_var } => {
                let image = Arc::new(get_device_ctx().get_screenshot().await.ok_or_else(|| {
                    Self::execute_error("action.capture", "获取设备截图失败".to_string())
                })?);
                self.activate_image_context("action.capture", image, Some(output_var))
                    .await?;
                Ok(ControlFlow::Next)
            }
            Action::Click { mode } => self.execute_click(mode).await,
            Action::Swipe { duration, mode } => self.execute_swipe(mode, *duration).await,
            Action::Reboot => {
                get_device_ctx()
                    .reboot()
                    .await
                    .map_err(|e| Self::execute_error("action.reboot", e))?;
                Ok(ControlFlow::Next)
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
                Ok(ControlFlow::Next)
            }
            Action::StopApp { pkg_name } => {
                get_device_ctx()
                    .stop_app(pkg_name)
                    .await
                    .map_err(|e| Self::execute_error("action.stopApp", e))?;
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_click(&mut self, mode: &ClickMode) -> ExecuteResult<ControlFlow> {
        let point = match mode {
            ClickMode::Point { p } => Self::point_to_absolute(p),
            ClickMode::Percent { p } => {
                let screen_size = self.ensure_screen_size().await?;
                Self::percent_point_to_absolute(p, screen_size)?
            }
            ClickMode::Txt { input_var, txt } => {
                self.resolve_ocr_target_point("action.click", input_var, txt.as_deref(), "点击目标")
                    .await?
            }
            ClickMode::LabelIdx { input_var, idx } => {
                self.resolve_det_target_point("action.click", input_var, *idx, "点击目标")
                    .await?
            }
        };
        get_device_ctx()
            .click(point)
            .await
            .map_err(|e| Self::execute_error("action.click", e))?;
        Ok(ControlFlow::Next)
    }

    async fn execute_swipe(
        &mut self,
        mode: &SwipeMode,
        duration: u64,
    ) -> ExecuteResult<ControlFlow> {
        let (from, to) = match mode {
            SwipeMode::Point { from, to } => {
                (Self::point_to_absolute(from), Self::point_to_absolute(to))
            }
            SwipeMode::Percent { from, to } => {
                let screen_size = self.ensure_screen_size().await?;
                (
                    Self::percent_point_to_absolute(from, screen_size)?,
                    Self::percent_point_to_absolute(to, screen_size)?,
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
                )
            }
        };
        get_device_ctx()
            .swipe(from, to, duration)
            .await
            .map_err(|e| Self::execute_error("action.swipe", e))?;
        Ok(ControlFlow::Next)
    }

    async fn resolve_ocr_target_point(
        &self,
        step_type: &str,
        input_var: &str,
        target_text: Option<&str>,
        target_label: &str,
    ) -> ExecuteResult<Point<u16>> {
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
        Self::bounding_box_center_to_point(step_type, target_label, &item.bounding_box)
    }

    async fn resolve_det_target_point(
        &self,
        step_type: &str,
        input_var: &str,
        target_idx: Option<u32>,
        target_label: &str,
    ) -> ExecuteResult<Point<u16>> {
        let items = self
            .read_runtime_result_vec::<DetResult>(input_var, step_type, "检测")
            .await?;
        let item = Self::select_det_result(&items, target_idx).ok_or_else(|| {
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
        Self::bounding_box_center_to_point(step_type, target_label, &item.bounding_box)
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
