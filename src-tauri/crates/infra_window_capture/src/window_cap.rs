use crate::monitor_capture;
use image::RgbaImage;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use xcap::Window;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCaptureInterface {
    Dxgi,
    Gdi,
}

#[derive(Clone, Debug)]
pub struct WindowCaptureConfig {
    pub title: String,
    pub interface: WindowCaptureInterface,
    pub frame_timeout: Duration,
    pub title_bar_height_px: u32,
}

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub window: Arc<RwLock<Option<Window>>>,
    pub config: WindowCaptureConfig,
}

impl WindowInfo {
    pub fn init(config: WindowCaptureConfig) -> Self {
        let title = config.title.trim().to_string();
        if title.is_empty() {
            tracing::error!("窗口名称未设置！");
            return Self {
                window: Arc::new(RwLock::new(None)),
                config,
            };
        }
        Self {
            window: Arc::new(RwLock::new(Self::find_window(&title))),
            config: WindowCaptureConfig { title, ..config },
        }
    }

    pub(crate) async fn refresh_window(&self) -> bool {
        let window_name = self.config.title.trim();
        if window_name.is_empty() {
            tracing::error!("未配置目标窗口标题，无法刷新窗口句柄！");
            *self.window.write().await = None;
            return false;
        }

        let window = Self::find_window(window_name);
        let found = window.is_some();
        *self.window.write().await = window;
        found
    }

    pub async fn valid_capture(&self) -> bool {
        if self.window.read().await.is_some() {
            return true;
        }
        self.refresh_window().await
    }

    async fn ensure_window(&self) -> Option<Window> {
        if self.window.read().await.is_none() {
            let target = if self.config.title.trim().is_empty() {
                "<unknown>".to_string()
            } else {
                self.config.title.clone()
            };
            tracing::warn!("窗口截图目标未初始化，尝试重新查找目标窗口: {target}");
            if !self.refresh_window().await {
                tracing::warn!("重新查找目标窗口失败");
                return None;
            }
        }
        self.window.read().await.clone()
    }

    fn crop_title_bar(image: RgbaImage, title_bar_height_px: u32) -> Result<RgbaImage, String> {
        if title_bar_height_px == 0 {
            return Ok(image);
        }
        if title_bar_height_px >= image.height() {
            return Err(format!(
                "标题栏高度({title_bar_height_px}px)超出截图高度({}px)",
                image.height()
            ));
        }
        Ok(image::imageops::crop_imm(
            &image,
            0,
            title_bar_height_px,
            image.width(),
            image.height() - title_bar_height_px,
        )
        .to_image())
    }

    fn capture_window_via_gdi(
        window: &Window,
        title_bar_height_px: u32,
    ) -> Result<RgbaImage, String> {
        let image = window
            .capture_image()
            .map_err(|error| format!("GDI窗口截图失败: {error}"))?;
        Self::crop_title_bar(image, title_bar_height_px)
    }

    fn window_bounds(window: &Window) -> Result<(i32, i32, u32, u32), String> {
        let window_x = window
            .x()
            .map_err(|error| format!("读取窗口x失败: {error}"))?;
        let window_y = window
            .y()
            .map_err(|error| format!("读取窗口y失败: {error}"))?;
        let window_width = window
            .width()
            .map_err(|error| format!("读取窗口宽度失败: {error}"))?;
        let window_height = window
            .height()
            .map_err(|error| format!("读取窗口高度失败: {error}"))?;
        Ok((window_x, window_y, window_width, window_height))
    }

    fn is_dxgi_wait_error(error: &str) -> bool {
        error.contains("DXGI AcquireNextFrame timeout") || error.contains("DXGI 没有新帧")
    }

    fn is_wait_state_error(error: &str) -> bool {
        Self::is_dxgi_wait_error(error) || error.contains("DXGI 等待新帧超时")
    }

