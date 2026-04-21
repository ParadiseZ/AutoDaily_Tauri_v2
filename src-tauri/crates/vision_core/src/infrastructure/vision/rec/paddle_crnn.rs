use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::image::crop_image::get_crop_image;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::base_model::{BaseModel, ModelSource};
use crate::infrastructure::vision::base_traits::{ModelHandler, TextRecognizer};
use crate::infrastructure::vision::tensor_view::select_batch_and_squeeze_to_2d;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::imageops::FilterType;
use image::DynamicImage;
use imageproc::drawing::Canvas;
use std::collections::BTreeMap;
use std::path::PathBuf;

use ndarray::{Array3, Array4, ArrayD, ArrayView2, ArrayViewD, ArrayViewMut3, Axis};
use rayon::prelude::*;
use tokio::fs::read_to_string;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum RecResizeFilter {
    Nearest,
    Triangle,
    Gaussian,
    CatmullRom,
    Lanczos3,
}

impl Default for RecResizeFilter {
    fn default() -> Self {
        Self::Triangle
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum RecProcessingMode {
    Single,
    MicroBatch,
}

impl Default for RecProcessingMode {
    fn default() -> Self {
        Self::Single
    }
}

#[derive(Debug)]
struct RecSample {
    original_index: usize,
    image: DynamicImage,
    target_width: u32,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaddleRecCrnn {
    pub base_model: BaseModel,
    #[ts(as = "Option<String>")]
    pub dict_path: Option<PathBuf>,
    #[serde(default)]
    pub resize_filter: RecResizeFilter,
    #[serde(default)]
    pub processing_mode: RecProcessingMode,
    #[serde(default = "PaddleRecCrnn::default_micro_batch_size")]
    pub micro_batch_size: usize,
    #[serde(default = "PaddleRecCrnn::default_width_bucket_step")]
    pub width_bucket_step: u32,
    #[serde(skip)]
    #[ts(skip)]
    pub dict: Vec<String>,
}

impl PaddleRecCrnn {
    /// 默认 micro-batch 大小。
    pub const fn default_micro_batch_size() -> usize {
        4
    }

    /// 默认宽度分桶步长。
    pub const fn default_width_bucket_step() -> u32 {
        32
    }

    fn active_filter(&self) -> FilterType {
        match self.resize_filter {
            RecResizeFilter::Nearest => FilterType::Nearest,
            RecResizeFilter::Triangle => FilterType::Triangle,
            RecResizeFilter::Gaussian => FilterType::Gaussian,
            RecResizeFilter::CatmullRom => FilterType::CatmullRom,
            RecResizeFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }

    fn align_width(width: u32) -> u32 {
        width.checked_add(7).map(|v| v & !7).unwrap_or(width).max(8)
    }

    fn calc_target_width(&self, img: &DynamicImage) -> u32 {
        let (origin_w, origin_h) = img.dimensions();
        let target_height = self.get_target_height();
        let ratio = origin_h as f32 / target_height as f32;
        let resize_w = (origin_w as f32 / ratio).ceil() as u32;
        Self::align_width(resize_w)
    }

    fn bucket_width(&self, target_width: u32) -> u32 {
        let step = Self::align_width(self.width_bucket_step.max(8));
        target_width
            .checked_add(step - 1)
            .map(|value| value / step * step)
            .unwrap_or(target_width)
            .max(step)
    }

    fn resolve_dict_path(&self) -> VisionResult<PathBuf> {
        if let Some(path) = self.dict_path.clone() {
            return Ok(path);
        }

        if self.base_model.model_source == ModelSource::BuiltIn {
            let relative = PathBuf::from("ppocr").join("ch_v5_dict.txt");
            let mut candidates = vec![
                PathBuf::from("src-tauri").join("models").join(&relative),
                PathBuf::from("models").join(&relative),
                PathBuf::from("resources").join("models").join(&relative),
            ];

            if let Ok(current_exe) = std::env::current_exe() {
                if let Some(exe_dir) = current_exe.parent() {
                    candidates.push(exe_dir.join("models").join(&relative));
                    candidates.push(exe_dir.join("resources").join("models").join(&relative));
                }
            }

            return candidates
                .into_iter()
                .find(|path| path.exists())
                .ok_or_else(|| VisionError::IoError {
                    path: relative.to_string_lossy().to_string(),
                    e: "未找到内置识别字典文件".to_string(),
                });
        }

        Err(VisionError::IoError {
            path: "".to_string(),
            e: "字典路径为空".to_string(),
        })
    }

    fn fill_image_tensor(
        &self,
        img: &DynamicImage,
        resize_width: u32,
        padded_width: u32,
        mut img_view: ArrayViewMut3<'_, f32>,
    ) {
        let resized_img =
            img.resize_exact(resize_width, self.get_target_height(), self.active_filter());
        let rgb_view = resized_img.to_rgb8();
        let target_height_usize = self.get_target_height() as usize;
        let resize_width_usize = resize_width as usize;
        let padded_width_usize = padded_width as usize;

        for y in 0..target_height_usize {
            for x in 0..resize_width_usize {
                let p = rgb_view.get_pixel(x as u32, y as u32);
                img_view[[0, y, x]] = (p[0] as f32 / 255.0 - 0.5) / 0.5;
                img_view[[1, y, x]] = (p[1] as f32 / 255.0 - 0.5) / 0.5;
                img_view[[2, y, x]] = (p[2] as f32 / 255.0 - 0.5) / 0.5;
            }
            for x in resize_width_usize..padded_width_usize {
                img_view[[0, y, x]] = -1.0;
                img_view[[1, y, x]] = -1.0;
                img_view[[2, y, x]] = -1.0;
            }
        }
    }

    /// 根据检测框裁出文本图，并提前计算每个样本的目标宽度。
    fn collect_samples(&self, image: &DynamicImage, det_results: &[DetResult]) -> Vec<RecSample> {
        let mut samples: Vec<_> = det_results
            .par_iter()
            .enumerate()
            .filter_map(|(idx, det_res)| {
                get_crop_image(image, det_res).ok().map(|crop| RecSample {
                    original_index: idx,
                    target_width: self.calc_target_width(&crop),
                    image: crop,
                })
            })
            .collect();

        samples.sort_by_key(|item| item.original_index);
        samples
    }

    /// 执行单张识别链路。
    ///
    /// 这条路径会保留原始检测框索引，按“预处理 -> 串行推理 -> 原索引回填后处理”执行。
    fn recognize_single_samples(
        &self,
        samples: &[RecSample],
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        // Single 模式走“裁剪 -> 单张预处理 -> 串行推理 -> 原索引回填”。
        let preprocessed_inputs: Vec<(usize, ArrayD<f32>)> = samples
            .par_iter()
            .filter_map(|sample| {
                self.preprocess(&sample.image)
                    .ok()
                    .map(|input| (sample.original_index, input.0))
            })
            .collect();

        self.recognize_preprocessed_inputs(preprocessed_inputs, det_results)
    }

    /// 将一个 micro-batch 的样本拼成统一宽度的批量输入张量。
    fn preprocess_sample_batch(
        &self,
        samples: &[RecSample],
        padded_width: u32,
    ) -> VisionResult<ArrayD<f32>> {
        if samples.is_empty() {
            return Err(VisionError::InputImageCollectionEmpty);
        }

        let batch_size = samples.len();
        let target_height_usize = self.get_target_height() as usize;
        let padded_width_usize = padded_width as usize;
        let mut batch_data =
            Array4::<f32>::zeros((batch_size, 3, target_height_usize, padded_width_usize));
        let chunk_len = 3 * target_height_usize * padded_width_usize;

        if let Some(flat_data) = batch_data.as_slice_mut() {
            flat_data
                .par_chunks_mut(chunk_len)
                .zip(samples.par_iter())
                .for_each(|(chunk, sample)| {
                    let img_view = ArrayViewMut3::from_shape(
                        (3, target_height_usize, padded_width_usize),
                        chunk,
                    )
                    .expect("文字识别失败：批预处理失败【crnn preprocess_sample_batch】");
                    self.fill_image_tensor(
                        &sample.image,
                        sample.target_width,
                        padded_width,
                        img_view,
                    );
                });
        } else {
            Log::error("文字识别失败：批预处理失败: Batch data 切片失败！");
        }

        Ok(batch_data.into_dyn())
    }

    /// 执行按宽度分桶的 micro-batch 识别链路。
    ///
    /// 这条路径用于减少“极宽文本拖慢整批”的问题。
    fn recognize_micro_batches(
        &self,
        samples: Vec<RecSample>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        // Micro-batch 模式先按宽度分桶，再在桶内做小批次推理，避免被极宽文本拖慢整批。
        let mut buckets: BTreeMap<u32, Vec<RecSample>> = BTreeMap::new();
        for sample in samples {
            buckets
                .entry(self.bucket_width(sample.target_width))
                .or_default()
                .push(sample);
        }

        let mut results: Vec<(usize, OcrResult)> = Vec::new();
        let batch_limit = self.micro_batch_size.max(1);

        for (_bucket, bucket_samples) in buckets.iter_mut() {
            bucket_samples.sort_by_key(|sample| (sample.target_width, sample.original_index));

            for chunk in bucket_samples.chunks(batch_limit) {
                let padded_width = chunk
                    .iter()
                    .map(|sample| sample.target_width)
                    .max()
                    .unwrap_or(8);
                let input = self.preprocess_sample_batch(chunk, padded_width)?;
                let output = self.inference(input.view())?;

                for (batch_index, sample) in chunk.iter().enumerate() {
                    let det_res = det_results.get(sample.original_index).ok_or_else(|| {
                        VisionError::BatchMatchDetSizeFailed {
                            batch: chunk.len(),
                            det_num: det_results.len(),
                        }
                    })?;
                    let ocr = self.postprocess(output.view(), det_res, batch_index)?;
                    results.push((sample.original_index, ocr));
                }
            }
        }

        results.sort_by_key(|(idx, _)| *idx);
        Ok(results.into_iter().map(|(_, item)| item).collect())
    }

    /// 从识别模型输出中取出单个样本的 `[T, C]` 序列视图。
    fn extract_sequence_view<'a>(
        &self,
        output: ArrayViewD<'a, f32>,
        batch_index: usize,
    ) -> VisionResult<ArrayView2<'a, f32>> {
        select_batch_and_squeeze_to_2d(output, batch_index, "rec_extract_sequence_view")
    }

    /// 加载识别字典。
    ///
    /// 会保留原始行内容，只在首行移除 BOM，避免把空格字符误删。
    pub async fn load_dict(&mut self) -> VisionResult<()> {
        let dict_path = self.resolve_dict_path()?;

        let content = read_to_string(dict_path.clone())
            .await
            .map_err(|e| VisionError::IoError {
                path: dict_path.to_string_lossy().to_string(),
                e: e.to_string(),
            })?;

        let mut dict = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let value = if idx == 0 {
                line.trim_start_matches('\u{feff}').to_string()
            } else {
                line.to_string()
            };
            dict.push(value);
        }

        if dict.is_empty() {
            return Err(VisionError::IoError {
                path: dict_path.to_string_lossy().to_string(),
                e: "字典文件为空".to_string(),
            });
        }
        self.dict_path = Some(dict_path);
        self.dict = dict;

        Ok(())
    }
}

impl ModelHandler for PaddleRecCrnn {
    fn load_model(&mut self) -> VisionResult<()> {
        self.base_model.load_model_base::<Self>("paddle_rec_crnn")
    }
    fn get_input_size(&self) -> (u32, u32) {
        (self.base_model.input_width, self.base_model.input_height)
    }

