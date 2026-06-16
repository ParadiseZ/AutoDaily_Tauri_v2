use crate::domain::devices::device_conf::{DeviceConfig, DevicePlatform};
use crate::domain::scripts::point::Point;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::devices::device_runtime::{
    AndroidDeviceRuntime, DesktopDeviceRuntime, DeviceOperation, DeviceRuntime,
};
use crate::infrastructure::logging::log_trait::Log;
use image::RgbaImage;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

static DEVICE_CTX: OnceLock<Arc<DeviceCtx>> = OnceLock::new();

pub fn try_get_device_ctx() -> Option<Arc<DeviceCtx>> {
    DEVICE_CTX.get().cloned()
}

pub fn get_device_ctx() -> Arc<DeviceCtx> {
    DEVICE_CTX.get().expect("DeviceCtx not initialized").clone()
}

pub fn init_device_ctx(ctx: Arc<DeviceCtx>) -> InitResult<()> {
    DEVICE_CTX
        .set(ctx)
        .map_err(|_| InitError::InitChildAppCtxFailed {
            e: "DeviceCtx already initialized".to_string(),
        })?;
    Ok(())
}

pub struct DeviceCtx {
    //设备配置
    pub device_config: Arc<RwLock<DeviceConfig>>,
    pub runtime: Arc<RwLock<DeviceRuntime>>,
}

impl DeviceCtx {
    fn build_runtime(config: &DeviceConfig) -> DeviceRuntime {
        let (capture_method, window_title) = match &config.cap_method {
            crate::domain::devices::device_conf::CapMethod::Window { title }
                if config.supports_window_capture() =>
            {
                (CaptureMethod::Window, Some(title.clone()))
            }
            crate::domain::devices::device_conf::CapMethod::Window { .. } => {
                Log::warn("[ DeviceCtx ] 当前设备通道不支持窗口截图，运行时回退为 ADB 截图");
                (CaptureMethod::Adb, None)
            }
            crate::domain::devices::device_conf::CapMethod::Adb => (CaptureMethod::Adb, None),
        };
        match config.platform {
            DevicePlatform::Android => {
                DeviceRuntime::Android(AndroidDeviceRuntime::new(capture_method, window_title))
            }
            DevicePlatform::Desktop => DeviceRuntime::Desktop(DesktopDeviceRuntime::new()),
        }
    }

    pub async fn new(
        device_config: Arc<RwLock<DeviceConfig>>,
        _capture_method: CaptureMethod,
        _window_title: Option<String>,
    ) -> DeviceCtx {
        Log::debug("初始化设备上下文数据...");
        let config = device_config.read().await.clone();
        let runtime = Self::build_runtime(&config);
        DeviceCtx {
            device_config,
            runtime: Arc::new(RwLock::new(runtime)),
        }
    }

    pub async fn valid_capture(&self) -> bool {
        let runtime = self.runtime.read().await.clone();
        runtime.valid_capture().await
    }

    pub async fn get_screenshot(&self) -> Option<RgbaImage> {
        let runtime = self.runtime.read().await.clone();
        runtime.capture_screen().await
    }

    pub async fn change_cap_method(&self, method: CaptureMethod) -> bool {
        let runtime = self.runtime.read().await.clone();
        runtime.change_cap_method(method).await
    }

    pub async fn apply_device_config(&self, next_config: DeviceConfig) {
        let runtime = Self::build_runtime(&next_config);
        *self.device_config.write().await = next_config;
        *self.runtime.write().await = runtime;
    }

    pub async fn execute_operations(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        let runtime = self.runtime.read().await.clone();
        runtime.execute_operations(operations).await
    }

    pub async fn execute_operation(
        &self,
        operation: DeviceOperation,
    ) -> Result<(), String> {
        let runtime = self.runtime.read().await.clone();
        runtime.execute_operation(operation).await
    }

    pub async fn execute_sequence(&self, operations: &[DeviceOperation]) -> Result<(), String> {
        let runtime = self.runtime.read().await.clone();
        runtime.execute_sequence(operations).await
    }

    pub async fn click(&self, point: Point<u16>) -> Result<(), String> {
        self.execute_operation(DeviceOperation::Click(point))
            .await
    }

    pub async fn long_click(&self, point: Point<u16>) -> Result<(), String> {
        self.execute_operation(DeviceOperation::LongClick(point))
            .await
    }

    pub async fn swipe(
        &self,
        from: Point<u16>,
        to: Point<u16>,
        duration: u64,
    ) -> Result<(), String> {
        self.execute_operation(DeviceOperation::Swipe { from, to, duration })
            .await
    }

    pub async fn reboot(&self) -> Result<(), String> {
        self.execute_operation(DeviceOperation::Reboot).await
    }

    pub async fn launch_app(&self, pkg_name: &str, activity_name: &str) -> Result<(), String> {
        self.execute_operation(DeviceOperation::LaunchApp {
            pkg_name: pkg_name.to_string(),
            activity_name: activity_name.to_string(),
        })
        .await
    }

    pub async fn stop_app(&self, pkg_name: &str) -> Result<(), String> {
        self.execute_operation(DeviceOperation::StopApp {
            pkg_name: pkg_name.to_string(),
        })
        .await
    }

    pub async fn back(&self) -> Result<(), String> {
        self.execute_operation(DeviceOperation::Back).await
    }

    pub async fn home(&self) -> Result<(), String> {
        self.execute_operation(DeviceOperation::Home).await
    }

    pub async fn input_text(&self, text: &str) -> Result<(), String> {
        self.execute_operation(DeviceOperation::InputText(text.to_string()))
            .await
    }
}
