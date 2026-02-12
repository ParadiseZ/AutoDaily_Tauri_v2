use crate::domain::scripts::point::{Point, PointU16};
use crate::infrastructure::core::{Deserialize, Serialize};

/// 点击操作枚举
///
#[derive(Debug,Clone,Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum Click{
    Percent {
        x:f32,
        y:f32
    },
    Point(#[ts(as = "PointU16")] Point<u16>),
    Label(Vec<u16>),
    Txt(Vec<String>),
    Var(String)
}