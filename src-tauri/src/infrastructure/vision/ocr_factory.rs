use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::vision::base_traits::{TextDetector, TextRecognizer};
use crate::infrastructure::vision::det::paddle_dbnet::PaddleDetDbNet;
use crate::infrastructure::vision::det::yolo::YoloDet;
use crate::infrastructure::vision::rec::paddle_crnn::PaddleRecCrnn;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::read_to_string;

/// 检测器类型枚举
#[derive(Debug, Serialize, Deserialize)]
pub enum DetectorType {
    Yolo11(YoloDet),
    PaddleDbNet(PaddleDetDbNet),
}

/// 识别器类型枚举
#[derive(Debug, Serialize, Deserialize)]
pub enum RecognizerType {
    PaddleCrnn(PaddleRecCrnn),
}

/// OCR模型工厂
pub struct OcrModelFactory;

impl OcrModelFactory {
    /// 内部方法：创建检测器实现（不通过管理器）
    pub(crate) async fn create_detector(
        det : DetectorType,
    ) -> VisionResult<Arc<dyn TextDetector>> {
        match det {
            DetectorType::Yolo11(net) => {
                Ok(Arc::new(net))
            }
            DetectorType::PaddleDbNet(det) => {
                Ok(Arc::new(det))
            }
        }
    }

    /// 内部方法：创建识别器实现（不通过管理器）
    pub(crate) async fn create_recognizer(rec : RecognizerType) -> VisionResult<Arc<dyn TextRecognizer>> {
        match rec {
            RecognizerType::PaddleCrnn(mut crnn) => {
                // 加载字典
                let dict = if let Some(dict_path) = crnn.dict_path.clone() {
                    Log::debug(&format!(
                        "加载字典{}",
                        dict_path.to_string_lossy().to_string()
                    ));
                    Self::load_dict(&dict_path).await?
                } else {
                    // 默认字符集
                    return Err(VisionError::IoError {
                        path: "".to_string(),
                        e: "字典路径不存在！".to_string(),
                    });
                };
                crnn.dict = dict;
                Ok(Arc::new(crnn))
            }
        }
    }

    /// 加载字典文件
    async fn load_dict(dict_path: &PathBuf) -> VisionResult<Vec<String>> {
        let content = read_to_string(dict_path).await.map_err(|e| VisionError::IoError {
            path: dict_path.to_string_lossy().to_string(),
            e: e.to_string(),
        })?;

        let dict: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if dict.is_empty() {
            return Err(VisionError::IoError {
                path: dict_path.to_string_lossy().to_string(),
                e: "字典文件为空".to_string(),
            });
        }

        Ok(dict)
    }
}
