use crate::Log;
use domain_device::{DeviceOperation, DevicePlatform};
use image::RgbaImage;
use infra_adb::{ADBCommand, try_get_adb_ctx};
use infra_window_capture::{CaptureMethod, WindowCaptureConfig, WindowCaptureOffsets, WindowInfo};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

#[derive(Clone)]
pub(crate) enum DeviceRuntime {
    Android(AndroidDeviceRuntime),
    Desktop(DesktopDeviceRuntime),
}

impl DeviceRuntime {
    pub(crate) async fn valid_capture(&self) -> bool {
        match self {
            Self::Android(runtime) => runtime.valid_capture().await,
            Self::Desktop(runtime) => runtime.valid_capture().await,
        }
    }

    pub(crate) async fn capture_screen_result(&self) -> Result<RgbaImage, String> {
        match self {
            Self::Android(runtime) => runtime.capture_screen_result().await,
            Self::Desktop(runtime) => runtime.capture_screen_result().await,
        }
    }

    pub(crate) async fn execute_operation(&self, operation: DeviceOperation) -> Result<(), String> {
        match self {
            Self::Android(runtime) => runtime.execute_operation(operation).await,
            Self::Desktop(_) => Err(Self::unsupported_platform_message(
                "Android 操作执行",
                DevicePlatform::Desktop,
            )),
        }
    }

    pub(crate) async fn execute_operations(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        match self {
            Self::Android(runtime) => runtime.execute_operations(operations).await,
            Self::Desktop(_) => Err(Self::unsupported_platform_message(
                "Android 动作序列执行",
                DevicePlatform::Desktop,
            )),
        }
    }

    pub(crate) async fn execute_sequence(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        match self {
            Self::Android(runtime) => runtime.execute_sequence(operations).await,
            Self::Desktop(_) => Err(Self::unsupported_platform_message(
                "Android Sequence 执行",
                DevicePlatform::Desktop,
            )),
        }
    }

    fn unsupported_platform_message(action: &str, platform: DevicePlatform) -> String {
        format!("当前平台 {:?} 尚未实现 {}", platform, action)
    }
}

#[derive(Clone)]
pub(crate) struct AndroidDeviceRuntime {
    capture_method: Arc<AtomicU8>,
    cap_tx: crossbeam_channel::Sender<Result<RgbaImage, String>>,
    cap_rx: crossbeam_channel::Receiver<Result<RgbaImage, String>>,
    window_info: Arc<WindowInfo>,
}

