use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::logging::log_trait::Log;
use xcap::Window;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowInfo {
    pub window: Option<Window>,
    pub title: Option<String>,
}

impl WindowInfo {
    pub(crate) fn init(window_name: &str) -> Self {
        // 获取所有窗口
        let windows = Window::all();
        if let Err(e) = windows {
            Log::error(&format!("获取窗口列表失败: {:?}", e));
            return Self {
                window: None,
                title: None,
            };
        }
        for window in windows.unwrap() {
            // 最小化的窗口不能截屏
            if let Ok(is_min) = window.is_minimized() {
                if is_min {
                    continue;
                }
            }
            let title = window.title().unwrap_or_else(|_| "无标题".to_string());
            //Log::info(&format!("发现窗口: {}", title));
            // 检查是否是目标窗口
            if title.contains(window_name) {
                //Log::info(&format!("找到目标窗口: {}", title));
                // 找到并截图后退出循环
                return Self {
                    window: Some(window),
                    title: Some(title),
                };
            }
        }
        Self {
            window: None,
            title: None,
        }
    }
}
