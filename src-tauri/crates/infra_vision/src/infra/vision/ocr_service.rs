use crate::infra::vision::base_traits::{ModelHandler, TextDetector, TextRecognizer};
use crate::infra::vision::det::RuntimeDetector;
use crate::infra::vision::rec::RuntimeRecognizer;
use crate::infra::vision::vision_error::{VisionError, VisionResult};
use domain_vision::{DetResult, DetectorType, ModelType, OcrResult, RecognizerType};
use image::{DynamicImage, RgbaImage};
use infra_logging::Log;
use std::sync::Arc;

/// 新的OCR模型管理器 - 使用trait对象来处理不同的模型
#[derive(Debug, Clone)]
pub struct OcrService {
    detector: Option<Arc<dyn TextDetector + Send + Sync>>,
    recognizer: Option<Arc<dyn TextRecognizer + Send + Sync>>,
}

impl OcrService {
    /// 创建新的OCR服务实例
    pub fn new() -> Self {
        Self {
            detector: None,
            recognizer: None,
        }
    }

    /// 使用配置初始化检测器
    pub async fn init_detector(&mut self, config: DetectorType) -> VisionResult<()> {
        Log::info("初始化检测模型...");
        let detector: Arc<dyn TextDetector + Send + Sync> = match RuntimeDetector::from(config) {
            RuntimeDetector::Yolo11(mut yolo) => {
                yolo.base_model.model_type = ModelType::Yolo11;
                //加载标签
                Log::debug("加载yolo标签文件...");
                yolo.load_labels().await?;
                Log::debug("加载yolo检测模型...");
                yolo.load_model()?;
                Arc::new(yolo)
            }
            RuntimeDetector::Yolo26(mut yolo) => {
                yolo.base_model.model_type = ModelType::Yolo26;
                //加载标签
                Log::debug("加载yolo26标签文件...");
                yolo.load_labels().await?;
                Log::debug("加载yolo26检测模型...");
                yolo.load_model()?;
                Arc::new(yolo)
            }
            RuntimeDetector::PaddleDbNet(mut db_net) => {
                Log::info("加载DBNet检测模型...");
                db_net.load_model()?;
                Arc::new(db_net)
            }
        };
        self.detector = Some(detector);
        Ok(())
    }

    /// 使用配置初始化识别器。
    ///
    /// 初始化顺序固定为：加载字典 -> 加载 ONNX 模型 -> 缓存 trait 对象实例。
    pub async fn init_recognizer(&mut self, config: RecognizerType) -> VisionResult<()> {
        Log::info("初始化文字识别模型...");
        let recognizer: Arc<dyn TextRecognizer + Send + Sync> =
            match RuntimeRecognizer::from(config) {
                RuntimeRecognizer::PaddleCrnn(mut crnn) => {
                    // 加载字典
                    Log::debug("加载字典文件...");
                    crnn.load_dict().await?;
                    Log::debug("加载CRNN识别模型...");
                    crnn.load_model()?;
                    Arc::new(crnn)
                }
            };
        self.recognizer = Some(recognizer);
        Ok(())
    }

    /// 只执行文本检测，不做识别。
    pub fn detect(&mut self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        if let Some(ref mut detector) = self.detector {
            Ok(detector.detect(image)?)
        } else {
            Err(VisionError::DetectorNotInit)
        }
    }

    pub fn detect_rgba(&mut self, image: &RgbaImage) -> VisionResult<Vec<DetResult>> {
        if let Some(ref mut detector) = self.detector {
            Ok(detector.detect_rgba(image)?)
        } else {
            Err(VisionError::DetectorNotInit)
        }
    }

    /// 对给定检测框执行逐框识别。
    pub fn recognize(
        &mut self,
        image: &DynamicImage,
        det_result: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        if let Some(ref mut recognizer) = self.recognizer {
            Ok(recognizer.recognize(image, det_result)?)
        } else {
            Err(VisionError::RecognizeNotInit)
        }
    }

    #[cfg(test)]
    pub(crate) fn recognize_crops(
        &mut self,
        cropped_images: Vec<DynamicImage>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        if let Some(ref mut recognizer) = self.recognizer {
            Ok(recognizer.recognize_crops(cropped_images, det_results)?)
        } else {
            Err(VisionError::RecognizeNotInit)
        }
    }

