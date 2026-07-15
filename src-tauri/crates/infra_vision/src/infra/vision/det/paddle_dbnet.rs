use crate::infra::vision::base_model::BaseModel;
use crate::infra::vision::base_traits::{ModelHandler, TextDetector};
use crate::infra::vision::vision_error::{VisionError, VisionResult};
use domain_vision::{BoundingBox, DetResult, PaddleDetDbNet as PaddleDetDbNetConfig};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageBuffer, RgbaImage};
use imageproc::contours::find_contours;
use imageproc::point::Point;

use ndarray::{Array4, ArrayD, ArrayView2, ArrayView4, ArrayViewD, s};
use rayon::prelude::*;
use std::sync::Mutex;

/// dbNet通常值
const MIN_AREA: f32 = 3.0;
/// 经验值
const MIN_AREA_AFTER: f32 = 5.0; //MIN_AREA+2.0

#[derive(Debug)]
pub(crate) struct PaddleDetDbNet {
    pub base_model: BaseModel,
    pub db_thresh: f32,
    pub db_box_thresh: f32,
    pub unclip_ratio: f32,
    pub use_dilation: bool,
    preprocess_buffer: Mutex<Vec<f32>>,
}

impl From<PaddleDetDbNetConfig> for PaddleDetDbNet {
    fn from(config: PaddleDetDbNetConfig) -> Self {
        Self {
            base_model: config.base_model.into(),
            db_thresh: config.db_thresh,
            db_box_thresh: config.db_box_thresh,
            unclip_ratio: config.unclip_ratio,
            use_dilation: config.use_dilation,
            preprocess_buffer: Self::default_preprocess_buffer(),
        }
    }
}

const INV_255: f32 = 1.0 / 255.0;
const DBNET_R_SCALE: f32 = INV_255 / 0.229;
const DBNET_G_SCALE: f32 = INV_255 / 0.224;
const DBNET_B_SCALE: f32 = INV_255 / 0.225;
const DBNET_R_PAD: f32 = -0.485 / 0.229;
const DBNET_G_PAD: f32 = -0.456 / 0.224;
const DBNET_B_PAD: f32 = -0.406 / 0.225;

impl PaddleDetDbNet {
    fn default_preprocess_buffer() -> Mutex<Vec<f32>> {
        Mutex::new(Vec::new())
    }

    fn input_geometry(&self, image: &DynamicImage) -> (u32, u32, f32, [u32; 2]) {
        let (origin_w, origin_h) = image.dimensions();
        self.input_geometry_from_dims(origin_w, origin_h)
    }

    fn input_geometry_rgba(&self, image: &RgbaImage) -> (u32, u32, f32, [u32; 2]) {
        let (origin_w, origin_h) = image.dimensions();
        self.input_geometry_from_dims(origin_w, origin_h)
    }

    fn input_geometry_from_dims(&self, origin_w: u32, origin_h: u32) -> (u32, u32, f32, [u32; 2]) {
        let scale = self.get_target_height() as f32 / origin_h as f32;
        let width = (origin_w as f32 * scale).round() as u32;
        let target_width = width.next_multiple_of(32);

        (width, target_width, scale, [origin_h, origin_w])
    }

    fn fill_chw_buffer(
        &self,
        image: &DynamicImage,
        resized_width: u32,
        padded_width: u32,
        input_buffer: &mut [f32],
    ) {
        let target_height = self.get_target_height();
        let target_height_usize = target_height as usize;
        let resized_width_usize = resized_width as usize;
        let padded_width_usize = padded_width as usize;
        let plane_len = target_height_usize * padded_width_usize;
        let (r_plane, rest) = input_buffer.split_at_mut(plane_len);
        let (g_plane, b_plane) = rest.split_at_mut(plane_len);

        r_plane.fill(DBNET_R_PAD);
        g_plane.fill(DBNET_G_PAD);
        b_plane.fill(DBNET_B_PAD);

        let resized_img = image
            .resize_exact(resized_width, target_height, FilterType::Triangle)
            .to_rgb8();
        for (y, row) in resized_img
            .as_raw()
            .chunks_exact(resized_width_usize * 3)
            .enumerate()
        {
            let row_offset = y * padded_width_usize;
            for (x, pixel) in row.chunks_exact(3).enumerate() {
                let idx = row_offset + x;
                r_plane[idx] = pixel[0] as f32 * DBNET_R_SCALE + DBNET_R_PAD;
                g_plane[idx] = pixel[1] as f32 * DBNET_G_SCALE + DBNET_G_PAD;
                b_plane[idx] = pixel[2] as f32 * DBNET_B_SCALE + DBNET_B_PAD;
            }
        }
    }

