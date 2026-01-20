use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::vision::det::paddle_dbnet::PaddleDetDbNet;
use crate::infrastructure::vision::det::yolo::YoloDet;

// 检测器模块 - 包含各种文本检测算法的实现
pub mod paddle_dbnet;
pub mod yolo;

#[derive(Serialize,Deserialize,Debug)]
pub enum DetectorType{
    Yolo11(YoloDet),
    PaddleDbNet(PaddleDetDbNet)
}