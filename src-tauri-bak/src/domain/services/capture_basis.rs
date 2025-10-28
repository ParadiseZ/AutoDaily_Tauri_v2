use async_trait::async_trait;
use image::DynamicImage;
use crate::domain::entities::app_result::AppResult;

#[async_trait]
pub trait CaptureHandler{
    fn capture(&self) -> AppResult<DynamicImage>;
}