    fn fill_chw_buffer_rgba(
        &self,
        image: &RgbaImage,
        resized_width: u32,
        padded_width: u32,
        input_buffer: &mut [f32],
    ) {
        let target_height = self.get_target_height();
        let target_height_usize = target_height as usize;
        let resized_width_usize = resized_width as usize;
        let padded_width_usize = padded_width as usize;
        let plane_len = target_height_usize * padded_width_usize;
        let (r_plane, rest) = input_buffer.split_at_mut(plane_len);
        let (g_plane, b_plane) = rest.split_at_mut(plane_len);

        r_plane.fill(DBNET_R_PAD);
        g_plane.fill(DBNET_G_PAD);
        b_plane.fill(DBNET_B_PAD);

        let resized_img =
            image::imageops::resize(image, resized_width, target_height, FilterType::Triangle);
        for (y, row) in resized_img
            .as_raw()
            .chunks_exact(resized_width_usize * 4)
            .enumerate()
        {
            let row_offset = y * padded_width_usize;
            for (x, pixel) in row.chunks_exact(4).enumerate() {
                let idx = row_offset + x;
                r_plane[idx] = pixel[0] as f32 * DBNET_R_SCALE + DBNET_R_PAD;
                g_plane[idx] = pixel[1] as f32 * DBNET_G_SCALE + DBNET_G_PAD;
                b_plane[idx] = pixel[2] as f32 * DBNET_B_SCALE + DBNET_B_PAD;
            }
        }
    }
}

impl ModelHandler for PaddleDetDbNet {
    fn load_model(&mut self) -> VisionResult<()> {
        self.base_model.load_model_base::<Self>("paddle_det_dbnet")
    }
    fn get_input_size(&self) -> (u32, u32) {
        (self.base_model.input_width, self.base_model.input_height)
    }

    fn preprocess(&self, image: &DynamicImage) -> VisionResult<(ArrayD<f32>, [f32; 2], [u32; 2])> {
        let (width, target_width, scale, origin_shape) = self.input_geometry(image);
        let target_height_usize = self.get_target_height() as usize;
        let target_width_usize = target_width as usize;
        let mut input_buffer = vec![0.0; 3 * target_height_usize * target_width_usize];
        self.fill_chw_buffer(image, width, target_width, input_buffer.as_mut_slice());
        let input = Array4::from_shape_vec(
            (1, 3, target_height_usize, target_width_usize),
            input_buffer,
        )
        .map_err(|e| VisionError::DataProcessingErr {
            method: "dbnet_preprocess".to_string(),
            e: e.to_string(),
        })?;

        Ok((input.into_dyn(), [scale, scale], origin_shape))
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

    fn get_target_height(&self) -> u32 {
        self.base_model.input_height
    }
}

impl TextDetector for PaddleDetDbNet {
    fn detect(&self, image: &DynamicImage) -> VisionResult<Vec<DetResult>> {
        let (width, target_width, scale, origin_shape) = self.input_geometry(image);
        let target_height_usize = self.get_target_height() as usize;
        let target_width_usize = target_width as usize;
        let input_len = 3 * target_height_usize * target_width_usize;
        let mut input_buffer =
            self.preprocess_buffer
                .lock()
                .map_err(|_| VisionError::DataProcessingErr {
                    method: "dbnet_detect".to_string(),
                    e: "获取DBNet预处理缓存失败".to_string(),
                })?;
        if input_buffer.len() < input_len {
            input_buffer.resize(input_len, 0.0);
        }
        let input_buffer = &mut input_buffer[..input_len];
        self.fill_chw_buffer(image, width, target_width, input_buffer);
        let input_view = ArrayView4::from_shape(
            (1, 3, target_height_usize, target_width_usize),
            input_buffer,
        )
        .map_err(|e| VisionError::DataProcessingErr {
            method: "dbnet_detect".to_string(),
            e: e.to_string(),
        })?;

        self.base_model.inference_with_output_view(
            input_view.into_dyn(),
            self.get_input_node_name(),
            self.get_output_node_name(),
            |output| self.postprocess(output, [scale, scale], origin_shape),
        )
    }

