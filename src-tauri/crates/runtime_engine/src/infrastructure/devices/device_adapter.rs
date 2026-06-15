use crate::domain::devices::device_conf::DevicePlatform;
use crate::domain::scripts::point::Point;
use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::adb_cli_local::adb_context::try_get_adb_ctx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::capture::window_cap::WindowInfo;
use crate::infrastructure::logging::log_trait::Log;
use async_trait::async_trait;
use image::RgbaImage;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait DeviceAdapter: Send + Sync {
    fn platform(&self) -> DevicePlatform;

    async fn valid_capture(&self) -> bool;

    async fn capture_screen(&self) -> Option<RgbaImage>;

    async fn click(&self, point: Point<u16>) -> Result<(), String>;

    async fn long_click(&self, point: Point<u16>) -> Result<(), String>;

    async fn swipe(&self, from: Point<u16>, to: Point<u16>, duration: u64) -> Result<(), String>;

    async fn reboot(&self) -> Result<(), String>;

    async fn launch_app(&self, pkg_name: &str, activity_name: &str) -> Result<(), String>;

    async fn stop_app(&self, pkg_name: &str) -> Result<(), String>;

    async fn back(&self) -> Result<(), String>;

    async fn home(&self) -> Result<(), String>;

    async fn input_text(&self, text: &str) -> Result<(), String>;

    async fn change_cap_method(&self, method: CaptureMethod) -> bool;
}

pub struct AndroidDeviceAdapter {
    capture_method: Arc<AtomicU8>,
    cap_tx: crossbeam_channel::Sender<Result<RgbaImage, String>>,
    cap_rx: crossbeam_channel::Receiver<Result<RgbaImage, String>>,
    window_info: Arc<WindowInfo>,
}

impl AndroidDeviceAdapter {
    pub fn new(capture_method: CaptureMethod, window_title: Option<String>) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(1);
        Self {
            capture_method: Arc::new(AtomicU8::new(capture_method as u8)),
            cap_tx: tx,
            cap_rx: rx,
            window_info: Arc::new(WindowInfo::init(window_title)),
        }
    }

    ///ADB命令发送不等待结果，直接发出后续命令，ADBExecutor内部会处理命令队列和结果回传
    fn send_command(&self, command: ADBCommand) -> Result<(), String> {
        let adb_ctx = try_get_adb_ctx()?;
        adb_ctx.send_adb_cmd(&command)
    }

    async fn send_await_result_command(&self, command: ADBCommand) -> Result<(), String> {
        let adb_ctx = try_get_adb_ctx()?;
        adb_ctx.send_adb_cmd_await(command).await
    }
}

#[async_trait]
impl DeviceAdapter for AndroidDeviceAdapter {
    fn platform(&self) -> DevicePlatform {
        DevicePlatform::Android
    }

