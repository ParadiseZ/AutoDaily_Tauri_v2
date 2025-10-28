use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum CapError {
    #[error("窗口截图失败: {e}")]
    WinLocalCapErr { e: String },

    #[error("窗口-网络截图失败: {e}")]
    WinNetCapErr { e: String },

    #[error("adb模拟器截图失败: {e}")]
    AdbEmuCapErr { e: String },

    #[error("adb模拟器-网络截图失败: {e}")]
    AdbEmuNetCapErr { e: String },

    #[error("adb-usb截图失败: {e}")]
    AdbUsbCapErr { e: String },

    #[error("adb-真机截图失败: {e}")]
    AdbNetCapErr { e: String },
}

pub type CapResult<T> = Result<T, CapError>;

pub use CapError::*;