
#[derive(Debug)]
pub enum OrtError {
    #[error("{method} 模型加载错误: {e}")]
    LoadModelErr { method: String ,e: String },
}