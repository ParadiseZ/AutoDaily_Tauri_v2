use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::base_model::{BaseModel, ModelSource, ModelType};
use crate::infrastructure::vision::base_traits::{ModelHandler, TextRecognizer};
use crate::infrastructure::vision::tensor_view::select_batch_and_squeeze_to_2d;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::collections::BTreeMap;
use std::path::PathBuf;

use ndarray::{Array3, Array4, ArrayD, ArrayView2, ArrayViewD, ArrayViewMut3, Axis};
use rayon::prelude::*;
use tokio::fs::read_to_string;

const REC_SCALE: f32 = 2.0 / 255.0;

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
    image: RgbaImage,
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
    #[serde(default = "PaddleRecCrnn::default_parallel_cpu_session_intra_threads")]
    pub parallel_cpu_session_intra_threads: usize,
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

    pub const fn default_parallel_cpu_session_intra_threads() -> usize {
        1
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

    fn calc_target_width_from_dims(&self, origin_w: u32, origin_h: u32) -> u32 {
        let target_height = self.get_target_height();
        let ratio = origin_h as f32 / target_height as f32;
        let resize_w = (origin_w as f32 / ratio).ceil() as u32;
        Self::align_width(resize_w)
    }

    fn calc_target_width(&self, img: &DynamicImage) -> u32 {
        let (origin_w, origin_h) = GenericImageView::dimensions(img);
        self.calc_target_width_from_dims(origin_w, origin_h)
    }

    fn calc_target_width_rgba(&self, img: &RgbaImage) -> u32 {
        let (origin_w, origin_h) = img.dimensions();
        self.calc_target_width_from_dims(origin_w, origin_h)
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
            let relative = match self.base_model.model_type {
                ModelType::PaddleCrnn5 => PathBuf::from("ppocr").join("ch_v5_dict.txt"),
                ModelType::PaddleCrnn6 => PathBuf::from("ppocr").join("v6_dict.txt"),
                _ => {
                    Log::error("内置模型加载失败：未找到内置识别字典文件");
                    return Err(VisionError::IoError {
                        path: "".to_string(),
                        e: format!(
                            "类型：{:?}, 未找到内置识别字典文件",
                            self.base_model.model_type
                        ),
                    });
                }
            };
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

    pub fn resolved_dict_path(&self) -> VisionResult<PathBuf> {
        self.resolve_dict_path()
    }

    fn fill_image_tensor(
        &self,
        img: &DynamicImage,
        resize_width: u32,
        padded_width: u32,
        mut img_view: ArrayViewMut3<'_, f32>,
    ) {
        let resized_img = img
            .resize_exact(resize_width, self.get_target_height(), self.active_filter())
            .to_rgb8();
        let target_height_usize = self.get_target_height() as usize;
        let resize_width_usize = resize_width as usize;
        let padded_width_usize = padded_width as usize;
        let plane_len = target_height_usize * padded_width_usize;

        if let Some(buffer) = img_view.as_slice_mut() {
            buffer.fill(-1.0);
            let (r_plane, rest) = buffer.split_at_mut(plane_len);
            let (g_plane, b_plane) = rest.split_at_mut(plane_len);
            for (y, row) in resized_img
                .as_raw()
                .chunks_exact(resize_width_usize * 3)
                .enumerate()
            {
                let row_offset = y * padded_width_usize;
                for (x, pixel) in row.chunks_exact(3).enumerate() {
                    let idx = row_offset + x;
                    r_plane[idx] = pixel[0] as f32 * REC_SCALE - 1.0;
                    g_plane[idx] = pixel[1] as f32 * REC_SCALE - 1.0;
                    b_plane[idx] = pixel[2] as f32 * REC_SCALE - 1.0;
                }
            }
        } else {
            for y in 0..target_height_usize {
                for x in 0..resize_width_usize {
                    let idx = (y * resize_width_usize + x) * 3;
                    let p = &resized_img.as_raw()[idx..idx + 3];
                    img_view[[0, y, x]] = p[0] as f32 * REC_SCALE - 1.0;
                    img_view[[1, y, x]] = p[1] as f32 * REC_SCALE - 1.0;
                    img_view[[2, y, x]] = p[2] as f32 * REC_SCALE - 1.0;
                }
                for x in resize_width_usize..padded_width_usize {
                    img_view[[0, y, x]] = -1.0;
                    img_view[[1, y, x]] = -1.0;
                    img_view[[2, y, x]] = -1.0;
                }
            }
        }
    }

    fn fill_image_tensor_rgba(
        &self,
        img: &RgbaImage,
        resize_width: u32,
        padded_width: u32,
        mut img_view: ArrayViewMut3<'_, f32>,
    ) {
        let resized_img = image::imageops::resize(
            img,
            resize_width,
            self.get_target_height(),
            self.active_filter(),
        );
        let target_height_usize = self.get_target_height() as usize;
        let resize_width_usize = resize_width as usize;
        let padded_width_usize = padded_width as usize;
        let plane_len = target_height_usize * padded_width_usize;

        if let Some(buffer) = img_view.as_slice_mut() {
            buffer.fill(-1.0);
            let (r_plane, rest) = buffer.split_at_mut(plane_len);
            let (g_plane, b_plane) = rest.split_at_mut(plane_len);
            for (y, row) in resized_img
                .as_raw()
                .chunks_exact(resize_width_usize * 4)
                .enumerate()
            {
                let row_offset = y * padded_width_usize;
                for (x, pixel) in row.chunks_exact(4).enumerate() {
                    let idx = row_offset + x;
                    r_plane[idx] = pixel[0] as f32 * REC_SCALE - 1.0;
                    g_plane[idx] = pixel[1] as f32 * REC_SCALE - 1.0;
                    b_plane[idx] = pixel[2] as f32 * REC_SCALE - 1.0;
                }
            }
        } else {
            for y in 0..target_height_usize {
                for x in 0..resize_width_usize {
                    let idx = (y * resize_width_usize + x) * 4;
                    let p = &resized_img.as_raw()[idx..idx + 4];
                    img_view[[0, y, x]] = p[0] as f32 * REC_SCALE - 1.0;
                    img_view[[1, y, x]] = p[1] as f32 * REC_SCALE - 1.0;
                    img_view[[2, y, x]] = p[2] as f32 * REC_SCALE - 1.0;
                }
                for x in resize_width_usize..padded_width_usize {
                    img_view[[0, y, x]] = -1.0;
                    img_view[[1, y, x]] = -1.0;
                    img_view[[2, y, x]] = -1.0;
                }
            }
        }
    }

    fn preprocess_rgba(&self, img: &RgbaImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])> {
        let (origin_w, origin_h) = img.dimensions();
        let target_height = self.get_target_height();
        let ratio = origin_h as f32 / target_height as f32;
        let target_width = self.calc_target_width_rgba(img);

        Log::debug(&format!(
            "Rec缩放: 原始={}x{}, 调整后={}x{}, 填充后={}x{}",
            origin_w, origin_h, target_width, target_height, target_width, target_height
        ));

        let mut input = Array3::<f32>::zeros((3, target_height as usize, target_width as usize));
        let img_view = input.view_mut();
        self.fill_image_tensor_rgba(img, target_width, target_width, img_view);
        let input = input.insert_axis(Axis(0));

        Ok((input.into_dyn(), [ratio, ratio], [origin_h, origin_w]))
    }

    /// 执行单张识别链路。
    ///
    /// 这条路径会保留原始检测框索引，按“预处理 -> 串行推理 -> 原索引回填后处理”执行。
    fn recognize_single_samples(
        &self,
        samples: &[RecSample],
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let preprocessed_inputs: Vec<(usize, ArrayD<f32>)> = samples
            .par_iter()
            .filter_map(|sample| {
                self.preprocess_rgba(&sample.image)
                    .ok()
                    .map(|input| (sample.original_index, input.0))
            })
            .collect();

        if preprocessed_inputs.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-预处理：部分图像预处理失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    preprocessed_inputs.len()
                )
                .as_str(),
            );
        }

        let mut results: Vec<(usize, OcrResult)> = if self.base_model.has_parallel_session_pool() {
            preprocessed_inputs
                .par_iter()
                .filter_map(|(idx, input)| {
                    let Some(det_res) = det_results.get(*idx) else {
                        Log::warn(format!("文字识别-后处理：索引 {} 越界", idx).as_str());
                        return None;
                    };
                    match self.base_model.inference_with_output_view(
                        input.view(),
                        self.get_input_node_name(),
                        self.get_output_node_name(),
                        |output| self.postprocess(output, det_res, 0),
                    ) {
                        Ok(ocr) => Some((*idx, ocr)),
                        Err(e) => {
                            Log::warn(
                                format!("文字识别-推理：第 {} 项推理失败: {:?}", idx, e).as_str(),
                            );
                            None
                        }
                    }
                })
                .collect()
        } else {
            let mut results = Vec::with_capacity(preprocessed_inputs.len());
            for (idx, input) in preprocessed_inputs {
                let Some(det_res) = det_results.get(idx) else {
                    Log::warn(format!("文字识别-后处理：索引 {} 越界", idx).as_str());
                    continue;
                };
                match self.base_model.inference_with_output_view(
                    input.view(),
                    self.get_input_node_name(),
                    self.get_output_node_name(),
                    |output| self.postprocess(output, det_res, 0),
                ) {
                    Ok(ocr) => results.push((idx, ocr)),
                    Err(e) => {
                        Log::warn(
                            format!("文字识别-推理：第 {} 项推理失败: {:?}", idx, e).as_str(),
                        );
                    }
                }
            }
            results
        };

        if results.len() != det_results.len() {
            Log::warn(
                format!(
                    "文字识别-后处理：部分结果处理失败！(总数: {}, 成功: {})",
                    det_results.len(),
                    results.len()
                )
                .as_str(),
            );
        }

        results.sort_by_key(|(idx, _)| *idx);
        Ok(results.into_iter().map(|(_, item)| item).collect())
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
                    self.fill_image_tensor_rgba(
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
        let parallel_inference = self.base_model.has_parallel_session_pool();

        for (_bucket, bucket_samples) in buckets.iter_mut() {
            bucket_samples.sort_by_key(|sample| (sample.target_width, sample.original_index));

            if parallel_inference {
                let prepared_chunks: VisionResult<Vec<(Vec<usize>, ArrayD<f32>)>> = bucket_samples
                    .chunks(batch_limit)
                    .map(|chunk| {
                        let padded_width = chunk
                            .iter()
                            .map(|sample| sample.target_width)
                            .max()
                            .unwrap_or(8);
                        let input = self.preprocess_sample_batch(chunk, padded_width)?;
                        Ok((
                            chunk.iter().map(|sample| sample.original_index).collect(),
                            input,
                        ))
                    })
                    .collect();

                let chunk_results = prepared_chunks?
                    .into_par_iter()
                    .map(|(sample_indexes, input)| {
                        self.base_model.inference_with_output_view(
                            input.view(),
                            self.get_input_node_name(),
                            self.get_output_node_name(),
                            |output| {
                                sample_indexes
                                    .iter()
                                    .enumerate()
                                    .map(|(batch_index, original_index)| {
                                        let det_res =
                                            det_results.get(*original_index).ok_or_else(|| {
                                                VisionError::BatchMatchDetSizeFailed {
                                                    batch: sample_indexes.len(),
                                                    det_num: det_results.len(),
                                                }
                                            })?;
                                        let ocr =
                                            self.postprocess(output.view(), det_res, batch_index)?;
                                        Ok((*original_index, ocr))
                                    })
                                    .collect::<VisionResult<Vec<_>>>()
                            },
                        )
                    })
                    .collect::<VisionResult<Vec<_>>>()?;

                results.extend(chunk_results.into_iter().flatten());
            } else {
                for chunk in bucket_samples.chunks(batch_limit) {
                    let padded_width = chunk
                        .iter()
                        .map(|sample| sample.target_width)
                        .max()
                        .unwrap_or(8);
                    let input = self.preprocess_sample_batch(chunk, padded_width)?;
                    let chunk_results = self.base_model.inference_with_output_view(
                        input.view(),
                        self.get_input_node_name(),
                        self.get_output_node_name(),
                        |output| {
                            chunk
                                .par_iter()
                                .enumerate()
                                .map(|(batch_index, sample)| {
                                    let det_res = det_results
                                        .get(sample.original_index)
                                        .ok_or_else(|| VisionError::BatchMatchDetSizeFailed {
                                            batch: chunk.len(),
                                            det_num: det_results.len(),
                                        })?;
                                    let ocr =
                                        self.postprocess(output.view(), det_res, batch_index)?;
                                    Ok((sample.original_index, ocr))
                                })
                                .collect::<VisionResult<Vec<_>>>()
                        },
                    )?;
                    results.extend(chunk_results);
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

        let content =
            read_to_string(dict_path.clone())
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
        self.base_model
            .load_model_base_with_session_intra_threads::<Self>(
                "paddle_rec_crnn",
                Some(self.parallel_cpu_session_intra_threads),
            )
    }
    fn get_input_size(&self) -> (u32, u32) {
        (self.base_model.input_width, self.base_model.input_height)
    }

    fn preprocess(&self, img: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])> {
        // 获取原始图像尺寸
        let (origin_w, origin_h) = GenericImageView::dimensions(img);
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
    fn recognize_crops(
        &self,
        cropped_images: Vec<DynamicImage>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        self.recognize_crops_rgba(
            cropped_images
                .into_iter()
                .map(|image| image.to_rgba8())
                .collect(),
            det_results,
        )
    }

    fn recognize_crops_rgba(
        &self,
        cropped_images: Vec<RgbaImage>,
        det_results: &[DetResult],
    ) -> VisionResult<Vec<OcrResult>> {
        let samples: Vec<_> = cropped_images
            .into_par_iter()
            .enumerate()
            .map(|(idx, image)| RecSample {
                original_index: idx,
                target_width: self.calc_target_width_rgba(&image),
                image,
            })
            .collect();
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
}
