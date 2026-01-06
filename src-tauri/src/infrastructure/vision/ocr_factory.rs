use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::execution_provider_mgr::InferenceBackend;
use crate::infrastructure::vision::base_traits::{TextDetector, TextRecognizer};
use crate::infrastructure::vision::det::paddle_dbnet::PaddleDetDbNet;
use crate::infrastructure::vision::det::yolo::YoloDet;
use crate::infrastructure::vision::rec::paddle_crnn::PaddleRecCrnn;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
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

/// 检测器配置
#[derive(Debug, Serialize, Deserialize)]
pub struct DetectorConfig {
    pub detector_type: DetectorType,
    pub model_path: PathBuf,
    pub execution_provider: InferenceBackend, // "cuda", "dml", "cpu"
    pub input_width: u32,
    pub input_height: u32,

    pub intra_thread_num: usize,
    pub intra_spinning: bool,
    pub inter_thread_num: usize,
    pub inter_spinning: bool,

    // YOLO特有配置
    pub confidence_thresh: Option<f32>,
    pub iou_thresh: Option<f32>,
    pub class_count: Option<usize>,
    pub class_labels: Option<Vec<String>>,
    pub class_file_path: Option<PathBuf>,

    // DBNet特有配置
    pub db_thresh: Option<f32>,
    pub db_box_thresh: Option<f32>,
    pub unclip_ratio: Option<f32>,
    pub use_dilation: Option<bool>,
}

impl DetectorConfig {
    pub fn new_yolo(
        detector_type: DetectorType,
        model_path: PathBuf,
        execution_provider: InferenceBackend, // "cuda", "dml", "cpu"
        input_width: u32,
        input_height: u32,

        intra_thread_num: usize,
        intra_spinning: bool,
        inter_thread_num: usize,
        inter_spinning: bool,

        // YOLO特有配置
        confidence_thresh: Option<f32>,
        iou_thresh: Option<f32>,
        class_count: Option<usize>,
        class_labels: Option<Vec<String>>,
        class_file_path: Option<PathBuf>,
    ) -> Self {
        Self {
            detector_type,
            model_path,
            execution_provider,
            input_width,
            input_height,
            intra_thread_num,
            intra_spinning,
            inter_thread_num,
            inter_spinning,
            confidence_thresh,
            iou_thresh,
            class_count,
            class_labels,
            class_file_path,
            //dbnet
            db_thresh: None,
            db_box_thresh: None,
            unclip_ratio: None,
            use_dilation: None,
        }
    }

    pub fn new_paddle_det(
        detector_type: DetectorType,
        model_path: PathBuf,
        execution_provider: InferenceBackend, // "cuda", "dml", "cpu"
        input_width: u32,
        input_height: u32,

        intra_thread_num: usize,
        intra_spinning: bool,
        inter_thread_num: usize,
        inter_spinning: bool,

        // DBNet特有配置
        db_thresh: Option<f32>,
        db_box_thresh: Option<f32>,
        unclip_ratio: Option<f32>,
        use_dilation: Option<bool>,
    ) -> Self {
        Self {
            detector_type,
            model_path,
            execution_provider,
            input_width,
            input_height,
            intra_thread_num,
            intra_spinning,
            inter_thread_num,
            inter_spinning,
            confidence_thresh: None,
            iou_thresh: None,
            class_count: None,
            class_labels: None,
            class_file_path: None,
            db_thresh,
            db_box_thresh,
            unclip_ratio,
            use_dilation,
        }
    }
}

/// 识别器配置
#[derive(Debug, Serialize, Deserialize)]
pub struct RecognizerConfig {
    pub recognizer_type: RecognizerType,
    pub model_path: PathBuf,
    pub execution_provider: InferenceBackend,
    pub input_width: u32,
    pub input_height: u32,
    pub dict_path: Option<PathBuf>,

    pub intra_thread_num: usize,
    pub intra_spinning: bool,
    pub inter_thread_num: usize,
    pub inter_spinning: bool,
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
