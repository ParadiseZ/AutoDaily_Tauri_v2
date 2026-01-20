use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::base_model::BaseModel;
use crate::infrastructure::vision::base_traits::{ModelHandler, TextRecognizer};
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use image::DynamicImage;
use imageproc::drawing::Canvas;
use std::path::PathBuf;

use ndarray::{Array3, Array4, ArrayD, ArrayViewD, ArrayViewMut3, Axis};
use rayon::prelude::*;
use tokio::fs::read_to_string;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaddleRecCrnn {
    pub base_model: BaseModel,
    pub dict_path: Option<PathBuf>,
    #[serde(skip)]
    pub dict: Vec<String>,
}

impl PaddleRecCrnn{
    pub async fn load_dict(&mut self) -> VisionResult<()> {
        if self.dict_path.is_none(){
            return Err(VisionError::IoError {
                path: "".to_string(),
                e: "字典路径为空".to_string(),
            })
        }

        let content = read_to_string(self.dict_path.unwrap()).await.map_err(|e| VisionError::IoError {
            path: self.dict_path.unwrap().to_string_lossy().to_string(),
            e: e.to_string(),
        })?;

        let dict: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if dict.is_empty() {
            return Err(VisionError::IoError {
                path: self.dict_path.unwrap().to_string_lossy().to_string(),
                e: "字典文件为空".to_string(),
            });
        }
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
        let resize_w = (origin_w as f32 / ratio).ceil() as u32;

        // CRNN模型需要将宽度调整为8的倍数，以获得最佳性能并避免在下采样层中出现问题
/*        let target_width = if resize_w % 8 != 0 {
            (resize_w / 8 + 1) * 8
        } else {
            resize_w
        }
        .max(8); // 确保宽度至少为8*/

        // 3. 计算目标宽度（带8对齐）
        let target_width = resize_w
            .checked_add(7)
            .map(|v| v & !7) // 更快的8对齐：v - (v % 8) 的优化版
            .unwrap_or(resize_w)
            .max(8);

        Log::debug(&format!(
            "Rec缩放: 原始={}x{}, 调整后={}x{}, 填充后={}x{}",
            origin_w, origin_h, resize_w, target_height, target_width, target_height
        ));

        // 调整图像大小
        let resized_img = img.resize_exact(
            resize_w,
            target_height,
            image::imageops::FilterType::CatmullRom,
        );

        // 初始化输入数组 (使用带填充的宽度)
        let mut input = Array3::<f32>::zeros((3, target_height as usize, target_width as usize));

        // 转换图像格式并归一化
        // 优化：先转换为RGB8，避免在循环中进行DynamicImage的动态分发，提高性能
        let rgb_img = resized_img.to_rgb8();

        // 使用Rayon并行处理每一行
        // 将每一行(Axis 1)的可变视图收集起来，然后并行迭代
        let mut rows: Vec<_> = input.axis_iter_mut(Axis(1)).collect();
        
        rows.par_iter_mut().enumerate().for_each(|(y, row)| {
            let y = y as u32;
            // 填充真实图像数据
            for x in 0..resize_w {
                // 直接访问RGB数据，比DynamicImage.get_pixel快
                let pixel = rgb_img.get_pixel(x, y);
                // 标准化: (pixel / 255.0 - 0.5) / 0.5
                let x = x as usize;
                row[[0, x]] = (pixel[0] as f32 / 255.0 - 0.5) / 0.5;
                row[[1, x]] = (pixel[1] as f32 / 255.0 - 0.5) / 0.5;
                row[[2, x]] = (pixel[2] as f32 / 255.0 - 0.5) / 0.5;
            }
            // 对多余的宽度部分进行填充
            for x in resize_w..target_width {
                let x = x as usize;
                // 使用归一化后的0值 (-1.0) 填充
                row[[0, x]] = -1.0;
                row[[1, x]] = -1.0;
                row[[2, x]] = -1.0;
            }
        });

        // 扩展到批次维度 (1, C, H, W)
        let input = input.insert_axis(Axis(0));

        Ok((input.into_dyn(), [ratio, ratio], [origin_h, origin_w]))
    }

