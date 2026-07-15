use crate::infra::vision::base_model;
use crate::infra::vision::rec::paddle_crnn;
use crate::infra::vision::vision_error::VisionResult;
use domain_vision::{BaseModel, PaddleRecCrnn};
use std::path::PathBuf;

/// Resolves the model file selected by a persisted vision configuration.
pub fn resolve_model_path(config: &BaseModel) -> VisionResult<PathBuf> {
    base_model::resolve_model_config_path(config)
}

/// Resolves the recognition dictionary selected by a persisted vision configuration.
pub fn resolve_recognizer_dict_path(config: &PaddleRecCrnn) -> VisionResult<PathBuf> {
    paddle_crnn::resolve_recognizer_dict_config_path(config)
}
