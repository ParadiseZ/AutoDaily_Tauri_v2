use crate::infrastructure::logging::log_trait::Log;
use std::sync::Arc;
use tokio::sync::RwLock;
use xcap::Window;

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub window: Arc<RwLock<Option<Window>>>,
    pub title: Arc<RwLock<Option<String>>>,
}

impl WindowInfo {
    pub(crate) fn init(window_name: Option<String>) -> Self {
        let title = window_name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let Some(window_name) = title.clone() else {
            Log::error("窗口名称未设置！");
            return Self {
                window: Arc::new(RwLock::new(None)),
                title: Arc::new(RwLock::new(None)),
            };
        };
        Self {
            window: Arc::new(RwLock::new(Self::find_window(&window_name))),
            title: Arc::new(RwLock::new(title)),
        }
    }

    pub(crate) async fn refresh_window(&self) -> bool {
        let Some(window_name) = self.target_title().await else {
            Log::error("未配置目标窗口标题，无法刷新窗口句柄！");
            *self.window.write().await = None;
            return false;
        };

        let window = Self::find_window(&window_name);
        let found = window.is_some();
        *self.window.write().await = window;
        found
    }

    pub(crate) async fn target_title(&self) -> Option<String> {
        self.title
            .read()
            .await
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    }

    fn find_window(window_name: &str) -> Option<Window> {
        let target_name = window_name.to_lowercase();
        let windows = match Window::all() {
            Ok(windows) => windows,
            Err(error) => {
                Log::error(&format!("获取窗口列表失败: {:?}", error));
                return None;
            }
        };

        for window in windows {
            if window.is_minimized().unwrap_or(false) {
                continue;
            }
            let title = window.title().unwrap_or_else(|_| "无标题".to_string());
            if title.to_lowercase().contains(target_name.as_str()) {
                return Some(window);
            }
        }
        None
    }
}
