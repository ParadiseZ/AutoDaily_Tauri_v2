use crate::domain::scripts::point::Point;
use crate::infrastructure::core::{Deserialize, Serialize};

/// 点击操作枚举
///
#[derive(Debug,Clone,Serialize, Deserialize)]
pub enum Click{
    Percent {
        x:f32,
        y:f32
    },
    Point(Point<u16>),
    Label(Vec<u16>),
    Txt(Vec<String>),
    Var(String)
}