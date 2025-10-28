use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::services::capture_basis::CaptureHandler;
use image::RgbaImage;
use xcap::Window;

pub struct WindowInfo{
    pub window : Option<Window>,
    pub title : Option<String>
}

impl WindowInfo{
    fn init(window_name : &str) -> Self{
        // 获取所有窗口
        let windows =  Window::all();
        if let Err(e) = windows{
            Log::error(&format!("获取窗口列表失败: {:?}", e));
            return Self{
                window : None,
                title : None,
            }
        }
        for window in windows.unwrap() {
            // 最小化的窗口不能截屏
            if let Ok(is_min) = window.is_minimized() {
                if is_min {
                    continue;
                }
            }

            let title = window.title().unwrap_or_else(|_| "无标题".to_string());
            Log::info(&format!("发现窗口: {}", title));
            // 检查是否是目标窗口
            if title.contains(window_name) {
                Log::info(&format!("找到目标窗口: {}", title));
                // 找到并截图后退出循环
                return Self{
                    window : Some(window),
                    title : Some(title)
                }
            }
        }
        Self{
            window : None,
            title : None,
        }
    }
}

impl CaptureHandler for WindowInfo{
    fn capture(&self) -> AppResult<RgbaImage> {
        // 捕获窗口图像
        if let Some(win) = &self.window{
            let img = win.capture_image().map_err(|e| AppError::CaptureError(format!("截图失败：{}",e)))?;
            Ok(img)
        }else { 
            Err(AppError::CaptureError("截图失败:窗口未初始化或窗口已最小化".into()))
        }
    }
}