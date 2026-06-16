impl ScriptExecutor {
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
        let det_capture_image = capture_image.clone();
        let ocr_capture_image = capture_image.clone();
        let det_results = if has_img_det_model {
            Self::run_ocr_service_with_timeout(
                "action.capture",
                "目标检测",
                VISION_INFERENCE_TIMEOUT_MS,
                img_det_service,
                move |service| {
                    service
                        .detect(&det_capture_image)
                        .map_err(|error| format!("目标检测执行失败: {}", error))
                },
            )
            .await?
        } else {
            Vec::new()
        };
        let ocr_results = if has_txt_det_model {
            Self::run_ocr_service_with_timeout(
                "action.capture",
                "OCR",
                VISION_INFERENCE_TIMEOUT_MS,
                ocr_service,
                move |service| {
                    service
                        .ocr_batch(&ocr_capture_image)
                        .map_err(|error| format!("OCR 执行失败: {}", error))
                },
            )
            .await?
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
        let image = self.capture_device_screenshot("action.screenSize").await?;
        let screen_size = (image.width(), image.height());
        let mut ctx = self.runtime_ctx.write().await;
        ctx.observation.last_capture_image = Some(Arc::new(image));
        ctx.observation.screen_size = screen_size;
        Ok(screen_size)
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

    async fn capture_device_screenshot(&self, step_type: &str) -> ExecuteResult<RgbaImage> {
        Self::await_device_option_with_timeout(
            step_type,
            "设备截图",
            DEVICE_EXTERNAL_TIMEOUT_MS,
            get_device_ctx().get_screenshot(),
        )
        .await
    }

    async fn await_device_result_with_timeout<T, F>(
        step_type: &str,
        label: &str,
        timeout_ms: u64,
        future: F,
    ) -> ExecuteResult<T>
    where
        F: Future<Output = Result<T, String>>,
    {
        match tokio::time::timeout(Duration::from_millis(timeout_ms), future).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(Self::execute_error(step_type, error)),
            Err(_) => Err(Self::execute_error(
                step_type,
                format!("{}超时，超过{}ms", label, timeout_ms),
            )),
        }
    }

    async fn await_device_option_with_timeout<T, F>(
        step_type: &str,
        label: &str,
        timeout_ms: u64,
        future: F,
    ) -> ExecuteResult<T>
    where
        F: Future<Output = Option<T>>,
    {
        match tokio::time::timeout(Duration::from_millis(timeout_ms), future).await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(Self::execute_error(step_type, format!("{}失败", label))),
            Err(_) => Err(Self::execute_error(
                step_type,
                format!("{}超时，超过{}ms", label, timeout_ms),
            )),
        }
    }

    async fn run_ocr_service_with_timeout<T, F>(
        step_type: &str,
        label: &str,
        timeout_ms: u64,
        service: Arc<Mutex<OcrService>>,
        operation: F,
    ) -> ExecuteResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut OcrService) -> Result<T, String> + Send + 'static,
    {
        let task = tokio::task::spawn_blocking(move || {
            let mut service = service.blocking_lock();
            operation(&mut service)
        });

        match tokio::time::timeout(Duration::from_millis(timeout_ms), task).await {
            Ok(Ok(Ok(value))) => Ok(value),
            Ok(Ok(Err(error))) => Err(Self::execute_error(step_type, error)),
            Ok(Err(error)) => Err(Self::execute_error(
                step_type,
                format!("{}任务执行失败: {}", label, error),
            )),
            Err(_) => Err(Self::execute_error(
                step_type,
                format!("{}超时，超过{}ms", label, timeout_ms),
            )),
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
}
