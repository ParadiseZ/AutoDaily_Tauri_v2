use serde::{Deserialize, Serialize};
use crate::domain::entities::vision::bounding_box::{BoundingBox};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub id : i32,
    pub pre_id : i32,
    pub next_id : i32,
    pub bounding_box: BoundingBox, // 四个点 (业务关键！)
    pub txt : String,
    pub score : Vec<f32>,
    pub index : Vec<usize>,
    pub txt_char : Vec<String>
}