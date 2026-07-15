#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PointU16 {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PointF32 {
    pub x: f32,
    pub y: f32,
}
