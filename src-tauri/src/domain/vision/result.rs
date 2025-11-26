use core::fmt;
use crate::infrastructure::core::{Deserialize, Serialize};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct DetResult {
    pub id: i32,
    pub pre_id: i32,
    pub next_id: i32,
    pub bounding_box: BoundingBox, // 四个点 (业务关键！)
    pub index: i32,
    pub label: String,
    pub score: f32,
}

impl fmt::Display for DetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}, pre_id: {}, next_id: {}, bounding_box: {:?}, index: {}, label: {}, score: {}",
            self.id,
            self.pre_id,
            self.next_id,
            self.bounding_box,
            self.index,
            self.label,
            self.score
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct OcrResult {
    pub id: i32,
    pub pre_id: i32,
    pub next_id: i32,
    pub bounding_box: BoundingBox, // 四个点 (业务关键！)
    pub txt: String,
    pub score: Vec<f32>,
    pub index: Vec<usize>,
    pub txt_char: Vec<String>,
}

impl fmt::Display for OcrResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}, pre_id: {}, next_id: {}, bounding_box: {:?}, txt: {}, score: {:?}, index: {:?}, txt_char: {:?}",
            self.id,
            self.pre_id,
            self.next_id,
            self.bounding_box,
            self.txt,
            self.score,
            self.index,
            self.txt_char
    )
}
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct BoundingBox {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl BoundingBox {
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        BoundingBox { x1, y1, x2, y2 }
    }
}

#[derive(Clone)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}