    fn detect_rgba(&self, image: &RgbaImage) -> VisionResult<Vec<DetResult>> {
        let (width, target_width, scale, origin_shape) = self.input_geometry_rgba(image);
        let target_height_usize = self.get_target_height() as usize;
        let target_width_usize = target_width as usize;
        let input_len = 3 * target_height_usize * target_width_usize;
        let mut input_buffer =
            self.preprocess_buffer
                .lock()
                .map_err(|_| VisionError::DataProcessingErr {
                    method: "dbnet_detect_rgba".to_string(),
                    e: "获取DBNet预处理缓存失败".to_string(),
                })?;
        if input_buffer.len() < input_len {
            input_buffer.resize(input_len, 0.0);
        }
        let input_buffer = &mut input_buffer[..input_len];
        self.fill_chw_buffer_rgba(image, width, target_width, input_buffer);
        let input_view = ArrayView4::from_shape(
            (1, 3, target_height_usize, target_width_usize),
            input_buffer,
        )
        .map_err(|e| VisionError::DataProcessingErr {
            method: "dbnet_detect_rgba".to_string(),
            e: e.to_string(),
        })?;

        self.base_model.inference_with_output_view(
            input_view.into_dyn(),
            self.get_input_node_name(),
            self.get_output_node_name(),
            |output| self.postprocess(output, [scale, scale], origin_shape),
        )
    }

    fn postprocess(
        &self,
        output: ArrayViewD<f32>,
        scale_factor: [f32; 2],
        origin_shape: [u32; 2],
    ) -> VisionResult<Vec<DetResult>> {
        // 实现DBNet后处理逻辑
        // 1. 二值化处理
        // 2. 轮廓提取
        // 3. 多边形近似
        // 4. 文本框扩展(unclip)
        let (_, _, h, w) = (
            output.shape()[0],
            output.shape()[1],
            output.shape()[2],
            output.shape()[3],
        );

        // 提取概率图
        let prob_map = output.slice(s![0, 0, .., ..]);

        // 二值化处理
        let mut binary_map: ImageBuffer<image::Luma<u8>, Vec<u8>> = ImageBuffer::from_vec(
            w as u32,
            h as u32,
            (0..h * w)
                .into_par_iter()
                .map(|idx| {
                    let y = idx / w;
                    let x = idx % w;
                    if prob_map[[y, x]] > self.db_thresh {
                        255
                    } else {
                        0
                    }
                })
                .collect(),
        )
        .ok_or_else(|| VisionError::DataProcessingErr {
            method: "dbnet_postprocess".to_string(),
            e: "二值图尺寸不匹配".to_string(),
        })?;

        // 可选的膨胀操作
        if self.use_dilation {
            let source = binary_map.as_raw();
            let dilated_data: Vec<u8> = (0..h * w)
                .into_par_iter()
                .map(|idx| {
                    let y = idx / w;
                    let x = idx % w;
                    let mut max_val = 0u8;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let nx = (x as i32 + dx).clamp(0, w as i32 - 1) as u32;
                            let ny = (y as i32 + dy).clamp(0, h as i32 - 1) as u32;
                            max_val = max_val.max(source[ny as usize * w + nx as usize]);
                        }
                    }
                    max_val
                })
                .collect();
            binary_map =
                ImageBuffer::from_vec(w as u32, h as u32, dilated_data).ok_or_else(|| {
                    VisionError::DataProcessingErr {
                        method: "dbnet_postprocess".to_string(),
                        e: "膨胀图尺寸不匹配".to_string(),
                    }
                })?;
        }

        // 查找轮廓
        let contours = find_contours::<i32>(&binary_map);
        // 处理每个轮廓
        let boxes: Vec<DetResult> = contours
            .par_iter()
            .filter_map(|contour| {
                // --- 步骤 1: 计算初始边界框 ---
                // [重要] 这里应该使用 `get_min_area_rect` (最小面积旋转矩形)
                // 但 imageproc 库标准版没有提供。我们暂时使用轴对齐框作为替代，
                // 但请注意这是导致检测倾斜文本失败的主要原因。
                let (min_box, min_side_len) = get_bounding_rect(&contour.points);
                if min_box.is_empty() {
                    return None;
                }

                // 根据最小边长过滤
                if min_side_len < MIN_AREA {
                    return None;
                }

                // --- 步骤 2: 计算得分并过滤 ---
                let score = box_score_fast(prob_map.view(), &contour.points, &min_box);

                if score < self.db_box_thresh {
                    return None;
                }

                // --- 步骤 3: 扩展多边形 (unclip) ---
                let unclipped_poly = unclip_polygon(&min_box, self.unclip_ratio);
                if unclipped_poly.is_empty() {
                    return None;
                }

                // --- 步骤 4: 对扩展后的多边形计算最终边界框 ---
                // 同样，这里也应该使用 `get_min_area_rect`

                // 检查扩展后的多边形是否满足最小面积要求
                let final_min_side = if unclipped_poly.len() == 4 {
                    let width = (unclipped_poly[2].x - unclipped_poly[0].x) as f32;
                    let height = (unclipped_poly[2].y - unclipped_poly[0].y) as f32;
                    width.min(height)
                } else {
                    0.0 // 不应该出现这种情况，因为输入是矩形
                };

                // 再次根据最小边长过滤
                if final_min_side < MIN_AREA_AFTER {
                    return None;
                }

                // --- 步骤 5: 调整回原始图像坐标 ---
                //let scaled_points = scale_polygon(final_box, scale_factor, origin_shape);
                let scaled_points = scale_polygon(unclipped_poly, scale_factor, origin_shape);

                //boxes.push((scaled_points, score));
                Some(DetResult::new(
                    BoundingBox::new(
                        scaled_points[0].x,
                        scaled_points[0].y,
                        scaled_points[2].x,
                        scaled_points[2].y,
                    ),
                    0,
                    "txt".into(),
                    score,
                    8,
                ))
            })
            .collect();

