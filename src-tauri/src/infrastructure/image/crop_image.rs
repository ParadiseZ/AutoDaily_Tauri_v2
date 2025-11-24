use crate::domain::vision::result::DetResult;
use crate::infrastructure::image::img_error::{ImageError, ImageResult};
use crate::infrastructure::vision::vision_error::VisionResult;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp, Projection};
use rayon::iter::IntoParallelRefIterator;

pub fn get_crop_images(
    img: &DynamicImage,
    results: &[DetResult],
) -> VisionResult<Vec<DynamicImage>> {
    if results.is_empty() {
        return Ok(Vec::new());
    }
    let rgba_img = img.to_rgba8();

    let cropped_images = results
        .par_iter()
        .filter_map(|&det_res| {
            get_crop_image(&rgba_img, det_res).ok()
        })
        .collect();
    Ok(cropped_images)
}

pub fn get_crop_image(img: &RgbaImage, det_res: &DetResult) -> ImageResult<DynamicImage> {
    // 回退到原始的串行处理逻辑
    let res = &det_res.bounding_box;
    let box_width = (res.x2 as f32 - res.x1 as f32).abs().max(1.0);
    let box_height = (res.y2 as f32 - res.y1 as f32).abs().max(1.0);

    let src_points = [
        (res.x1 as f32, res.y1 as f32),
        (res.x2 as f32, res.y1 as f32),
        (res.x2 as f32, res.y2 as f32),
        (res.x1 as f32, res.y2 as f32),
    ];

    let width = box_width as u32;
    let height = box_height as u32;

    let dst_points = [
        (0.0, 0.0),
        (width as f32, 0.0),
        (width as f32, height as f32),
        (0.0, height as f32),
    ];

    if let Some(proj) = Projection::from_control_points(src_points, dst_points) {
        let warped: ImageBuffer<Rgba<u8>, Vec<u8>> = warp(
            &img,
            &proj,
            imageproc::geometric_transformations::Interpolation::Bilinear,
            Rgba([0, 0, 0, 0]),
        );
        Ok(DynamicImage::ImageRgba8(warped).crop_imm(0, 0, width, height))
    } else {
        Err(ImageError::CropErr {
            detail: det_res.clone(),
            e: "".to_string(),
        })
    }
}
