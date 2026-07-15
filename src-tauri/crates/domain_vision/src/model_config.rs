use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// User-visible source of a vision model file.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default, ts_rs::TS)]
#[ts(export)]
pub enum ModelSource {
    BuiltIn,
    #[default]
    Custom,
}

/// Stable model family used by configuration and asset resolution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum ModelType {
    Yolo11,
    Yolo26,
    PaddleDet5,
    PaddleCrnn5,
    PaddleDet6,
    PaddleCrnn6,
}

/// Requested inference provider. Provider fallback and ORT construction remain infrastructure work.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum InferenceBackend {
    Cuda,
    DirectML,
    CPU,
}

impl InferenceBackend {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cuda => "CUDA",
            Self::DirectML => "DirectML",
            Self::CPU => "CPU",
        }
    }
}

/// Persisted, runtime-independent base configuration for one model.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BaseModel {
    pub intra_thread_num: usize,
    pub intra_spinning: bool,
    pub inter_thread_num: usize,
    pub inter_spinning: bool,
    pub execution_provider: InferenceBackend,
    pub input_width: u32,
    pub input_height: u32,
    #[serde(default)]
    pub model_source: ModelSource,
    #[ts(as = "String")]
    pub model_path: PathBuf,
    pub model_type: ModelType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum YoloPostprocessKind {
    LegacyNms,
    EndToEnd,
}

impl Default for YoloPostprocessKind {
    fn default() -> Self {
        Self::LegacyNms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct YoloDet {
    pub base_model: BaseModel,
    pub class_count: usize,
    pub confidence_thresh: Option<f32>,
    pub iou_thresh: Option<f32>,
    #[ts(as = "Option<String>")]
    pub label_path: Option<PathBuf>,
    pub txt_idx: Option<u16>,
    #[serde(default)]
    pub postprocess_kind: Option<YoloPostprocessKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaddleDetDbNet {
    pub base_model: BaseModel,
    pub db_thresh: f32,
    pub db_box_thresh: f32,
    pub unclip_ratio: f32,
    pub use_dilation: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum RecResizeFilter {
    Nearest,
    Triangle,
    Gaussian,
    CatmullRom,
    Lanczos3,
}

impl Default for RecResizeFilter {
    fn default() -> Self {
        Self::Triangle
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum RecProcessingMode {
    Single,
    MicroBatch,
}

impl Default for RecProcessingMode {
    fn default() -> Self {
        Self::Single
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaddleRecCrnn {
    pub base_model: BaseModel,
    #[ts(as = "Option<String>")]
    pub dict_path: Option<PathBuf>,
    #[serde(default)]
    pub resize_filter: RecResizeFilter,
    #[serde(default)]
    pub processing_mode: RecProcessingMode,
    #[serde(default = "PaddleRecCrnn::default_micro_batch_size")]
    pub micro_batch_size: usize,
    #[serde(default = "PaddleRecCrnn::default_width_bucket_step")]
    pub width_bucket_step: u32,
    #[serde(default = "PaddleRecCrnn::default_parallel_cpu_session_intra_threads")]
    pub parallel_cpu_session_intra_threads: usize,
}

impl PaddleRecCrnn {
    pub const fn default_micro_batch_size() -> usize {
        4
    }

    pub const fn default_width_bucket_step() -> u32 {
        32
    }

    pub const fn default_parallel_cpu_session_intra_threads() -> usize {
        1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum DetectorType {
    Yolo11(YoloDet),
    Yolo26(YoloDet),
    PaddleDbNet(PaddleDetDbNet),
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum RecognizerType {
    PaddleCrnn(PaddleRecCrnn),
}
