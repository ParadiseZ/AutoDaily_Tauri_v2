impl ScriptExecutor {
    async fn build_capture_observation(
        &self,
        image: Arc<RgbaImage>,
    ) -> ExecuteResult<(Vec<DetResult>, Vec<OcrResult>, VisionSnapshot)> {
        let (grid_size, has_img_det_model, has_txt_det_model, has_txt_rec_model) = {
            let ctx = self.runtime_ctx.read().await;
            let Some(script_info) = ctx.execution.script_info.as_ref() else {
                return Err(Self::execute_error(
                    "action.capture",
                    "当前运行时缺少 script_info，无法生成视觉快照".to_string(),
                ));
            };

            (
                ctx.observation.vision_signature_grid_size,
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

        let det_results = if has_img_det_model {
            self.run_img_det_pipeline("action.capture", Arc::clone(&image))
                .await?
        } else {
            Vec::new()
        };
        let ocr_results = if has_txt_det_model {
            self.run_ocr_pipeline("action.capture", Arc::clone(&image))
                .await?
                .1
        } else {
            Vec::new()
        };

        let mut snapshot = VisionSnapshot::new(det_results.clone(), grid_size).map_err(
            |error| Self::execute_error("action.capture", format!("构建视觉快照失败: {}", error)),
        )?;
        snapshot.set_ocr_results(ocr_results.clone()).map_err(|error| {
            Self::execute_error("action.capture", format!("写入 OCR 结果到视觉快照失败: {}", error))
        })?;

        Ok((det_results, ocr_results, snapshot))
    }

    async fn activate_image_context(
        &mut self,
        step_type: &str,
        image: Arc<RgbaImage>,
        output_var: Option<&str>,
    ) -> ExecuteResult<()> {
        let (det_results, ocr_results, snapshot) =
            self.build_capture_observation(image.clone()).await?;
        let fingerprint = Self::build_page_fingerprint(&snapshot);
        let screen_size = (image.width(), image.height());

        if let Some(output_var) = output_var {
            self.set_runtime_var(output_var, Dynamic::from(image.clone()))
                .await?;
        }
        self.set_runtime_var("runtime.captureResult", Dynamic::from(image.clone()))
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
        ctx.observation.last_capture_image = Some(image.clone());
        ctx.observation.last_vision_input_signature = Some(Self::build_image_signature(
            ctx.observation.capture_asset_signature.as_str(),
            image.as_ref(),
        ));
        ctx.observation.last_det_results = det_results;
        ctx.observation.last_ocr_results = ocr_results;
        ctx.observation.screen_size = screen_size;
        ctx.observation.last_snapshot = Some(snapshot);
        ctx.observation.last_hits.clear();
        drop(ctx);

        self.push_active_policy_page_fingerprint(fingerprint);
        Ok(())
    }

    async fn store_capture_image(
        &mut self,
        image: Arc<RgbaImage>,
        output_var: Option<&str>,
    ) -> ExecuteResult<()> {
        let screen_size = (image.width(), image.height());
        if let Some(output_var) = output_var {
            self.set_runtime_var(output_var, Dynamic::from(image.clone()))
                .await?;
        }

        let mut ctx = self.runtime_ctx.write().await;
        ctx.observation.last_capture_image = Some(image);
        ctx.observation.last_vision_input_signature = None;
        ctx.observation.last_det_results.clear();
        ctx.observation.last_ocr_results.clear();
        ctx.observation.screen_size = screen_size;
        ctx.observation.last_snapshot = None;
        ctx.observation.last_hits.clear();
        Ok(())
    }

    async fn activate_image_var(&mut self, step_type: &str, input_var: &str) -> ExecuteResult<()> {
        let image = self.read_runtime_image_var(input_var, step_type).await?;
        self.activate_image_context(step_type, image, None).await
    }

    async fn activate_runtime_results_context(
        &mut self,
        step_type: &str,
        det_input_var: &str,
        ocr_input_var: &str,
    ) -> ExecuteResult<()> {
        let det_results = self
            .read_runtime_result_vec::<DetResult>(det_input_var, step_type, "检测")
            .await?;
        let ocr_results = self
            .read_runtime_result_vec::<OcrResult>(ocr_input_var, step_type, "OCR")
            .await?;
        let grid_size = {
            let ctx = self.runtime_ctx.read().await;
            ctx.observation.vision_signature_grid_size
        };
        let mut snapshot = VisionSnapshot::new(det_results.clone(), grid_size).map_err(
            |error| Self::execute_error(step_type, format!("构建视觉快照失败: {}", error)),
        )?;
        snapshot.set_ocr_results(ocr_results.clone()).map_err(|error| {
            Self::execute_error(step_type, format!("写入 OCR 结果到视觉快照失败: {}", error))
        })?;
        let fingerprint = Self::build_page_fingerprint(&snapshot);

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
        ctx.observation.last_det_results = det_results;
        ctx.observation.last_ocr_results = ocr_results;
        ctx.observation.last_vision_input_signature = None;
        ctx.observation.last_snapshot = Some(snapshot);
        ctx.observation.last_hits.clear();
        drop(ctx);

        self.push_active_policy_page_fingerprint(fingerprint);
        Ok(())
    }

    async fn execute_detect_step(
        &mut self,
        step_type: &str,
        input_var: &str,
        out_var: &str,
    ) -> ExecuteResult<Vec<DetResult>> {
        let image = self.read_runtime_image_var(input_var, step_type).await?;
        let det_results = self.run_img_det_pipeline(step_type, image.clone()).await?;
        self.store_explicit_vision_results(step_type, image, Some(det_results.clone()), None)
            .await?;
        self.set_runtime_var(
            out_var,
            Self::results_to_dynamic(step_type, "检测", &det_results)?,
        )
        .await?;
        self.set_runtime_var(
            "runtime.detResults",
            Self::results_to_dynamic(step_type, "检测", &det_results)?,
        )
        .await?;
        Ok(det_results)
    }

    async fn execute_ocr_step(
        &mut self,
        step_type: &str,
        input_var: &str,
        out_var: &str,
    ) -> ExecuteResult<Vec<OcrResult>> {
        let image = self.read_runtime_image_var(input_var, step_type).await?;
        let (det_results, ocr_results) = self.run_ocr_pipeline(step_type, image.clone()).await?;
        self.store_explicit_vision_results(
            step_type,
            image,
            Some(det_results),
            Some(ocr_results.clone()),
        )
            .await?;
        self.set_runtime_var(
            out_var,
            Self::results_to_dynamic(step_type, "OCR", &ocr_results)?,
        )
        .await?;
        self.set_runtime_var(
            "runtime.ocrResults",
            Self::results_to_dynamic(step_type, "OCR", &ocr_results)?,
        )
        .await?;
        Ok(ocr_results)
    }

    async fn store_explicit_vision_results(
        &mut self,
        step_type: &str,
        image: Arc<RgbaImage>,
        det_results: Option<Vec<DetResult>>,
        ocr_results: Option<Vec<OcrResult>>,
    ) -> ExecuteResult<()> {
        let (grid_size, capture_asset_signature) = {
            let ctx = self.runtime_ctx.read().await;
            (
                ctx.observation.vision_signature_grid_size,
                ctx.observation.capture_asset_signature.clone(),
            )
        };
        let current_signature =
            Self::build_image_signature(capture_asset_signature.as_str(), image.as_ref());
        let next_det_results = det_results.unwrap_or_default();
        let next_ocr_results = ocr_results.unwrap_or_default();
        let mut snapshot = VisionSnapshot::new(next_det_results.clone(), grid_size).map_err(
            |error| Self::execute_error(step_type, format!("构建视觉快照失败: {}", error)),
        )?;
        snapshot.set_ocr_results(next_ocr_results.clone()).map_err(|error| {
            Self::execute_error(step_type, format!("写入 OCR 结果到视觉快照失败: {}", error))
        })?;
        let fingerprint = Self::build_page_fingerprint(&snapshot);

        let screen_size = (image.width(), image.height());
        let mut ctx = self.runtime_ctx.write().await;
        ctx.observation.last_capture_image = Some(image);
        ctx.observation.last_vision_input_signature = Some(current_signature);
        ctx.observation.last_det_results = next_det_results;
        ctx.observation.last_ocr_results = next_ocr_results;
        ctx.observation.screen_size = screen_size;
        ctx.observation.last_snapshot = Some(snapshot);
        ctx.observation.last_hits.clear();
        drop(ctx);

        self.push_active_policy_page_fingerprint(fingerprint);
        Ok(())
    }

    async fn run_img_det_pipeline(
        &self,
        step_type: &str,
        image: Arc<RgbaImage>,
    ) -> ExecuteResult<Vec<DetResult>> {
        let service = {
            let ctx = self.runtime_ctx.read().await;
            let Some(script_info) = ctx.execution.script_info.as_ref() else {
                return Err(Self::execute_error(
                    step_type,
                    "当前运行时缺少 script_info，无法执行目标检测".to_string(),
                ));
            };
            if script_info.img_det_model.is_none() {
                return Err(Self::execute_error(
                    step_type,
                    "当前脚本未配置图像检测模型".to_string(),
                ));
            }
            ctx.img_det_service.clone()
        };

        Self::run_ocr_service_with_timeout(
            step_type,
            "目标检测",
            VISION_INFERENCE_TIMEOUT_MS,
            service,
            move |service| {
                service
                    .detect_rgba(image.as_ref())
                    .map_err(|error| format!("目标检测执行失败: {}", error))
            },
        )
        .await
    }

    async fn run_ocr_pipeline(
        &self,
        step_type: &str,
        image: Arc<RgbaImage>,
    ) -> ExecuteResult<(Vec<DetResult>, Vec<OcrResult>)> {
        let (service, use_cache, rec_model_signature, cached_ocr_results) = {
            let ctx = self.runtime_ctx.read().await;
            let Some(script_info) = ctx.execution.script_info.as_ref() else {
                return Err(Self::execute_error(
                    step_type,
                    "当前运行时缺少 script_info，无法执行 OCR".to_string(),
                ));
            };
            if script_info.txt_det_model.is_none() || script_info.txt_rec_model.is_none() {
                return Err(Self::execute_error(
                    step_type,
                    "当前脚本未完整配置 OCR 模型".to_string(),
                ));
            }
            let use_cache = ctx.vision_text_cache.is_enabled();
            let (rec_model_signature, cached_ocr_results) = if use_cache {
                (
                    ctx.observation.text_rec_model_signature.clone(),
                    ctx.vision_text_cache
                        .current_entries()
                        .iter()
                        .map(|entry| (entry.cache_key().to_string(), entry.to_ocr_result()))
                        .collect::<std::collections::HashMap<_, _>>(),
                )
            } else {
                (String::new(), std::collections::HashMap::new())
            };

            (
                ctx.ocr_service.clone(),
                use_cache,
                rec_model_signature,
                cached_ocr_results,
            )
        };

        let detect_image = Arc::clone(&image);
        let det_results = Self::run_ocr_service_with_timeout(
            step_type,
            "OCR文字检测",
            VISION_INFERENCE_TIMEOUT_MS,
            service.clone(),
            move |service| {
                service
                    .detect_rgba(detect_image.as_ref())
                    .map_err(|error| format!("OCR 文字检测执行失败: {}", error))
            },
        )
        .await?;

        let ocr_crop_entries = Self::collect_ocr_crop_entries(image.as_ref(), &det_results);
        let ocr_results = if use_cache {
            let mut merged_results = vec![None; det_results.len()];
            let mut missing_indices = Vec::new();
            let mut missing_det_results = Vec::new();
            let mut missing_crops = Vec::new();
            let mut missing_keys = Vec::new();
            for (idx, crop_image) in ocr_crop_entries {
                let cache_key =
                    Self::build_ocr_text_cache_key(&crop_image, rec_model_signature.as_str());
                if let Some(cached) = cached_ocr_results.get(cache_key.as_str()).cloned() {
                    merged_results[idx] = Some(cached);
                    continue;
                }
                if let Some(det_result) = det_results.get(idx).cloned() {
                    missing_indices.push(idx);
                    missing_det_results.push(det_result);
                    missing_crops.push(crop_image);
                    missing_keys.push(cache_key);
                }
            }

            let (miss_results, new_cache_entries) = Self::run_ocr_service_with_timeout(
                step_type,
                "OCR",
                VISION_INFERENCE_TIMEOUT_MS,
                service,
                move |service| {
                    let ocr_results = service
                        .recognize_crops_rgba(missing_crops, &missing_det_results)
                        .map_err(|error| format!("OCR 执行失败: {}", error))?;
                    let mut new_cache_entries = Vec::new();
                    for (offset, ocr_result) in ocr_results.iter().cloned().enumerate() {
                        if let Some(cache_key) = missing_keys.get(offset) {
                            new_cache_entries.push((cache_key.clone(), ocr_result));
                        }
                    }
                    Ok((ocr_results, new_cache_entries))
                },
            )
            .await?;
            if miss_results.len() != missing_indices.len() {
                Log::warn(&format!(
                    "[ executor ] OCR 结果数量与未命中裁图数量不一致: result={}, miss={}",
                    miss_results.len(),
                    missing_indices.len()
                ));
            }
            for (offset, ocr_result) in miss_results.into_iter().enumerate() {
                if let Some(original_index) = missing_indices.get(offset).copied() {
                    merged_results[original_index] = Some(ocr_result);
                }
            }
            if !new_cache_entries.is_empty() {
                let mut ctx = self.runtime_ctx.write().await;
                for (cache_key, ocr_result) in new_cache_entries {
                    if let Err(error) = ctx
                        .vision_text_cache
                        .record_entry(cache_key, ocr_result)
                    {
                        Log::warn(&format!(
                            "[ executor ] 写入 OCR 文字缓存失败，已忽略本次缓存: {}",
                            error
                        ));
                    }
                }
            }
            merged_results.into_iter().flatten().collect::<Vec<_>>()
        } else {
            let mut cropped_images = Vec::with_capacity(ocr_crop_entries.len());
            let mut ocr_det_inputs = Vec::with_capacity(ocr_crop_entries.len());
            for (idx, crop_image) in ocr_crop_entries {
                if let Some(det_result) = det_results.get(idx).cloned() {
                    cropped_images.push(crop_image);
                    ocr_det_inputs.push(det_result);
                }
            }
            Self::run_ocr_service_with_timeout(
                step_type,
                "OCR",
                VISION_INFERENCE_TIMEOUT_MS,
                service,
                move |service| {
                    service
                        .recognize_crops_rgba(cropped_images, &ocr_det_inputs)
                        .map_err(|error| format!("OCR 执行失败: {}", error))
                },
            )
            .await?
        };

        Ok((det_results, ocr_results))
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

    fn build_ocr_text_cache_key(image: &RgbaImage, rec_model_signature: &str) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"1");
        Self::write_hash_segment(&mut hasher, rec_model_signature.as_bytes());
        hasher.write(&image.width().to_le_bytes());
        hasher.write(&image.height().to_le_bytes());
        Self::write_hash_segment(&mut hasher, image.as_raw());
        format!("1:{:016x}", hasher.finish())
    }

    fn collect_ocr_crop_entries(
        image: &RgbaImage,
        det_results: &[DetResult],
    ) -> Vec<(usize, RgbaImage)> {
        det_results
            .iter()
            .enumerate()
            .filter_map(|(idx, det_result)| {
                vision_core::infrastructure::image::crop_image::get_crop_image_rgba(
                    image, det_result,
                )
                .ok()
                .map(|crop| (idx, crop))
            })
            .collect()
    }

    fn build_image_signature(capture_asset_signature: &str, image: &RgbaImage) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"vision-image:v1");
        Self::write_hash_segment(&mut hasher, capture_asset_signature.as_bytes());
        hasher.write(&image.width().to_le_bytes());
        hasher.write(&image.height().to_le_bytes());
        Self::write_hash_segment(&mut hasher, image.as_raw());
        format!("vision-image:v1:{:016x}", hasher.finish())
    }

    pub(crate) fn build_capture_asset_signature(
        script_info: &crate::domain::scripts::script_info::ScriptInfo,
    ) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"capture-assets:v1");
        Self::write_hash_segment(
            &mut hasher,
            Self::detector_asset_signature(script_info.img_det_model.as_ref()).as_bytes(),
        );
        Self::write_hash_segment(
            &mut hasher,
            Self::detector_asset_signature(script_info.txt_det_model.as_ref()).as_bytes(),
        );
        Self::write_hash_segment(
            &mut hasher,
            Self::recognizer_asset_signature(script_info.txt_rec_model.as_ref()).as_bytes(),
        );
        format!("capture-assets:v1:{:016x}", hasher.finish())
    }

    pub(crate) fn build_text_rec_model_signature(
        script_info: &crate::domain::scripts::script_info::ScriptInfo,
    ) -> String {
        let Some(model) = script_info.txt_rec_model.as_ref() else {
            Log::warn("build_text_rec_model_signature: 未找到模型配置，将影响后续依赖该签名的内容！");
            return "none".to_string();
        };

        match model {
            crate::infrastructure::vision::rec::RecognizerType::PaddleCrnn(cfg) => {
                Self::base_model_asset_signature(&cfg.base_model)
            }
        }
    }

    fn write_hash_segment(hasher: &mut XxHash3_64, bytes: &[u8]) {
        hasher.write(&(bytes.len() as u64).to_le_bytes());
        hasher.write(bytes);
    }

    fn detector_asset_signature(
        model: Option<&crate::infrastructure::vision::det::DetectorType>,
    ) -> String {
        let Some(model) = model else {
            return "none".to_string();
        };

        match model {
            crate::infrastructure::vision::det::DetectorType::Yolo11(cfg)
            | crate::infrastructure::vision::det::DetectorType::Yolo26(cfg) => format!(
                "model={};label={}",
                Self::base_model_asset_signature(&cfg.base_model),
                Self::optional_file_asset_signature(cfg.label_path.as_deref())
            ),
            crate::infrastructure::vision::det::DetectorType::PaddleDbNet(cfg) => {
                Self::base_model_asset_signature(&cfg.base_model)
            }
        }
    }

    fn recognizer_asset_signature(
        model: Option<&crate::infrastructure::vision::rec::RecognizerType>,
    ) -> String {
        let Some(model) = model else {
            return "none".to_string();
        };

        match model {
            crate::infrastructure::vision::rec::RecognizerType::PaddleCrnn(cfg) => format!(
                "model={};dict={}",
                Self::base_model_asset_signature(&cfg.base_model),
                Self::resolved_path_asset_signature(cfg.resolved_dict_path())
            ),
        }
    }

    fn base_model_asset_signature(
        model: &vision_core::infrastructure::vision::base_model::BaseModel,
    ) -> String {
        Self::resolved_path_asset_signature(model.resolve_model_path())
    }

    fn optional_file_asset_signature(path: Option<&std::path::Path>) -> String {
        match path {
            Some(path) => Self::file_asset_signature(path),
            None => "none".to_string(),
        }
    }

    fn resolved_path_asset_signature(
        path: vision_core::infrastructure::vision::vision_error::VisionResult<std::path::PathBuf>,
    ) -> String {
        match path {
            Ok(path) => Self::file_asset_signature(&path),
            Err(error) => format!("resolve-error:{}", error),
        }
    }

    fn file_asset_signature(path: &std::path::Path) -> String {
        let path_text = path.display().to_string();
        match Self::sha256_file_hex(path) {
            Ok(hash) => format!("path={};sha256={}", path_text, hash),
            Err(error) => format!("path={};unhashed={}", path_text, error),
        }
    }

    fn sha256_file_hex(path: &std::path::Path) -> Result<String, String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let mut file = std::fs::File::open(path)
            .map_err(|error| format!("open-failed:{}", error))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 8192];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| format!("read-failed:{}", error))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn capture_device_screenshot(&self, step_type: &str) -> ExecuteResult<RgbaImage> {
        let timeout_ms = self.resolve_capture_timeout_ms().await;
        Self::await_device_result_with_timeout(
            step_type,
            "设备截图",
            timeout_ms,
            get_device_ctx().get_screenshot_result(),
        )
        .await
    }

    async fn resolve_capture_timeout_ms(&self) -> u64 {
        let device_ctx = get_device_ctx();
        let device_config = device_ctx.device_config.read().await;
        match &device_config.cap_method {
            runtime_engine::domain::devices::device_conf::CapMethod::Window {
                interface,
                frame_timeout_secs,
                ..
            } if matches!(
                interface,
                runtime_engine::domain::devices::device_conf::WindowCaptureInterface::Dxgi
            ) =>
            {
                u64::from((*frame_timeout_secs).max(1)) * 1_000 + 1_000
            }
            _ => DEVICE_EXTERNAL_TIMEOUT_MS,
        }
    }

    fn is_dxgi_capture_timeout_error(error: &crate::domain::scripts::script_error::ScriptError) -> bool {
        matches!(
            error,
            crate::domain::scripts::script_error::ScriptError::ExecuteErr { step_type, e }
                if step_type == "action.capture"
                    && (e.contains("DXGI 等待新帧超时")
                        || e.contains("DXGI AcquireNextFrame timeout"))
        )
    }

    fn is_window_capture_bind_error(
        error: &crate::domain::scripts::script_error::ScriptError,
    ) -> bool {
        matches!(
            error,
            crate::domain::scripts::script_error::ScriptError::ExecuteErr { step_type, e }
                if step_type == "action.capture" && e.contains("窗口绑定失败")
        )
    }

    fn capture_error_message(
        error: &crate::domain::scripts::script_error::ScriptError,
    ) -> Option<String> {
        match error {
            crate::domain::scripts::script_error::ScriptError::ExecuteErr { step_type, e }
                if step_type == "action.capture" =>
            {
                Some(e.clone())
            }
            _ => None,
        }
    }

    async fn handle_capture_timeout_action(
        &mut self,
        step_type: &str,
        message: String,
    ) -> ExecuteResult<Option<ControlFlow>> {
        let Some(runtime_policy) = get_runtime_execution_policy().await else {
            return Err(Self::execute_error(step_type, message));
        };

        self.emit_timeout_signals(
            runtime_policy.timeout_action.clone(),
            runtime_policy.timeout_notify_channels.clone(),
            None,
            Some("action.capture".to_string()),
            message.clone(),
        )
        .await;
        self.handle_timeout_action(&runtime_policy, message).await
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
        let started = tokio::time::Instant::now();
        tokio::pin!(future);
        loop {
            if crate::infrastructure::context::child_process_sec::stop_requested() {
                return Err(Self::execute_error(
                    step_type,
                    format!("{}已收到停止命令", label),
                ));
            }
            let remaining = Duration::from_millis(timeout_ms).saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(Self::execute_error(
                    step_type,
                    format!("{}超时，超过{}ms", label, timeout_ms),
                ));
            }
            let slice = remaining.min(Duration::from_millis(WAIT_TIMEOUT_CHECK_SLICE_MS));
            tokio::select! {
                result = &mut future => {
                    return match result {
                        Ok(value) => Ok(value),
                        Err(error) => Err(Self::execute_error(step_type, error)),
                    };
                }
                _ = tokio::time::sleep(slice) => {}
            }
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
        let started = tokio::time::Instant::now();
        let mut task = Box::pin(task);
        loop {
            if crate::infrastructure::context::child_process_sec::stop_requested() {
                return Err(Self::execute_error(
                    step_type,
                    format!("{}已收到停止命令", label),
                ));
            }
            let remaining = Duration::from_millis(timeout_ms).saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(Self::execute_error(
                    step_type,
                    format!("{}超时，超过{}ms", label, timeout_ms),
                ));
            }
            let slice = remaining.min(Duration::from_millis(WAIT_TIMEOUT_CHECK_SLICE_MS));
            tokio::select! {
                result = &mut task => {
                    return match result {
                        Ok(Ok(value)) => Ok(value),
                        Ok(Err(error)) => Err(Self::execute_error(step_type, error)),
                        Err(error) => Err(Self::execute_error(
                            step_type,
                            format!("{}任务执行失败: {}", label, error),
                        )),
                    };
                }
                _ = tokio::time::sleep(slice) => {}
            }
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