impl AndroidDeviceRuntime {
    pub(crate) fn new(
        capture_method: CaptureMethod,
        window_capture_config: Option<WindowCaptureConfig>,
    ) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(1);
        Self {
            capture_method: Arc::new(AtomicU8::new(capture_method as u8)),
            cap_tx: tx,
            cap_rx: rx,
            window_info: Arc::new(WindowInfo::init(window_capture_config.unwrap_or(
                WindowCaptureConfig {
                    title: String::new(),
                    interface: infra_window_capture::WindowCaptureInterface::Gdi,
                    frame_timeout: Duration::from_secs(10),
                    offsets: WindowCaptureOffsets {
                        left: 1,
                        top: 40,
                        right: 1,
                        bottom: 1,
                    },
                },
            ))),
        }
    }

    fn send_command(&self, command: ADBCommand) -> Result<(), String> {
        let adb_ctx = try_get_adb_ctx()?;
        adb_ctx.send_adb_cmd(&command)
    }

    async fn send_await_result_command(&self, command: ADBCommand) -> Result<(), String> {
        let adb_ctx = try_get_adb_ctx()?;
        adb_ctx.send_adb_cmd_await(command).await
    }

    async fn send_reliable_sequence(&self, commands: Vec<ADBCommand>) -> Result<(), String> {
        if commands.is_empty() {
            return Ok(());
        }

        let adb_ctx = try_get_adb_ctx()?;
        adb_ctx
            .send_adb_cmd_await_timeout(
                ADBCommand::ReliableSequence(commands.clone()),
                Self::estimate_reliable_sequence_timeout_ms(&commands),
            )
            .await
    }

    async fn send_sequence(&self, commands: Vec<ADBCommand>) -> Result<(), String> {
        if commands.is_empty() {
            return Ok(());
        }

        let adb_ctx = try_get_adb_ctx()?;
        adb_ctx
            .send_adb_cmd_await_timeout(
                ADBCommand::Sequence(commands.clone()),
                Self::estimate_reliable_sequence_timeout_ms(&commands),
            )
            .await
    }

    async fn capture_screen_via_window_result(&self) -> Result<RgbaImage, String> {
        self.window_info.capture_image_result().await
    }

    async fn capture_screen_via_adb_result(&self) -> Result<RgbaImage, String> {
        Log::debug("ADB方式截图...");
        let adb_ctx = match try_get_adb_ctx() {
            Ok(adb_ctx) => adb_ctx,
            Err(error) => {
                return Err(format!("截图失败：{}", error));
            }
        };
        if let Err(error) = adb_ctx.send_capture_cmd(&ADBCommand::Capture(self.cap_tx.clone())) {
            return Err(format!("截图失败：{}", error));
        }
        Log::debug("等待获取图像数据...");
        let cap_rx = self.cap_rx.clone();
        match tokio::task::spawn_blocking(move || cap_rx.recv_timeout(Duration::from_secs(10)))
            .await
        {
            Ok(Ok(Ok(img))) => Ok(img),
            Ok(Ok(Err(error))) => Err(format!("截图失败：{}", error)),
            Ok(Err(crossbeam_channel::RecvTimeoutError::Timeout)) => {
                Err("截图失败：等待截图结果已超过10秒！".to_string())
            }
            Ok(Err(crossbeam_channel::RecvTimeoutError::Disconnected)) => {
                Err("截图失败：截图数据通道已关闭！".to_string())
            }
            Err(error) => Err(format!("截图失败：等待截图任务异常：{}", error)),
        }
    }

    fn estimate_reliable_sequence_timeout_ms(commands: &[ADBCommand]) -> u64 {
        let duration_sum_ms: u64 = commands
            .iter()
            .filter_map(|command| match command {
                ADBCommand::Duration(ms) => Some(*ms),
                _ => None,
            })
            .sum();

        let command_overhead_ms = commands
            .iter()
            .filter(|command| !matches!(command, ADBCommand::Duration(_)))
            .count() as u64
            * 1_000;

        5_000 + duration_sum_ms + command_overhead_ms
    }

    fn to_sequence_command(operation: &DeviceOperation) -> Result<ADBCommand, String> {
        match operation {
            DeviceOperation::Click(point) => Ok(ADBCommand::Click(*point)),
            DeviceOperation::LongClick(point) => Ok(ADBCommand::LongClick(*point)),
            DeviceOperation::Swipe { from, to, duration } => {
                Ok(ADBCommand::SwipeWithDuration(*from, *to, *duration))
            }
            DeviceOperation::LaunchApp {
                pkg_name,
                activity_name,
            } => Ok(ADBCommand::StartActivity(
                pkg_name.clone(),
                activity_name.clone(),
            )),
            DeviceOperation::StopApp { pkg_name } => Ok(ADBCommand::StopApp(pkg_name.clone())),
            DeviceOperation::InputText(text) => Ok(ADBCommand::InputText(text.clone())),
            DeviceOperation::Back => Ok(ADBCommand::Back),
            DeviceOperation::Home => Ok(ADBCommand::Home),
            DeviceOperation::Delay(ms) => Ok(ADBCommand::Duration(*ms)),
            DeviceOperation::Reboot => {
                Err("设备重启依赖独立 ADB 指令通道，不能合并进 ADBCommand::Sequence".to_string())
            }
        }
    }

    async fn execute_batchable_operations(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        let commands = operations
            .iter()
            .map(Self::to_sequence_command)
            .collect::<Result<Vec<_>, _>>()?;
        self.send_reliable_sequence(commands).await
    }

    pub(crate) async fn valid_capture(&self) -> bool {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("验证窗口截图设置...");
                if !self.window_info.valid_capture().await {
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

    pub(crate) async fn capture_screen_result(&self) -> Result<RgbaImage, String> {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("窗口方式截图...");
                self.capture_screen_via_window_result().await
            }
            2 => self.capture_screen_via_adb_result().await,
            _ => Err("截图失败：不支持的截图方式！".to_string()),
        }
    }

    pub(crate) async fn execute_operation(&self, operation: DeviceOperation) -> Result<(), String> {
        match operation {
            DeviceOperation::Click(point) => self.send_command(ADBCommand::Click(point)),
            DeviceOperation::LongClick(point) => self.send_command(ADBCommand::LongClick(point)),
            DeviceOperation::Swipe { from, to, duration } => {
                self.send_command(ADBCommand::SwipeWithDuration(from, to, duration))
            }
            DeviceOperation::LaunchApp {
                pkg_name,
                activity_name,
            } => {
                self.send_await_result_command(ADBCommand::StartActivity(pkg_name, activity_name))
                    .await
            }
            DeviceOperation::StopApp { pkg_name } => {
                self.send_await_result_command(ADBCommand::StopApp(pkg_name))
                    .await
            }
            DeviceOperation::InputText(text) => self.send_command(ADBCommand::InputText(text)),
            DeviceOperation::Back => self.send_command(ADBCommand::Back),
            DeviceOperation::Home => self.send_command(ADBCommand::Home),
            DeviceOperation::Reboot => self.send_await_result_command(ADBCommand::Reboot).await,
            DeviceOperation::Delay(ms) => {
                tokio::time::sleep(Duration::from_millis(ms)).await;
                Ok(())
            }
        }
    }

    pub(crate) async fn execute_operations(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        if operations.is_empty() {
            return Ok(());
        }

        if operations
            .iter()
            .all(|operation| Self::to_sequence_command(operation).is_ok())
        {
            return self.execute_batchable_operations(operations).await;
        }

        for operation in operations {
            self.execute_operation(operation.clone()).await?;
        }
        Ok(())
    }

    pub(crate) async fn execute_sequence(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        if operations.is_empty() {
            return Ok(());
        }

        let commands = operations
            .iter()
            .map(Self::to_sequence_command)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                format!(
                    "Sequence 中存在无法转换为 ADBCommand::Sequence 的设备动作: {}",
                    error
                )
            })?;
        self.send_sequence(commands).await
    }
}

#[derive(Clone)]
pub(crate) struct DesktopDeviceRuntime;

impl DesktopDeviceRuntime {
    pub(crate) fn new() -> Self {
        Self
    }

    fn unsupported(action: &str) -> String {
        format!("DesktopDeviceRuntime 尚未实现: {}", action)
    }

    pub(crate) async fn valid_capture(&self) -> bool {
        Log::warn("DesktopDeviceRuntime 尚未实现截图校验");
        false
    }

    pub(crate) async fn capture_screen_result(&self) -> Result<RgbaImage, String> {
        Err("DesktopDeviceRuntime 尚未实现截图".to_string())
    }

    #[allow(dead_code)]
    pub(crate) async fn unsupported_action(&self, action: &str) -> Result<(), String> {
        Err(Self::unsupported(action))
    }
}