    pub fn recognize_crops_rgba(
        &mut self,
        cropped_images: Vec<RgbaImage>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        if let Some(ref mut recognizer) = self.recognizer {
            Ok(recognizer.recognize_crops_rgba(cropped_images, det_results)?)
        } else {
            Err(VisionError::RecognizeNotInit)
        }
    }

    /// 执行完整 OCR 流程：检测后走逐框识别链路。
    pub fn ocr(&mut self, image: &DynamicImage) -> VisionResult<Vec<OcrResult>> {
        // 1. 首先进行文本检测
        let mut det_result = self.detect(image)?;
        // 2. 对每个检测到的文本区域进行识别
        //let mut results = Vec::new();

        // 这里需要根据DetResult的具体结构来实现
        // 假设DetResult包含了检测到的文本框坐标
        // for text_box in det_result.boxes {
        //     let cropped_image = crop_image(image, &text_box)?;
        //     let text = self.recognize(&cropped_image).await?;
        //     results.push((text_box, text));
        // }

        // 暂时返回整体结果
        let ocr_results = self.recognize(image, &mut *det_result)?;
        //results.push((det_result, text));

        Ok(ocr_results)
    }
    /// 检查服务状态
    #[cfg(test)]
    pub(crate) fn is_ready(&self) -> bool {
        self.detector.is_some() && self.recognizer.is_some()
    }

    /*pub fn detector_info(&self) -> Option<String> {
        if let Some(detector) = &self.detector {
            let config = detector.get_detection_config();
            Some(format!(
                "检测器配置: 置信度阈值={:?}",
                config.confidence_thresh
            ))
        } else {
            None
        }
    }*/

    /*pub fn recognizer_info(&self) -> Option<String> {
        if let Some(recognizer) = &self.recognizer {
            let config = recognizer.get_recognition_config();
            Some(format!("识别器配置: 束搜索宽度={:?}", config.beam_width))
        } else {
            None
        }
    }*/
}

