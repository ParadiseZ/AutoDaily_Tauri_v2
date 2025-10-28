use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AdbError{
    #[error("未找到adb路径")]
    AdbNotFound,

    #[error("获取设备列表失败")]
    GetDevicesFailed,

    #[error("adb服务未启动")]
    ServerNotStarted
}

pub type AdbResult<T> = Result<T, AdbError>;
pub use AdbError::*;