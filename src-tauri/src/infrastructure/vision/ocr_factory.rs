use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::execution_provider_mgr::InferenceBackend;
use crate::infrastructure::vision::base_traits::{TextDetector, TextRecognizer};
use crate::infrastructure::vision::det::paddle_dbnet::PaddleDetDbNet;
use crate::infrastructure::vision::det::yolo::YoloDet;
use crate::infrastructure::vision::rec::paddle_crnn::PaddleRecCrnn;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{read_to_string, File};

/// 检测器类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DetectorType {
    Yolo11,
    PaddleDbNet,
}

/// 识别器类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecognizerType {
    PaddleCrnn,
}

/// 检测器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    async fn open_model_file(path: &PathBuf) -> VisionResult<File> {
        File::open(path)
            .await
            .map_err(|e| VisionError::OpenModelFailed {
                path: path.to_string_lossy().to_string(),
                e: e.to_string(),
            })
    }
    /// 内部方法：创建检测器实现（不通过管理器）
    pub(crate) async fn create_detector(
        config: DetectorConfig,
    ) -> VisionResult<Arc<dyn TextDetector>> {
        let mmap = Self::mapping_model_file(&config.model_path)?;

        match config.detector_type {
            DetectorType::Yolo11 => {
                let detector = YoloDet::new(
                    config.input_width,
                    config.input_height,
                    config.intra_thread_num,
                    config.intra_spinning,
                    config.inter_thread_num,
                    config.inter_spinning,
                    mmap,
                    config.execution_provider,
                    config.class_count.unwrap_or(1),
                    config
                        .class_labels
                        .unwrap_or_else(|| vec!["text".to_string()]),
                    config.confidence_thresh.unwrap_or(0.5),
                    config.iou_thresh.unwrap_or(0.4),
                );
                Ok(Arc::new(detector))
            }
            DetectorType::PaddleDbNet => {
                let detector = PaddleDetDbNet::new(
                    config.input_width,
                    config.input_height,
                    config.intra_thread_num,
                    config.intra_spinning,
                    config.inter_thread_num,
                    config.inter_spinning,
                    mmap,
                    config.execution_provider,
                    config.db_thresh.unwrap_or(0.3),
                    config.db_box_thresh.unwrap_or(0.6),
                    config.unclip_ratio.unwrap_or(1.5),
                    config.use_dilation.unwrap_or(false),
                );
                Ok(Arc::new(detector))
            }
        }
    }

    /// 内部方法：创建识别器实现（不通过管理器）
    pub(crate) async fn create_recognizer(
        config: RecognizerConfig,
    ) -> VisionResult<Arc<dyn TextRecognizer>> {
        let mmap = Self::mapping_model_file(&config.model_path)?;

        match config.recognizer_type {
            RecognizerType::PaddleCrnn => {
                // 加载字典
                let dict = if let Some(dict_path) = config.dict_path {
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
                    /*(0..=9).map(|i| i.to_string())
                    .chain(('a'..='z').map(|c| c.to_string()))
                    .chain(('A'..='Z').map(|c| c.to_string()))
                    .collect()*/
                };

                let recognizer = PaddleRecCrnn::new(
                    config.input_width,
                    config.input_height,
                    config.intra_thread_num,
                    config.intra_spinning,
                    config.inter_thread_num,
                    config.inter_spinning,
                    mmap,
                    config.execution_provider,
                    dict,
                );
                Ok(Arc::new(recognizer))
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

    async fn mapping_model_file(path : &PathBuf) -> VisionResult<Mmap> {
        let file = Self::open_model_file(path).await?;
        let map = unsafe {
            Mmap::map(&file).map_err(|e| VisionError::MappingErr {
                path: path.to_string_lossy().to_string(),
                e: e.to_string(),
            })?
        };
        Ok(map)
    }
}