impl Default for OcrService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::image::crop_image::get_crop_images;
    use crate::infra::image::load_image::load_img_from_path;
    use domain_vision::{BoundingBox, OcrResult};
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::hash::Hasher;
    use std::io::Read;
    use std::path::Path;
    use std::time::Instant;
    use twox_hash::XxHash3_64;

    const OCR_TEST_CONFIG_PATH_ENV: &str = "AUTODAILY_OCR_TEST_CONFIG_PATH";

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RealOcrTestConfig {
        detector: DetectorType,
        recognizer: RecognizerType,
        image_path: String,
        print_res: bool,
        #[serde(default)]
        use_cache_results: bool,
        #[serde(default)]
        cache_file_path: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestCacheEntry {
        key: String,
        x1: u32,
        y1: u32,
        x2: u32,
        y2: u32,
        txt: String,
    }

    fn required_env(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| panic!("missing required env var: {}", name))
    }

    fn real_ocr_test_config() -> RealOcrTestConfig {
        let config_path = required_env(OCR_TEST_CONFIG_PATH_ENV);
        let config_text = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|e| panic!("failed to read test config '{}': {}", config_path, e));

        serde_json::from_str(&config_text)
            .unwrap_or_else(|e| panic!("failed to parse test config '{}': {}", config_path, e))
    }

    fn write_hash_segment(hasher: &mut XxHash3_64, bytes: &[u8]) {
        hasher.write(&(bytes.len() as u64).to_le_bytes());
        hasher.write(bytes);
    }

    fn sha256_file_hex(path: &Path) -> Result<String, String> {
        use sha2::{Digest, Sha256};

        let mut file =
            std::fs::File::open(path).map_err(|error| format!("open-failed:{}", error))?;
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

    fn file_asset_signature(path: &Path) -> String {
        let path_text = path.display().to_string();
        match sha256_file_hex(path) {
            Ok(hash) => format!("path={};sha256={}", path_text, hash),
            Err(error) => format!("path={};unhashed={}", path_text, error),
        }
    }

    fn recognizer_model_signature(config: &RecognizerType) -> String {
        match config {
            RecognizerType::PaddleCrnn(cfg) => {
                crate::infra::vision::base_model::resolve_model_config_path(&cfg.base_model)
                    .map(|path| file_asset_signature(&path))
                    .unwrap_or_else(|error| format!("resolve-error:{}", error))
            }
        }
    }

    fn test_cache_key(image: &DynamicImage, rec_model_signature: &str) -> String {
        let rgba = image.to_rgba8();
        let mut hasher = XxHash3_64::default();
        hasher.write(b"1");
        write_hash_segment(&mut hasher, rec_model_signature.as_bytes());
        hasher.write(&rgba.width().to_le_bytes());
        hasher.write(&rgba.height().to_le_bytes());
        write_hash_segment(&mut hasher, rgba.as_raw());
        format!("1:{:016x}", hasher.finish())
    }

    fn test_cache_entry_to_ocr_result(entry: TestCacheEntry) -> OcrResult {
        OcrResult::new(
            BoundingBox::new(
                entry.x1 as i32,
                entry.y1 as i32,
                entry.x2 as i32,
                entry.y2 as i32,
            ),
            entry.txt,
            Vec::new(),
            Vec::new(),
            1,
        )
    }

    fn ocr_result_to_test_cache_entry(cache_key: &str, ocr_result: &OcrResult) -> TestCacheEntry {
        TestCacheEntry {
            key: cache_key.to_string(),
            x1: ocr_result.bounding_box.x1.max(0) as u32,
            y1: ocr_result.bounding_box.y1.max(0) as u32,
            x2: ocr_result.bounding_box.x2.max(0) as u32,
            y2: ocr_result.bounding_box.y2.max(0) as u32,
            txt: ocr_result.txt.clone(),
        }
    }

    fn quote_field(value: &str) -> String {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    }

    fn parse_quoted_field(value: &str) -> String {
        let mut output = String::new();
        let mut escaped = false;
        for ch in value[1..value.len() - 1].chars() {
            if escaped {
                output.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else {
                output.push(ch);
            }
        }
        output
    }

    fn split_cache_line(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut escaped = false;
        for ch in line.chars() {
            if escaped {
                current.push(ch);
                escaped = false;
                continue;
            }
            match ch {
                '\\' if in_quotes => {
                    current.push(ch);
                    escaped = true;
                }
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(ch);
                }
                ',' if !in_quotes => {
                    fields.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(ch),
            }
        }
        fields.push(current.trim().to_string());
        fields
    }

    fn parse_test_cache_entry(line: &str) -> TestCacheEntry {
        let fields = split_cache_line(line);
        assert_eq!(fields.len(), 6, "cache line should have 6 fields");
        TestCacheEntry {
            key: parse_quoted_field(&fields[0]),
            x1: fields[1].parse().expect("x1 should be u32"),
            y1: fields[2].parse().expect("y1 should be u32"),
            x2: fields[3].parse().expect("x2 should be u32"),
            y2: fields[4].parse().expect("y2 should be u32"),
            txt: parse_quoted_field(&fields[5]),
        }
    }

    fn format_test_cache_entry(entry: &TestCacheEntry) -> String {
        format!(
            "{},{},{},{},{},{}",
            quote_field(&entry.key),
            entry.x1,
            entry.y1,
            entry.x2,
            entry.y2,
            quote_field(&entry.txt)
        )
    }

    fn load_test_cache_map(cache_file_path: &str) -> HashMap<String, OcrResult> {
        let cache_path = Path::new(cache_file_path);
        if !cache_path.exists() {
            if let Some(parent) = cache_path.parent() {
                std::fs::create_dir_all(parent).unwrap_or_else(|error| {
                    panic!(
                        "failed to create cache dir for '{}': {}",
                        cache_file_path, error
                    )
                });
            }
            std::fs::write(cache_path, "").unwrap_or_else(|error| {
                panic!(
                    "failed to create empty cache file '{}': {}",
                    cache_file_path, error
                )
            });
        }
        let text = std::fs::read_to_string(cache_file_path).unwrap_or_else(|error| {
            panic!("failed to read cache file '{}': {}", cache_file_path, error)
        });
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(parse_test_cache_entry)
            .into_iter()
            .map(|entry| {
                let key = entry.key.clone();
                (key, test_cache_entry_to_ocr_result(entry))
            })
            .collect()
    }

    fn write_test_cache_map(cache_file_path: &str, cache_store: &HashMap<String, OcrResult>) {
        let mut entries = cache_store
            .iter()
            .map(|(cache_key, ocr_result)| ocr_result_to_test_cache_entry(cache_key, ocr_result))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.key.cmp(&right.key));
        let text = entries
            .iter()
            .map(format_test_cache_entry)
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(cache_file_path, text).unwrap_or_else(|error| {
            panic!(
                "failed to write cache file '{}': {}",
                cache_file_path, error
            )
        });
    }

    #[tokio::test]
    #[ignore = "requires AUTODAILY_OCR_TEST_CONFIG_PATH pointing to a real-path JSON config"]
    async fn detect() {
        let config = real_ocr_test_config();

        let mut service = OcrService::new();

        service
            .init_detector(config.detector)
            .await
            .expect("detector should initialize with real model path");

        let image = load_img_from_path(&config.image_path).expect("failed to load test image");
        let detect_start = Instant::now();
        let det_results = service
            .detect(&image)
            .expect("detect should run with real paths");
        let detect_elapsed = detect_start.elapsed();

        println!("detect elapsed: {:.3?}", detect_elapsed);
        println!("detector result count: {}", det_results.len());
        if config.print_res {
            for (idx, item) in det_results.iter().enumerate() {
                println!(
                    "#{idx}: label='{}' class={} score={:.4} box=({}, {}, {}, {})",
                    item.label,
                    item.index,
                    item.score,
                    item.bounding_box.x1,
                    item.bounding_box.y1,
                    item.bounding_box.x2,
                    item.bounding_box.y2
                );
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires AUTODAILY_OCR_TEST_CONFIG_PATH pointing to a real-path JSON config"]
    async fn ocr() {
        let config = real_ocr_test_config();
        let config_print = real_ocr_test_config();
        let recognizer_signature = recognizer_model_signature(&config_print.recognizer);

        let mut service = OcrService::new();

        service
            .init_detector(config.detector)
            .await
            .expect("detector should initialize with real model path");
        service
            .init_recognizer(config.recognizer)
            .await
            .expect("recognizer should initialize with real model path");

        let image = load_img_from_path(&config.image_path).expect("failed to load test image");
        let detect_start = Instant::now();
        let mut det_results = service
            .detect(&image)
            .expect("detect should run with real paths");
        let detect_elapsed = detect_start.elapsed();

        let recognize_start = Instant::now();
        let ocr_results = service
            .recognize(&image, &mut det_results)
            .expect("recognize should run with real paths");
        let recognize_elapsed = recognize_start.elapsed();
        let total_elapsed = detect_elapsed + recognize_elapsed;
        let mut cache_lookup_elapsed = None;
        let mut cache_hit_count = None;
        let mut cache_miss_count = None;
        let mut cache_merged_count = None;

        if config.use_cache_results {
            let cache_file_path = config.cache_file_path.as_deref().unwrap_or_else(|| {
                panic!("cacheFilePath is required when useCacheResults is true")
            });
            println!("ocr cache file: {}", cache_file_path);
            let mut cache_store = load_test_cache_map(cache_file_path);
            let cropped_images = get_crop_images(&image, &det_results)
                .expect("failed to crop OCR images for cache test");

            let cache_lookup_start = Instant::now();
            let mut hit_count = 0_usize;
            let mut merged_results = vec![None; det_results.len()];
            let mut missing_cache_keys = Vec::new();
            let mut missing_indices = Vec::new();
            let mut missing_det_results = Vec::new();
            let mut missing_crops = Vec::new();
            for (idx, (crop_image, det_result)) in cropped_images
                .into_iter()
                .zip(det_results.iter())
                .enumerate()
            {
                let cache_key = test_cache_key(&crop_image, recognizer_signature.as_str());
                if let Some(cached) = cache_store.get(cache_key.as_str()).cloned() {
                    hit_count += 1;
                    merged_results[idx] = Some(cached);
                } else {
                    missing_cache_keys.push(cache_key);
                    missing_indices.push(idx);
                    missing_det_results.push(det_result.clone());
                    missing_crops.push(crop_image);
                }
            }
            let miss_results = service
                .recognize_crops(missing_crops, &missing_det_results)
                .expect("recognize_crops should run for uncached OCR results");
            for (offset, ocr_result) in miss_results.into_iter().enumerate() {
                if let Some(cache_key) = missing_cache_keys.get(offset) {
                    cache_store.insert(cache_key.clone(), ocr_result.clone());
                }
                if let Some(original_index) = missing_indices.get(offset).copied() {
                    merged_results[original_index] = Some(ocr_result);
                }
            }
            let cached_results: Vec<_> = merged_results.into_iter().flatten().collect();
            write_test_cache_map(cache_file_path, &cache_store);
            cache_lookup_elapsed = Some(cache_lookup_start.elapsed());
            cache_hit_count = Some(hit_count);
            cache_miss_count = Some(missing_indices.len());
            cache_merged_count = Some(cached_results.len());
        }

        let det_conf = match config_print.detector {
            DetectorType::Yolo11(ref cfg) | DetectorType::Yolo26(ref cfg) => (
                cfg.base_model.intra_thread_num,
                cfg.base_model.intra_spinning,
                cfg.base_model.inter_thread_num,
                cfg.base_model.inter_spinning,
                cfg.base_model.execution_provider.as_str(),
                cfg.base_model.input_width,
                cfg.base_model.input_height,
            ),
            DetectorType::PaddleDbNet(ref cfg) => (
                cfg.base_model.intra_thread_num,
                cfg.base_model.intra_spinning,
                cfg.base_model.inter_thread_num,
                cfg.base_model.inter_spinning,
                cfg.base_model.execution_provider.as_str(),
                cfg.base_model.input_width,
                cfg.base_model.input_height,
            ),
        };
        let rec_conf = match config_print.recognizer {
            RecognizerType::PaddleCrnn(ref cfg) => (
                cfg.base_model.intra_thread_num,
                cfg.base_model.intra_spinning,
                cfg.base_model.inter_thread_num,
                cfg.base_model.inter_spinning,
                cfg.parallel_cpu_session_intra_threads,
                cfg.base_model.execution_provider.as_str(),
                cfg.base_model.input_width,
                cfg.base_model.input_height,
            ),
        };

        println!(
            "det: intraThreadNum: {}, intraSpinning:{}, interThreadNum: {}, interSpinning:{}, provider: {}, inputWidth: {}, inputHeight: {}",
            det_conf.0, det_conf.1, det_conf.2, det_conf.3, det_conf.4, det_conf.5, det_conf.6,
        );
        println!(
            "rec: intraThreadNum: {}, intraSpinning:{}, interThreadNum: {}, interSpinning:{}, singleSessionIntraThreadNum: {}, provider: {}, inputWidth: {}, inputHeight: {}",
            rec_conf.0,
            rec_conf.1,
            rec_conf.2,
            rec_conf.3,
            rec_conf.4,
            rec_conf.5,
            rec_conf.6,
            rec_conf.7
        );
        println!(
            "service: {},  detect:{:.3?},  rec:{:.3?},  ocr:{:.3?},  det_count:{},  ocr_count:{}",
            service.is_ready(),
            detect_elapsed,
            recognize_elapsed,
            total_elapsed,
            det_results.len(),
            ocr_results.len()
        );
        if let Some(elapsed) = cache_lookup_elapsed {
            println!(
                "ocr_cache: lookup={:.3?}, hits={}, misses={}, merged_count={}",
                elapsed,
                cache_hit_count.unwrap_or_default(),
                cache_miss_count.unwrap_or_default(),
                cache_merged_count.unwrap_or_default()
            );
        }
        if config.print_res {
            for (idx, item) in det_results.iter().enumerate() {
                println!("#{idx},{}", item);
            }
            for (idx, item) in ocr_results.iter().enumerate() {
                println!(
                    "#{idx}: txt='{}' score={:?} box=({}, {}, {}, {})",
                    item.txt,
                    item.score,
                    item.bounding_box.x1,
                    item.bounding_box.y1,
                    item.bounding_box.x2,
                    item.bounding_box.y2
                );
            }
        }
    }
}
