use crate::infra::image::img_error::{ImageError, ImageResult};
use crate::infra::vision::vision_error::VisionResult;
use domain_vision::DetResult;
use image::{DynamicImage, RgbaImage};
use infra_logging::Log;
use rayon::prelude::*;

pub(crate) fn get_crop_images(
    img: &DynamicImage,
    results: &[DetResult],
) -> VisionResult<Vec<DynamicImage>> {
    if results.is_empty() {
        return Ok(Vec::new());
    }
    //let rgba_img = img.to_rgba8();

    let cropped_images = results
        .par_iter()
        .filter_map(|det_res| get_crop_image(img, det_res).ok())
        .collect();
    Ok(cropped_images)
}
pub(crate) fn get_crop_image(img: &DynamicImage, det_res: &DetResult) -> ImageResult<DynamicImage> {
    let (x, y, width, height) = crop_rect(det_res)?;
    Ok(img.crop_imm(x, y, width, height))
}

pub fn get_crop_image_rgba(img: &RgbaImage, det_res: &DetResult) -> ImageResult<RgbaImage> {
    let (x, y, width, height) = crop_rect(det_res)?;
    Ok(image::imageops::crop_imm(img, x, y, width, height).to_image())
}

fn crop_rect(det_res: &DetResult) -> ImageResult<(u32, u32, u32, u32)> {
    let res = &det_res.bounding_box;
    let box_width = (res.x2 as f32 - res.x1 as f32).abs().max(1.0) as u32;
    if box_width < 8u32 {
        // crnn至少为8
        Log::error("图像裁剪错误：box_width < 8,图像区域太小！");
        return Err(ImageError::CropErr {
            detail: det_res.to_string(),
            e: "".to_string(),
        });
    }
    let box_height = (res.y2 as f32 - res.y1 as f32).abs().max(1.0) as u32;
    if box_height < 8u32 {
        // crnn至少为8
        Log::error("图像裁剪错误：box_height < 8,图像区域太小！");
        return Err(ImageError::CropErr {
            detail: det_res.to_string(),
            e: "".to_string(),
        });
    }
    Ok((res.x1 as u32, res.y1 as u32, box_width, box_height))
}
