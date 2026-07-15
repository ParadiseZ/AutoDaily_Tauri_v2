use bincode::{Decode, Encode};
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq, ts_rs::TS)]
#[ts(export)]
pub struct DetResult {
    pub bounding_box: BoundingBox, // 四个点 (业务关键！)
    pub stable_box: BoundingBox,
    pub stable_center: StablePoint,
    pub index: i32,
    pub label: String,
    pub score: f32,
}

impl fmt::Display for DetResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bounding_box: {:?}, stable_box: {:?}, stable_center: {:?}, index: {}, label: {}, score: {:.4}",
            self.bounding_box,
            self.stable_box,
            self.stable_center,
            self.index,
            self.label,
            self.score
        )
    }
}

impl DetResult {
    pub fn new(
        bounding_box: BoundingBox,
        index: i32,
        label: String,
        score: f32,
        grid_size: u16,
    ) -> Self {
        Self {
            stable_box: bounding_box.to_stable_box(grid_size),
            stable_center: bounding_box.to_stable_center(grid_size),
            bounding_box,
            index,
            label,
            score,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq, ts_rs::TS)]
#[ts(export)]
pub struct OcrResult {
    pub bounding_box: BoundingBox, // 四个点 (业务关键！)
    pub stable_box: BoundingBox,
    pub stable_center: StablePoint,
    pub txt: String,
    pub score: Vec<f32>,
    pub index: Vec<usize>,
}

impl fmt::Display for OcrResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bounding_box: {:?}, stable_box: {:?}, stable_center: {:?}, txt: {}, score: {:?}, index: {:?}",
            self.bounding_box,
            self.stable_box,
            self.stable_center,
            self.txt,
            self.score,
            self.index
        )
    }
}

impl OcrResult {
    pub fn new(
        bounding_box: BoundingBox,
        txt: String,
        score: Vec<f32>,
        index: Vec<usize>,
        grid_size: u16,
    ) -> Self {
        Self {
            stable_box: bounding_box.to_stable_box(grid_size),
            stable_center: bounding_box.to_stable_center(grid_size),
            bounding_box,
            txt,
            score,
            index,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
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

    pub fn center(&self) -> StablePoint {
        StablePoint {
            x: (self.x1 + self.x2) / 2,
            y: (self.y1 + self.y2) / 2,
        }
    }

    pub(crate) fn to_stable_box(&self, grid_size: u16) -> Self {
        let step = grid_size.max(1) as i32;
        Self {
            x1: quantize_coord(self.x1, step),
            y1: quantize_coord(self.y1, step),
            x2: quantize_coord(self.x2, step),
            y2: quantize_coord(self.y2, step),
        }
    }

    pub(crate) fn to_stable_center(&self, grid_size: u16) -> StablePoint {
        self.center().to_stable(grid_size)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
pub struct StablePoint {
    pub x: i32,
    pub y: i32,
}

impl StablePoint {
    pub(crate) fn to_stable(&self, grid_size: u16) -> Self {
        let step = grid_size.max(1) as i32;
        Self {
            x: quantize_coord(self.x, step),
            y: quantize_coord(self.y, step),
        }
    }
}

fn quantize_coord(value: i32, step: i32) -> i32 {
    if step <= 1 {
        return value;
    }

    let half = step / 2;
    if value >= 0 {
        ((value + half) / step) * step
    } else {
        ((value - half) / step) * step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stabilizes_coordinates_on_the_configured_grid() {
        let box_area = BoundingBox::new(9, 15, 25, 31);

        assert_eq!(box_area.to_stable_box(8), BoundingBox::new(8, 16, 24, 32));
        assert_eq!(box_area.to_stable_center(8), StablePoint { x: 16, y: 24 });
    }
}
