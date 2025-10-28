use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize,Deserialize)]
pub struct BoundingBox{
    pub x1 : i32,
    pub y1 : i32,
    pub x2 : i32,
    pub y2 : i32
}

impl BoundingBox{
    pub fn new(x1 : i32, y1: i32, x2: i32, y2: i32)-> Self{
        Self{
            x1,y1,x2,y2
        }
    }
}

#[derive(Clone)]
pub struct Rect{
    pub x1 : f32,
    pub y1 : f32,
    pub x2 : f32,
    pub y2 : f32
}

impl Rect{
    pub fn new(x1 : f32, y1: f32, x2: f32, y2: f32)-> Self{
        Self{
            x1,y1,x2,y2
        }
    }
}
