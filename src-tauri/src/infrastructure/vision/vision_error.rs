#[derive(Error, Debug, Serialize, Deserialize)]
pub enum VisionError {
    #[error("读取模型文件{path}失败: {e}")]
    OpenModelFailed{path: String, e:String},

    #[error(transparent)]
    LoadModelErr(#[from] OrtError::LoadModelErr),

    #[error("{method} 推理引擎配置失败: {e}")]
    SessionConfigFailed { method: String, e: String },

    #[error("{method} 数据处理失败: {e}")]
    DataProcessingErr { method: String, e: String },

    #[error("{method} 模型推理失败: {e}")]
    InferenceErr { method: String, e: String },

    #[error("文件{path}读取失败：{e}")]
    IoError { path: String, e: String },

    #[error("类型转换失败: {e}")]
    CastErr { e: String },

    #[error("配置{path}写入失败: {e}")]
    WriteErr { path: String, e: String },

    #[error("字典大小不匹配:输出：{out}，字典：{dict}")]
    DictSizeErr { out: usize, dict: usize },

    #[error("输入图像列表为空")]
    InputImageCollectionEmpty,

    #[error(transparent)]
    ImageErr(#[from] ImageError),

    #[error("输出批次和文字检测数量不一致，批次：{batch}，检测数量：{det_num}")]
    BatchMatchDetSizeFailed { batch: usize, det_num: usize },

    #[error("检测器未初始化")]
    DetectorNotInit,

    #[error("识别器未初始化")]
    RecognizeNotInit,

    #[error("映射文件{path}失败！请确认文件是否存在！")]
    MappingErr { path: String },
}

pub type VisionResult<T> = Result<T, VisionError>;

use crate::infrastructure::core::{Deserialize, Error, Serialize};
use crate::infrastructure::image::img_error::ImageError;
use crate::infrastructure::ort::ort_error::OrtError;
pub use VisionError::*;
