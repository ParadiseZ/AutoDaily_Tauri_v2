use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AdbError {
    #[error("连接设备{ipv4}失败:{e}")]
    ConnectFailed { ipv4: String, e: String },

    #[error("未找到adb路径")]
    AdbNotFound,

    #[error("adb配置失败")]
    ConfigErr { detail: String },

    #[error("通过设备名称连接设备失败{detail}")]
    GetDeviceByNameFailed { detail: String },

    #[error("连接设备{ipv4addr}失败")]
    GetDeviceByIpFailed { ipv4addr: String },

    #[error("获取锁失败")]
    GetLockFailed,

    #[error("获取设备列表失败")]
    GetDevicesFailed,

    #[error("adb服务未启动")]
    ServerNotStarted,

    #[error("发送截图到流程中失败！")]
    SendPictureFailed,

    #[error("执行shell命令{cmd}失败：{e}")]
    ExecuteShellFailed { cmd: String, e: String },
}

pub type AdbResult<T> = Result<T, AdbError>;
