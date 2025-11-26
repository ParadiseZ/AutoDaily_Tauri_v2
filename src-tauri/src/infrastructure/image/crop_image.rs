use crate::domain::vision::result::DetResult;
use crate::infrastructure::image::img_error::{ImageError, ImageResult};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::vision_error::VisionResult;
use image::DynamicImage;
use rayon::prelude::*;

pub fn get_crop_images(
    img: &DynamicImage,
    results: &[DetResult],
) -> VisionResult<Vec<DynamicImage>> {
    if results.is_empty() {
        return Ok(Vec::new());
    }
    //let rgba_img = img.to_rgba8();

    let cropped_images = results
        .par_iter()
        .filter_map(|det_res| {
            get_crop_image(img, det_res).ok()
        })
        .collect();
    Ok(cropped_images)
}
pub fn get_crop_image(img: &DynamicImage, det_res: &DetResult) -> ImageResult<DynamicImage> {
    let res = &det_res.bounding_box;
    let box_width = (res.x2 as f32 - res.x1 as f32).abs().max(1.0) as u32;
    if box_width < 8u32 { // crnn至少为8
        Log::error("图像裁剪错误：box_width < 8,图像区域太小！");
        return Err(ImageError::CropErr {
            detail: det_res.to_string(),
            e: "".to_string(),
        })
    }
    let box_height = (res.y2 as f32 - res.y1 as f32).abs().max(1.0) as u32;
    if box_height < 8u32 { // crnn至少为8
        Log::error("图像裁剪错误：box_height < 8,图像区域太小！");
        return Err(ImageError::CropErr {
            detail: det_res.to_string(),
            e: "".to_string(),
        })
    }
    Ok(img.crop_imm(res.x1 as u32, res.y1 as u32, box_width, box_height))
}
/*pub fn get_crop_image(img: &DynamicImage, det_res: &DetResult) -> ImageResult<DynamicImage> {
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
        let warped: RgbaImage = warp(
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
}*/