    fn preprocess(&self, img: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])> {
        // 获取原始图像尺寸
        let (origin_w, origin_h) = img.dimensions();
        let target_height = self.get_target_height();
        // 计算调整大小的比例
        let ratio = origin_h as f32 / target_height as f32;
        let target_width = self.calc_target_width(img);

        Log::debug(&format!(
            "Rec缩放: 原始={}x{}, 调整后={}x{}, 填充后={}x{}",
            origin_w, origin_h, target_width, target_height, target_width, target_height
        ));

        // 初始化输入数组 (使用带填充的宽度)
        let mut input = Array3::<f32>::zeros((3, target_height as usize, target_width as usize));
        let img_view = input.view_mut();
        self.fill_image_tensor(img, target_width, target_width, img_view);

        // 扩展到批次维度 (1, C, H, W)
        let input = input.insert_axis(Axis(0));

        Ok((input.into_dyn(), [ratio, ratio], [origin_h, origin_w]))
    }

    fn inference(&self, input: ArrayViewD<f32>) -> VisionResult<ArrayD<f32>> {
        // 使用通用推理方法，消除代码重复
        self.base_model.inference_base(
            input,
            self.get_input_node_name(),
            self.get_output_node_name(),
        )
    }

    fn get_input_node_name(&self) -> &'static str {
        "x"
    }

    fn get_output_node_name(&self) -> &'static str {
        "fetch_name_0"
    }

    fn get_target_width(&self) -> u32 {
        self.base_model.input_width
    }

    fn get_target_height(&self) -> u32 {
        self.base_model.input_height
    }
}

