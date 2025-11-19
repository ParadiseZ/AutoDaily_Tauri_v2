use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("解析路径{path}失败: {e}")]
    ParsingFailed { path: String ,e: String },

    #[error("创建目录{path}失败: {e}")]
    CreateDirErr { path:String,e: String },
}

pub type PathResult<T> = Result<T, PathError>;