    fn capture_window_via_dxgi_region(
        window: &Window,
        frame_timeout: Duration,
        title_bar_height_px: u32,
    ) -> Result<RgbaImage, String> {
        let started = Instant::now();
        loop {
            if window.is_minimized().unwrap_or(false) {
                return Err("目标窗口已最小化，无法执行 DXGI 截图".to_string());
            }

            let (window_x, window_y, window_width, window_height) = Self::window_bounds(window)?;
            let remaining = frame_timeout.saturating_sub(started.elapsed());
            let wait_slice = remaining.min(Duration::from_millis(250));
            let wait_timeout_ms = wait_slice.as_millis().max(1) as u32;

            match monitor_capture::capture_window_region(
                window_x,
                window_y,
                window_width,
                window_height,
                wait_timeout_ms,
            ) {
                Ok(image) => return Self::crop_title_bar(image, title_bar_height_px),
                Err(error)
                    if Self::is_dxgi_wait_error(&error) && started.elapsed() < frame_timeout =>
                {
                    continue;
                }
                Err(error) if Self::is_dxgi_wait_error(&error) => {
                    return Err(format!(
                        "DXGI 等待新帧超时，超过 {:?}: {}",
                        frame_timeout, error
                    ));
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn capture_with_retry_result<F>(
        &self,
        label: &str,
        capture_once: F,
    ) -> Result<RgbaImage, String>
    where
        F: Fn(&Window) -> Result<RgbaImage, String>,
    {
        if let Some(window) = self.ensure_window().await {
            match capture_once(&window) {
                Ok(image) => return Ok(image),
                Err(error) if Self::is_wait_state_error(&error) => return Err(error),
                Err(error) => {
                    tracing::warn!("{label}失败，尝试重新绑定窗口: {error}");
                }
            }
        }

        if !self.refresh_window().await {
            tracing::warn!("重新绑定目标窗口失败");
            return Err("窗口绑定失败：重新绑定目标窗口失败".to_string());
        }

        let Some(window) = self.window.read().await.clone() else {
            return Err("窗口绑定失败：重新绑定后窗口句柄为空".to_string());
        };
        match capture_once(&window) {
            Ok(img) => Ok(img),
            Err(error) if Self::is_wait_state_error(&error) => Err(error),
            Err(error) => {
                tracing::error!("{label}失败：{error}");
                Err(error)
            }
        }
    }

    pub(crate) fn is_dxgi_supported() -> bool {
        monitor_capture::is_supported()
    }

    pub(crate) async fn capture_image_via_gdi_result(&self) -> Result<RgbaImage, String> {
        let title_bar_height_px = self.config.title_bar_height_px;
        self.capture_with_retry_result("GDI窗口截图", move |window| {
            Self::capture_window_via_gdi(window, title_bar_height_px)
        })
        .await
    }

    pub(crate) async fn capture_image_via_dxgi_region_result(&self) -> Result<RgbaImage, String> {
        let frame_timeout = self.config.frame_timeout;
        let title_bar_height_px = self.config.title_bar_height_px;
        self.capture_with_retry_result("DXGI监视器区域截图", move |window| {
            Self::capture_window_via_dxgi_region(window, frame_timeout, title_bar_height_px)
        })
        .await
    }

    fn effective_interface(&self) -> WindowCaptureInterface {
        match self.config.interface {
            WindowCaptureInterface::Dxgi if !Self::is_dxgi_supported() => {
                WindowCaptureInterface::Gdi
            }
            interface => interface,
        }
    }

    pub async fn capture_image_result(&self) -> Result<RgbaImage, String> {
        match self.effective_interface() {
            WindowCaptureInterface::Dxgi => self.capture_image_via_dxgi_region_result().await,
            WindowCaptureInterface::Gdi => {
                if self.config.interface == WindowCaptureInterface::Dxgi {
                    tracing::warn!("DXGI 不可用，已回退为 GDI 窗口截图");
                }
                self.capture_image_via_gdi_result().await
            }
        }
    }

    fn find_window(window_name: &str) -> Option<Window> {
        let target_name = window_name.to_lowercase();
        let windows = match Window::all() {
            Ok(windows) => windows,
            Err(error) => {
                tracing::error!("获取窗口列表失败: {error:?}");
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

#[cfg(test)]
mod tests {
    use super::{WindowCaptureConfig, WindowCaptureInterface, WindowInfo};
    use std::time::{Duration, Instant};
    use std::{env, fs, path::PathBuf};
    use tokio::runtime::Builder;

    #[test]
    fn crops_the_title_bar_without_hardware() {
        let image = image::RgbaImage::new(2, 3);
        let cropped = WindowInfo::crop_title_bar(image, 1).unwrap();

        assert_eq!(cropped.dimensions(), (2, 2));
    }

    fn read_test_config() -> (String, usize) {
        let title = env::var("WINDOW_CAP_TEST_TITLE")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .expect("请先设置 WINDOW_CAP_TEST_TITLE 环境变量");
        let rounds = env::var("WINDOW_CAP_TEST_ROUNDS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(10);
        (title, rounds)
    }

    fn init_test_window(title: String, interface: WindowCaptureInterface) -> WindowInfo {
        WindowInfo::init(WindowCaptureConfig {
            title,
            interface,
            frame_timeout: Duration::from_secs(10),
            title_bar_height_px: 122,
        })
    }

    fn output_dir() -> PathBuf {
        PathBuf::from(r"D:\AdminFiles\Pictures")
    }

    fn save_capture(label: &str, image: &image::RgbaImage) {
        let dir = output_dir();
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("创建截图输出目录失败 {}: {error}", dir.display()));
        let path = dir.join(format!("window_capture_{label}.png"));
        image
            .save(&path)
            .unwrap_or_else(|error| panic!("保存截图失败 {}: {error}", path.display()));
        println!("saved_capture label={} path={}", label, path.display());
    }

    fn measure_capture<F>(label: &str, rounds: usize, mut capture: F) -> image::RgbaImage
    where
        F: FnMut() -> image::RgbaImage,
    {
        let warmup_started = Instant::now();
        let warmup = capture();
        let warmup_elapsed = warmup_started.elapsed();

        let mut total_elapsed = Duration::ZERO;
        let mut min_elapsed = Duration::MAX;
        let mut max_elapsed = Duration::ZERO;
        let mut last_image = warmup.clone();

        for index in 0..rounds {
            let started = Instant::now();
            let image = capture();
            let elapsed = started.elapsed();

            total_elapsed += elapsed;
            min_elapsed = min_elapsed.min(elapsed);
            max_elapsed = max_elapsed.max(elapsed);

            println!(
                "round={} elapsed={:?} size={}x{}",
                index + 1,
                elapsed,
                image.width(),
                image.height()
            );
            last_image = image;
        }

        let avg_elapsed = total_elapsed / rounds as u32;
        println!(
            "label={} rounds={} warmup={:?} total={:?} avg={:?} min={:?} max={:?} warmup_size={}x{}",
            label,
            rounds,
            warmup_elapsed,
            total_elapsed,
            avg_elapsed,
            min_elapsed,
            max_elapsed,
            warmup.width(),
            warmup.height()
        );
        save_capture(label, &last_image);
        last_image
    }

    #[test]
    #[ignore = "本地手动测试：设置 WINDOW_CAP_TEST_TITLE，可选 WINDOW_CAP_TEST_ROUNDS 后运行"]
    fn test_capture_timing_for_window_title() {
        let (title, rounds) = read_test_config();
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let window = init_test_window(title.clone(), WindowCaptureInterface::Dxgi);
        println!(
            "window_title={:?} rounds={} dxgi_supported={}",
            title,
            rounds,
            WindowInfo::is_dxgi_supported()
        );
        let _ = measure_capture("auto", rounds, || {
            runtime
                .block_on(window.capture_image_result())
                .unwrap_or_else(|error| panic!("自动截图失败: {error}"))
        });
    }

    #[test]
    #[ignore = "本地手动测试：设置 WINDOW_CAP_TEST_TITLE，可选 WINDOW_CAP_TEST_ROUNDS 后运行"]
    fn test_compare_capture_timing_for_window_title() {
        let (title, rounds) = read_test_config();
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let dxgi_window = init_test_window(title.clone(), WindowCaptureInterface::Dxgi);
        let gdi_window = init_test_window(title.clone(), WindowCaptureInterface::Gdi);
        println!(
            "window_title={:?} rounds={} dxgi_supported={}",
            title,
            rounds,
            WindowInfo::is_dxgi_supported()
        );
        if WindowInfo::is_dxgi_supported() {
            let _ = measure_capture("dxgi_monitor_region", rounds, || {
                runtime
                    .block_on(dxgi_window.capture_image_via_dxgi_region_result())
                    .unwrap_or_else(|error| panic!("DXGI监视器区域截图失败: {error}"))
            });
        }
        let _ = measure_capture("gdi_window", rounds, || {
            runtime
                .block_on(gdi_window.capture_image_via_gdi_result())
                .unwrap_or_else(|error| panic!("GDI窗口截图失败: {error}"))
        });
        let _ = measure_capture("auto", rounds, || {
            runtime
                .block_on(dxgi_window.capture_image_result())
                .unwrap_or_else(|error| panic!("自动截图失败: {error}"))
        });
    }
}