impl TextRecognizer for PaddleRecCrnn {
    fn recognize(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let samples = self.collect_samples(image, det_results);
        if samples.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-裁剪：部分图像裁剪失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    samples.len()
                )
                .as_str(),
            );
        }
        if samples.is_empty() {
            return Ok(Vec::new());
        }

        match self.processing_mode {
            RecProcessingMode::Single => self.recognize_single_samples(&samples, det_results),
            RecProcessingMode::MicroBatch => self.recognize_micro_batches(samples, det_results),
        }
    }

    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        det_result: &DetResult,
        batch_size: usize,
    ) -> VisionResult<OcrResult> {
        // 识别后处理只关心 [T, C]，上面先把 [B, T, C] / [B, 1, T, C] 等布局规整成 2D。
        let sequence = self.extract_sequence_view(output, batch_size)?;
        let seq_len = sequence.shape()[0];
        let class_num = sequence.shape()[1];

        if self.dict.len() + 1 != class_num - 1 {
            // 假设最后一个是空白
            return Err(VisionError::DictSizeErr {
                out: class_num,
                dict: self.dict.len(),
            });
        }

        let mut result_text = String::new();
        let mut chars = Vec::new();
        let mut scores = Vec::new();
        let mut indexes = Vec::new();
        let blank_idx = class_num - 1;
        for t in 0..seq_len {
            let mut max_idx = 0;
            let mut max_prob = sequence[[t, 0]];

            for c in 1..class_num {
                let prob = sequence[[t, c]];
                if prob > max_prob {
                    max_prob = prob;
                    max_idx = c;
                }
            }

            if max_idx != blank_idx && max_idx > 0 {
                // 不是空白
                if max_idx <= self.dict.len() {
                    let char = &self.dict[max_idx - 1];
                    scores.push(max_prob);
                    indexes.push(max_idx - 1);
                    result_text.push_str(char);
                    chars.push(char.into())
                }
            }
        }
        let ocr_result = OcrResult {
            bounding_box: det_result.bounding_box.clone(),
            stable_box: det_result.stable_box.clone(),
            stable_center: det_result.stable_center.clone(),
            txt: result_text,
            score: scores,
            index: indexes,
            txt_char: chars,
        };
        Ok(ocr_result)
    }

    fn preprocess_batch(&self, images: &[DynamicImage]) -> VisionResult<ArrayD<f32>> {
        if images.is_empty() {
            return Err(VisionError::InputImageCollectionEmpty);
        }

        let widths: Vec<u32> = images
            .par_iter()
            .map(|img: &DynamicImage| self.calc_target_width(img))
            .collect();
        let max_width = widths.iter().copied().max().unwrap_or(8);

        let batch_size = images.len();
        let target_height_usize = self.get_target_height() as usize;
        let max_width_usize = max_width as usize;

        let mut batch_data =
            Array4::<f32>::zeros((batch_size, 3, target_height_usize, max_width_usize));

        let chunk_len = 3 * target_height_usize * max_width_usize;

        if let Some(flat_data) = batch_data.as_slice_mut() {
            flat_data
                .par_chunks_mut(chunk_len)
                .zip(images.par_iter())
                .zip(widths.par_iter())
                .for_each(|((chunk, img), &width)| {
                    let img_view = ArrayViewMut3::from_shape(
                        (3, target_height_usize, max_width_usize),
                        chunk,
                    )
                    .expect("文字识别失败：批预处理失败【crnn preprocess_batch】Failed to create view from chunk");
                    self.fill_image_tensor(img, width, max_width, img_view);
                });
        } else {
            Log::error("文字识别失败：批预处理失败: Batch data 切片失败！");
        }

        Ok(batch_data.into_dyn())
    }

    fn postprocess_batch(
        &self,
        output: ArrayViewD<f32>,
        det_result: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let batch_size = output.shape()[0];

        if batch_size != det_result.len() {
            return Err(VisionError::BatchMatchDetSizeFailed {
                batch: batch_size,
                det_num: det_result.len(),
            });
        }

        let ocr_res: Vec<OcrResult> = det_result
            .into_par_iter()
            .enumerate()
            .filter_map(|(i, det_res)| self.postprocess(output.view(), det_res, i).ok())
            .collect();
        if ocr_res.len() != batch_size {
            Log::error("识别部分行的文字错误！");
        }
        Ok(ocr_res)
    }

    fn recognize_batch(
        &self,
        image: &DynamicImage,
        det_results: &mut [DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let samples = self.collect_samples(image, det_results);
        if samples.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-批处理裁剪：部分图像裁剪失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    samples.len()
                )
                .as_str(),
            );
        }
        if samples.is_empty() {
            return Ok(Vec::new());
        }
        self.recognize_micro_batches(samples, det_results)
    }
}
