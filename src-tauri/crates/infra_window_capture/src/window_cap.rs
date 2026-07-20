use crate::{monitor_capture, wgc_capture};
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
    DwmGetDxSharedSurface,
    Wgc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowCaptureOffsets {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

#[derive(Clone, Debug)]
pub struct WindowCaptureConfig {
    pub title: String,
    pub interface: WindowCaptureInterface,
    pub frame_timeout: Duration,
    pub offsets: WindowCaptureOffsets,
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

    fn normalize_window_image(
        image: RgbaImage,
        window_width: u32,
        window_height: u32,
    ) -> RgbaImage {
        if image.dimensions() == (window_width, window_height) {
            return image;
        }
        tracing::debug!(
            "窗口截图尺寸由{}x{}归一化为{}x{}",
            image.width(),
            image.height(),
            window_width,
            window_height
        );
        image::imageops::resize(
            &image,
            window_width,
            window_height,
            image::imageops::FilterType::Triangle,
        )
    }

    fn crop_window_content(
        image: RgbaImage,
        window_width: u32,
        window_height: u32,
        offsets: WindowCaptureOffsets,
    ) -> Result<RgbaImage, String> {
        let (content_width, content_height) =
            Self::content_dimensions(window_width, window_height, offsets)?;
        let image = Self::normalize_window_image(image, window_width, window_height);
        Ok(image::imageops::crop_imm(
            &image,
            offsets.left,
            offsets.top,
            content_width,
            content_height,
        )
        .to_image())
    }

    fn content_dimensions(
        window_width: u32,
        window_height: u32,
        offsets: WindowCaptureOffsets,
    ) -> Result<(u32, u32), String> {
        let horizontal = offsets
            .left
            .checked_add(offsets.right)
            .ok_or_else(|| "窗口水平偏移相加溢出".to_string())?;
        let vertical = offsets
            .top
            .checked_add(offsets.bottom)
            .ok_or_else(|| "窗口垂直偏移相加溢出".to_string())?;
        if horizontal >= window_width || vertical >= window_height {
            return Err(format!(
                "窗口偏移超出截图范围: window={}x{}, offsets={},{},{},{}",
                window_width,
                window_height,
                offsets.left,
                offsets.top,
                offsets.right,
                offsets.bottom
            ));
        }
        Ok((window_width - horizontal, window_height - vertical))
    }

    fn capture_window_via_gdi(
        window: &Window,
        offsets: WindowCaptureOffsets,
    ) -> Result<RgbaImage, String> {
        let (_, _, window_width, window_height) = Self::window_bounds(window)?;
        let image = window
            .capture_image()
            .map_err(|error| format!("GDI窗口截图失败: {error}"))?;
        Self::crop_window_content(image, window_width, window_height, offsets)
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

    fn window_content_bounds(
        window: &Window,
        offsets: WindowCaptureOffsets,
    ) -> Result<(i32, i32, u32, u32), String> {
        let (window_x, window_y, window_width, window_height) = Self::window_bounds(window)?;
        let (content_width, content_height) =
            Self::content_dimensions(window_width, window_height, offsets)?;
        Ok((
            window_x + offsets.left as i32,
            window_y + offsets.top as i32,
            content_width,
            content_height,
        ))
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
        offsets: WindowCaptureOffsets,
    ) -> Result<RgbaImage, String> {
        let started = Instant::now();
        loop {
            if window.is_minimized().unwrap_or(false) {
                return Err("目标窗口已最小化，无法执行 DXGI 截图".to_string());
            }

            let (capture_x, capture_y, capture_width, capture_height) =
                Self::window_content_bounds(window, offsets)?;
            let remaining = frame_timeout.saturating_sub(started.elapsed());
            let wait_slice = remaining.min(Duration::from_millis(250));
            let wait_timeout_ms = wait_slice.as_millis().max(1) as u32;

            match monitor_capture::capture_window_region(
                capture_x,
                capture_y,
                capture_width,
                capture_height,
                wait_timeout_ms,
            ) {
                Ok(image) => return Ok(image),
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

    fn capture_window_via_dwm_shared_surface(
        window: &Window,
        offsets: WindowCaptureOffsets,
    ) -> Result<RgbaImage, String> {
        let (_, _, window_width, window_height) = Self::window_bounds(window)?;
        let hwnd = window
            .id()
            .map_err(|error| format!("读取窗口句柄失败: {error}"))? as isize;
        let image = monitor_capture::capture_window_shared_surface(hwnd)
            .map_err(|error| format!("DwmGetDxSharedSurface窗口截图失败: {error}"))?;
        Self::crop_window_content(image, window_width, window_height, offsets)
    }

    fn capture_window_via_wgc(
        window: &Window,
        frame_timeout: Duration,
        offsets: WindowCaptureOffsets,
    ) -> Result<RgbaImage, String> {
        let (_, _, window_width, window_height) = Self::window_bounds(window)?;
        let hwnd = window
            .id()
            .map_err(|error| format!("读取窗口句柄失败: {error}"))? as isize;
        let image = wgc_capture::capture_window(hwnd, frame_timeout)
            .map_err(|error| format!("WGC窗口截图失败: {error}"))?;
        Self::crop_window_content(image, window_width, window_height, offsets)
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
        let offsets = self.config.offsets;
        self.capture_with_retry_result("GDI窗口截图", move |window| {
            Self::capture_window_via_gdi(window, offsets)
        })
        .await
    }

    pub(crate) async fn capture_image_via_dxgi_region_result(&self) -> Result<RgbaImage, String> {
        let frame_timeout = self.config.frame_timeout;
        let offsets = self.config.offsets;
        self.capture_with_retry_result("DXGI监视器区域截图", move |window| {
            Self::capture_window_via_dxgi_region(window, frame_timeout, offsets)
        })
        .await
    }

    pub(crate) async fn capture_image_via_dwm_shared_surface_result(
        &self,
    ) -> Result<RgbaImage, String> {
        let offsets = self.config.offsets;
        self.capture_with_retry_result("DwmGetDxSharedSurface窗口截图", move |window| {
            Self::capture_window_via_dwm_shared_surface(window, offsets)
        })
        .await
    }

    pub(crate) async fn capture_image_via_wgc_result(&self) -> Result<RgbaImage, String> {
        let frame_timeout = self.config.frame_timeout;
        let offsets = self.config.offsets;
        self.capture_with_retry_result("WGC窗口截图", move |window| {
            Self::capture_window_via_wgc(window, frame_timeout, offsets)
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
            WindowCaptureInterface::DwmGetDxSharedSurface => {
                self.capture_image_via_dwm_shared_surface_result().await
            }
            WindowCaptureInterface::Wgc => self.capture_image_via_wgc_result().await,
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
    use super::{WindowCaptureConfig, WindowCaptureInterface, WindowCaptureOffsets, WindowInfo};
    use crate::wgc_capture;
    use std::time::{Duration, Instant};
    use std::{env, fs, path::PathBuf};
    use tokio::runtime::Builder;

    #[test]
    fn crops_window_offsets_from_the_configured_origin() {
        let image = image::RgbaImage::from_fn(4, 4, |x, y| image::Rgba([x as u8, y as u8, 0, 255]));
        let cropped = WindowInfo::crop_window_content(
            image,
            4,
            4,
            WindowCaptureOffsets {
                left: 1,
                top: 1,
                right: 1,
                bottom: 1,
            },
        )
        .unwrap();

        assert_eq!(cropped.dimensions(), (2, 2));
        assert_eq!(cropped.get_pixel(0, 0).0, [1, 1, 0, 255]);
    }

    #[test]
    fn normalizes_window_size_before_cropping_title_bar() {
        let image = image::RgbaImage::new(4, 6);
        let cropped = WindowInfo::crop_window_content(
            image,
            2,
            3,
            WindowCaptureOffsets {
                left: 0,
                top: 1,
                right: 0,
                bottom: 0,
            },
        )
        .unwrap();

        assert_eq!(cropped.dimensions(), (2, 2));
    }

    fn read_test_config() -> (String, usize, WindowCaptureOffsets) {
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
        let offsets_value =
            env::var("OFFSET_LEFT_TOP_RIGHT_BOTTOM").unwrap_or_else(|_| "1,40,1,1".to_string());
        let values = offsets_value
            .split(',')
            .map(|value| {
                value.trim().parse::<u32>().unwrap_or_else(|_| {
                    panic!("OFFSET_LEFT_TOP_RIGHT_BOTTOM 必须是四个非负整数，例如 1,40,1,1")
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values.len(),
            4,
            "OFFSET_LEFT_TOP_RIGHT_BOTTOM 必须按 left,top,right,bottom 提供四个数"
        );
        let offsets = WindowCaptureOffsets {
            left: values[0],
            top: values[1],
            right: values[2],
            bottom: values[3],
        };
        (title, rounds, offsets)
    }

    fn init_test_window(
        title: String,
        interface: WindowCaptureInterface,
        offsets: WindowCaptureOffsets,
    ) -> WindowInfo {
        WindowInfo::init(WindowCaptureConfig {
            title,
            interface,
            frame_timeout: Duration::from_secs(10),
            offsets,
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

    fn measure_capture<F>(label: &str, rounds: usize, mut capture: F) -> bool
    where
        F: FnMut() -> Result<image::RgbaImage, String>,
    {
        let warmup_started = Instant::now();
        let warmup = match capture() {
            Ok(image) => image,
            Err(error) => {
                println!("label={label} skipped error={error}");
                return false;
            }
        };
        let warmup_elapsed = warmup_started.elapsed();

        let mut total_elapsed = Duration::ZERO;
        let mut min_elapsed = Duration::MAX;
        let mut max_elapsed = Duration::ZERO;
        let mut last_image = warmup.clone();

        for index in 0..rounds {
            let started = Instant::now();
            let image = match capture() {
                Ok(image) => image,
                Err(error) => {
                    println!("label={label} round={} failed error={error}", index + 1);
                    return false;
                }
            };
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
        true
    }

    #[test]
    #[ignore = "本地手动测试：设置 WINDOW_CAP_TEST_TITLE，可选 WINDOW_CAP_TEST_ROUNDS、OFFSET_LEFT_TOP_RIGHT_BOTTOM 后运行"]
    fn test_capture_timing_for_window_title() {
        let (title, rounds, offsets) = read_test_config();
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let window = init_test_window(title.clone(), WindowCaptureInterface::Dxgi, offsets);
        println!(
            "window_title={:?} rounds={} offsets={},{},{},{} dxgi_supported={}",
            title,
            rounds,
            offsets.left,
            offsets.top,
            offsets.right,
            offsets.bottom,
            WindowInfo::is_dxgi_supported()
        );
        assert!(measure_capture("auto", rounds, || {
            runtime.block_on(window.capture_image_result())
        }));
    }

    #[test]
    #[ignore = "本地手动测试：设置 WINDOW_CAP_TEST_TITLE，可选 WINDOW_CAP_TEST_ROUNDS、OFFSET_LEFT_TOP_RIGHT_BOTTOM 后运行"]
    fn test_compare_capture_timing_for_window_title() {
        let (title, rounds, offsets) = read_test_config();
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        let dxgi_window = init_test_window(title.clone(), WindowCaptureInterface::Dxgi, offsets);
        let gdi_window = init_test_window(title.clone(), WindowCaptureInterface::Gdi, offsets);
        let dwm_window = init_test_window(
            title.clone(),
            WindowCaptureInterface::DwmGetDxSharedSurface,
            offsets,
        );
        let wgc_window = init_test_window(title.clone(), WindowCaptureInterface::Wgc, offsets);
        println!(
            "window_title={:?} rounds={} offsets={},{},{},{} dxgi_supported={} wgc_supported={}",
            title,
            rounds,
            offsets.left,
            offsets.top,
            offsets.right,
            offsets.bottom,
            WindowInfo::is_dxgi_supported(),
            wgc_capture::is_supported()
        );
        let mut successful_methods = 0;
        if WindowInfo::is_dxgi_supported()
            && measure_capture("dxgi_monitor_region", rounds, || {
                runtime.block_on(dxgi_window.capture_image_via_dxgi_region_result())
            })
        {
            successful_methods += 1;
        }
        if measure_capture("dwm_get_dx_shared_surface", rounds, || {
            runtime.block_on(dwm_window.capture_image_via_dwm_shared_surface_result())
        }) {
            successful_methods += 1;
        }
        if wgc_capture::is_supported()
            && measure_capture("wgc_window", rounds, || {
                runtime.block_on(wgc_window.capture_image_via_wgc_result())
            })
        {
            successful_methods += 1;
        }
        if measure_capture("gdi_window", rounds, || {
            runtime.block_on(gdi_window.capture_image_via_gdi_result())
        }) {
            successful_methods += 1;
        }
        assert!(successful_methods > 0, "所有窗口截图方式均失败");
    }
}
