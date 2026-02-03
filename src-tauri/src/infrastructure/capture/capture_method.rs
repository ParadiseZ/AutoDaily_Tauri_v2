use crate::infrastructure::logging::log_trait::Log;

#[derive(Debug, Clone)]
pub enum CaptureMethod {
    Window = 1,
    ADB = 2,
}

impl From<u8> for CaptureMethod {
    fn from(v: u8) -> Self {
        match v {
            1 => CaptureMethod::Window,
            2 => CaptureMethod::ADB,
            _ => {
                Log::warn("不支持的截图方式！将设置为窗口截图！");
                CaptureMethod::Window
            }
        }
    }
}

impl std::fmt::Display for CaptureMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureMethod::Window => write!(f, "窗口截图"),
            CaptureMethod::ADB => write!(f, "ADB截图"),
        }
    }
}