    async fn valid_capture(&self) -> bool {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("验证窗口截图设置...");
                let win = self.window_info.window.read().await;
                if win.is_none() {
                    Log::error("验证截图设置失败：未初始化窗口信息！");
                    return false;
                }
                true
            }
            2 => {
                Log::debug("验证adb截图设置...");
                match try_get_adb_ctx() {
                    Ok(adb_ctx) => adb_ctx.validate_config(),
                    Err(error) => {
                        Log::error(&format!("验证ADB截图设置失败：{}", error));
                        false
                    }
                }
            }
            _ => {
                Log::error("不支持的截图设置！");
                false
            }
        }
    }

    async fn capture_screen(&self) -> Option<RgbaImage> {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("窗口方式截图...");
                if let Some(win) = self.window_info.window.read().await.as_ref() {
                    match win.capture_image() {
                        Ok(img) => Some(img),
                        Err(e) => {
                            Log::error(&format!("截图失败：{}", e));
                            None
                        }
                    }
                } else {
                    Log::error("截图失败：未初始化目标窗口信息！");
                    None
                }
            }
            2 => {
                Log::debug("ADB方式截图...");
                let adb_ctx = match try_get_adb_ctx() {
                    Ok(adb_ctx) => adb_ctx,
                    Err(error) => {
                        Log::error(&format!("截图失败：{}", error));
                        return None;
                    }
                };
                if let Err(error) =
                    adb_ctx.send_capture_cmd(&ADBCommand::Capture(self.cap_tx.clone()))
                {
                    Log::error(&format!("截图失败：{}", error));
                    return None;
                }
                Log::debug("等待获取图像数据...");
                let cap_rx = self.cap_rx.clone();
                match tokio::task::spawn_blocking(move || {
                    cap_rx.recv_timeout(Duration::from_secs(10))
                })
                .await
                {
                    Ok(Ok(Ok(img))) => Some(img),
                    Ok(Ok(Err(error))) => {
                        Log::error(&format!("截图失败：{}", error));
                        None
                    }
                    Ok(Err(crossbeam_channel::RecvTimeoutError::Timeout)) => {
                        Log::error("截图失败：等待截图结果已超过10秒！");
                        None
                    }
                    Ok(Err(crossbeam_channel::RecvTimeoutError::Disconnected)) => {
                        Log::error("截图失败：截图数据通道已关闭！");
                        None
                    }
                    Err(error) => {
                        Log::error(&format!("截图失败：等待截图任务异常：{}", error));
                        None
                    }
                }
            }
            _ => {
                Log::warn("截图失败：不支持的截图方式！");
                None
            }
        }
    }

    async fn click(&self, point: Point<u16>) -> Result<(), String> {
        self.send_command(ADBCommand::Click(point))
    }

    async fn long_click(&self, point: Point<u16>) -> Result<(), String> {
        self.send_command(ADBCommand::LongClick(point))
    }

    async fn swipe(&self, from: Point<u16>, to: Point<u16>, duration: u64) -> Result<(), String> {
        self.send_command(ADBCommand::SwipeWithDuration(from, to, duration))
    }

    async fn reboot(&self) -> Result<(), String> {
        self.send_await_result_command(ADBCommand::Reboot).await
    }

    async fn launch_app(&self, pkg_name: &str, activity_name: &str) -> Result<(), String> {
        self.send_await_result_command(ADBCommand::StartActivity(
            pkg_name.to_string(),
            activity_name.to_string(),
        ))
        .await
    }

    async fn stop_app(&self, pkg_name: &str) -> Result<(), String> {
        self.send_await_result_command(ADBCommand::StopApp(pkg_name.to_string()))
            .await
    }

    async fn back(&self) -> Result<(), String> {
        self.send_command(ADBCommand::Back)
    }

    async fn home(&self) -> Result<(), String> {
        self.send_command(ADBCommand::Home)
    }

    async fn input_text(&self, text: &str) -> Result<(), String> {
        self.send_command(ADBCommand::InputText(text.to_string()))
    }

    async fn change_cap_method(&self, method: CaptureMethod) -> bool {
        Log::debug(format!("切换截图方式为：{}", method).as_str());
        self.capture_method.store(method as u8, Ordering::Release);
        self.valid_capture().await
    }
}

pub struct DesktopDeviceAdapter;

impl DesktopDeviceAdapter {
    pub fn new() -> Self {
        Self
    }

    fn unsupported(action: &str) -> String {
        format!("DesktopDeviceAdapter 尚未实现: {}", action)
    }
}

#[async_trait]
impl DeviceAdapter for DesktopDeviceAdapter {
    fn platform(&self) -> DevicePlatform {
        DevicePlatform::Desktop
    }

    async fn valid_capture(&self) -> bool {
        Log::warn("DesktopDeviceAdapter 尚未实现截图校验");
        false
    }

    async fn capture_screen(&self) -> Option<RgbaImage> {
        Log::warn("DesktopDeviceAdapter 尚未实现截图");
        None
    }

    async fn click(&self, _point: Point<u16>) -> Result<(), String> {
        Err(Self::unsupported("click"))
    }

    async fn long_click(&self, _point: Point<u16>) -> Result<(), String> {
        Err(Self::unsupported("long_click"))
    }

    async fn swipe(
        &self,
        _from: Point<u16>,
        _to: Point<u16>,
        _duration: u64,
    ) -> Result<(), String> {
        Err(Self::unsupported("swipe"))
    }

    async fn reboot(&self) -> Result<(), String> {
        Err(Self::unsupported("reboot"))
    }

    async fn launch_app(&self, _pkg_name: &str, _activity_name: &str) -> Result<(), String> {
        Err(Self::unsupported("launch_app"))
    }

    async fn stop_app(&self, _pkg_name: &str) -> Result<(), String> {
        Err(Self::unsupported("stop_app"))
    }

    async fn back(&self) -> Result<(), String> {
        Err(Self::unsupported("back"))
    }

    async fn home(&self) -> Result<(), String> {
        Err(Self::unsupported("home"))
    }

    async fn input_text(&self, _text: &str) -> Result<(), String> {
        Err(Self::unsupported("input_text"))
    }

    async fn change_cap_method(&self, method: CaptureMethod) -> bool {
        Log::warn(
            format!(
                "DesktopDeviceAdapter 尚未实现截图方式切换，忽略请求: {}",
                method
            )
            .as_str(),
        );
        false
    }
}
