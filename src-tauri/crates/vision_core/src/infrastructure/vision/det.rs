use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::vision::det::paddle_dbnet::PaddleDetDbNet;
use crate::infrastructure::vision::det::yolo::YoloDet;

pub mod paddle_dbnet;
pub mod yolo;

#[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
#[ts(export)]
pub enum DetectorType {
    Yolo11(YoloDet),
    PaddleDbNet(PaddleDetDbNet),
}