        // 按分数排序（可选）
        //boxes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(boxes)
    }
}

// 来计算最小面积的旋转矩形，以正确处理倾斜的文本。
fn get_bounding_rect(points: &[Point<i32>]) -> (Vec<Point<i32>>, f32) {
    if points.is_empty() {
        return (Vec::new(), 0.0);
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    // 创建矩形
    let rect = vec![
        Point::new(min_x, min_y), // 左上
        Point::new(max_x, min_y), // 右上
        Point::new(max_x, max_y), // 右下
        Point::new(min_x, max_y), // 左下
    ];

    let width = (max_x - min_x) as f32;
    let height = (max_y - min_y) as f32;
    let min_side = width.min(height);

    (rect, min_side)
}

// 快速计算轮廓内概率图的平均分
fn box_score_fast(
    prob_map: ArrayView2<'_, f32>,
    points: &[Point<i32>],
    rect: &[Point<i32>],
) -> f32 {
    let h = prob_map.shape()[0] as i32;
    let w = prob_map.shape()[1] as i32;

    if rect.is_empty() {
        return 0.0;
    }

    let min_x = rect[0].x.max(0).min(w - 1);
    let min_y = rect[0].y.max(0).min(h - 1);
    let max_x = rect[2].x.max(0).min(w - 1);
    let max_y = rect[2].y.max(0).min(h - 1);

    let mut score_sum = 0.0;
    let mut count = 0;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_polygon(Point::new(x, y), points) {
                // 使用 unsafe 访问以提高性能，因为我们已经确保了边界
                unsafe {
                    score_sum += *prob_map.uget([y as usize, x as usize]);
                }
                count += 1;
            }
        }
    }

    if count == 0 {
        return 0.0;
    }

    score_sum / count as f32
}

// 判断点是否在多边形内 (Ray-casting algorithm)
fn point_in_polygon(point: Point<i32>, polygon: &[Point<i32>]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        let intersect = ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x);

        if intersect {
            inside = !inside;
        }

        j = i;
    }

    inside
}

// 扩展多边形 (unclip)
// [注意] 这是一个简化的实现，它正确地计算了偏移距离，但仅适用于轴对齐的矩形。
// 一个鲁棒的实现需要一个像 Clipper 这样的多边形偏移库。
fn unclip_polygon(points: &[Point<i32>], unclip_ratio: f32) -> Vec<Point<i32>> {
    if points.len() != 4 {
        return points.to_vec(); // 只处理矩形
    }

    // 计算矩形的宽度和高度
    let width = (points[1].x - points[0].x).abs() as f32;
    let height = (points[3].y - points[0].y).abs() as f32;

    let perimeter = 2.0 * (width + height);
    if perimeter == 0.0 {
        return points.to_vec();
    }

    // 根据 DBNet 论文计算膨胀距离
    let distance = (width * height * unclip_ratio) / perimeter;

    // 扩展矩形
    let expand = distance.round() as i32;
    vec![
        Point::new(points[0].x - expand, points[0].y - expand), // 左上
        Point::new(points[1].x + expand, points[1].y - expand), // 右上
        Point::new(points[2].x + expand, points[2].y + expand), // 右下
        Point::new(points[3].x - expand, points[3].y + expand), // 左下
    ]
}

// 将多边形坐标调整回原始图像尺寸
fn scale_polygon(
    points: Vec<Point<i32>>,
    scale_factor: [f32; 2],
    origin_shape: [u32; 2],
) -> Vec<Point<i32>> {
    // let scale = self.get_target_height() as f32 / origin_h as f32;
    let [h_scale, w_scale] = scale_factor;
    points
        .into_iter()
        .map(|p| {
            let sx = ((p.x as f32 / w_scale).round() as i32).clamp(0, origin_shape[1] as i32 - 1);
            let sy = ((p.y as f32 / h_scale).round() as i32).clamp(0, origin_shape[0] as i32 - 1);
            Point::new(sx, sy)
        })
        .collect()
}
