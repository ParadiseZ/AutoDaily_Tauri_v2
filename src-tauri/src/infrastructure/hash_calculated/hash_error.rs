use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug,Serialize,Deserialize)]
pub enum HashError {
    #[error("文件{path}不存在")]
    FileNotFound{path: String},
}

pub type HashResult<T> = Result<T, HashError>;

