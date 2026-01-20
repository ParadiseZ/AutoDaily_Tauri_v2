use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::vision::rec::paddle_crnn::PaddleRecCrnn;

// 识别器模块 - 包含各种文本识别算法的实现
pub mod paddle_crnn;


#[derive(Debug, Serialize, Deserialize)]
pub enum RecognizerType {
    PaddleCrnn(PaddleRecCrnn),
}