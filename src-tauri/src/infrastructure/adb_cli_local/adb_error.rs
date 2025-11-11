use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AdbError{
    #[error("连接设备{ipv4}失败:{e}")]
    ConnectFailed{ipv4: String, e: String},

    #[error("未找到adb路径")]
    AdbNotFound,

    #[error("adb配置失败")]
    ConfigErr{detail: String},

    #[error("通过设备名称连接设备失败{detail}")]
    GetDeviceByNameFailed{detail: String},

    #[error("连接设备{ipv4addr}失败")]
    GetDeviceByIpFailed{ ipv4addr: String},

    #[error("获取锁失败")]
    GetLockFailed,

    #[error("获取设备列表失败")]
    GetDevicesFailed,

    #[error("adb服务未启动")]
    ServerNotStarted
}

pub type AdbResult<T> = Result<T, AdbError>;
pub use AdbError::*;