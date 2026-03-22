use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::vision::rec::paddle_crnn::PaddleRecCrnn;

pub mod paddle_crnn;
#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum RecognizerType {
    PaddleCrnn(PaddleRecCrnn),
}