    fn inference(&self, input: ArrayViewD<f32>) -> VisionResult<ArrayD<f32>> {
        // 使用通用推理方法，消除代码重复
        self.base_model.inference_base(input, self.get_input_node_name(), self.get_output_node_name())
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
    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        det_result: &DetResult,
        batch_size: usize,
    ) -> VisionResult<OcrResult> {
        let seq_len = output.shape()[1];
        let class_num = output.shape()[2];

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
        let mut prev_idx: Option<usize> = None;
        for t in 0..seq_len {
            let mut max_idx = 0;
            let mut max_prob = output[[batch_size, 0, t, 0]];

            for c in 1..class_num {
                let prob = output[[batch_size, 0, t, c]];
                if prob > max_prob {
                    max_prob = prob;
                    max_idx = c;
                }
            }

            if max_idx != class_num - 1 && max_idx > 0 {
                // 不是空白
                if let Some(prev) = prev_idx {
                    if max_idx != prev && max_idx < self.dict.len() {
                        let char = &self.dict[max_idx - 1];
                        scores.push(max_prob);
                        indexes.push(max_idx - 1);
                        result_text.push_str(char);
                        chars.push(char.into())
                    }
                } else {
                    if max_idx < self.dict.len() {
                        let char = &self.dict[max_idx - 1];
                        scores.push(max_prob);
                        indexes.push(max_idx - 1);
                        result_text.push_str(char);
                        chars.push(char.into())
                    }
                }
                prev_idx = Some(max_idx - 1);
            } else {
                prev_idx = None;
            }
        }
        let ocr_result = OcrResult {
            id: 0,
            pre_id: 0,
            next_id: 0,
            bounding_box: det_result.bounding_box.clone(),
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

        let mut max_width = 0;
        // 计算所有图像的目标宽度
        let widths: Vec<u32> = images
            .par_iter()
            .map(|img: &DynamicImage| {
                let (origin_w, origin_h) = img.dimensions();
                let ratio = origin_h as f32 / (self.get_target_height() as f32);
                let resize_w = ((origin_w as f32) / ratio).ceil() as u32;

                // 3. 计算目标宽度（带8对齐）
                resize_w
                    .checked_add(7)
                    .map(|v| v & !7) // 更快的8对齐：v - (v % 8) 的优化版
                    .unwrap_or(resize_w)
                    .max(8)
            })
            .collect();
        for width in widths.iter() {
            if *width > max_width {
                max_width = *width;
            }
        }

        // 使用异步并行处理
        let batch_size = images.len();
        let target_height_usize = self.get_target_height() as usize;
        let max_width_usize = max_width as usize;

        let mut batch_data = Array4::<f32>::zeros((
            batch_size,
            3,
            target_height_usize,
            max_width_usize,
        ));

        let chunk_len = 3 * target_height_usize * max_width_usize;

        if let Some(flat_data) = batch_data.as_slice_mut() {
            flat_data
                .par_chunks_mut(chunk_len)
                .zip(images.par_iter())
                .zip(widths.par_iter())
                .for_each(|((chunk, img), &width)| {
                    let mut img_view = ArrayViewMut3::from_shape(
                        (3, target_height_usize, max_width_usize),
                        chunk,
                    )
                    .expect("文字识别失败：批预处理失败【crnn preprocess_batch】Failed to create view from chunk");

                    let resized_img = img.resize_exact(
                        width,
                        self.get_target_height(),
                        image::imageops::FilterType::Nearest,
                    );
                    let rgb_view = resized_img.to_rgb8();

                    for y in 0..target_height_usize {
                        for x in 0..width as usize {
                            let p = rgb_view.get_pixel(x as u32, y as u32);
                            img_view[[0, y, x]] = (p[0] as f32 / 255.0 - 0.5) / 0.5;
                            img_view[[1, y, x]] = (p[1] as f32 / 255.0 - 0.5) / 0.5;
                            img_view[[2, y, x]] = (p[2] as f32 / 255.0 - 0.5) / 0.5;
                        }
                        for x in width as usize..max_width_usize {
                            img_view[[0, y, x]] = -1.0;
                            img_view[[1, y, x]] = -1.0;
                            img_view[[2, y, x]] = -1.0;
                        }
                    }
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
}
