use crate::infra::vision::det::paddle_dbnet::PaddleDetDbNet;
use crate::infra::vision::det::yolo::YoloDet;
use domain_vision::DetectorType;

mod paddle_dbnet;
mod yolo;

#[derive(Debug)]
pub(crate) enum RuntimeDetector {
    Yolo11(YoloDet),
    Yolo26(YoloDet),
    PaddleDbNet(PaddleDetDbNet),
}

impl From<DetectorType> for RuntimeDetector {
    fn from(config: DetectorType) -> Self {
        match config {
            DetectorType::Yolo11(config) => Self::Yolo11(config.into()),
            DetectorType::Yolo26(config) => Self::Yolo26(config.into()),
            DetectorType::PaddleDbNet(config) => Self::PaddleDbNet(config.into()),
        }
    }
}
