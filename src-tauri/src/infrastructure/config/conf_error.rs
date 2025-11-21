use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ConfigError {
    #[error("序列化文件{path}内容失败: {e}")]
    SerializeErr { path: String, e: String },

    #[error("反序列化{path}内容失败: {e}")]
    DeserializeErr { path: String, e: String },

    #[error("读取文件{path}失败: {e}")]
    LoadErr { path: String, e: String },

    #[error("配置{conf_category} 未初始化: {e}")]
    NotInitErr { conf_category: String, e: String },

    #[error("类型转换失败: {e}")]
    CastErr { e: String },

    #[error("配置{path}写入失败: {e}")]
    WriteErr { path: String, e: String },
}

pub type ConfigResult<T> = Result<T, ConfigError>;

pub use ConfigError::*;
