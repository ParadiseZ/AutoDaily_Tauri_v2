use crate::domain::devices::device_conf::{DeviceConfig, DevicePlatform};
use crate::domain::scripts::point::Point;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::devices::device_adapter::{
    AndroidDeviceAdapter, DesktopDeviceAdapter, DeviceAdapter,
};
use crate::infrastructure::logging::log_trait::Log;
use image::RgbaImage;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

static DEVICE_CTX: OnceLock<Arc<DeviceCtx>> = OnceLock::new();

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
    pub adapter: Arc<dyn DeviceAdapter>,
}

impl DeviceCtx {
    pub fn new(
        device_config: Arc<RwLock<DeviceConfig>>,
        capture_method: CaptureMethod,
        window_title: Option<String>,
    ) -> DeviceCtx {
        Log::debug("初始化设备上下文数据...");
        let platform = device_config.blocking_read().platform.clone();
        let adapter: Arc<dyn DeviceAdapter> = match platform {
            DevicePlatform::Android => {
                Arc::new(AndroidDeviceAdapter::new(capture_method, window_title))
            }
            DevicePlatform::Desktop => Arc::new(DesktopDeviceAdapter::new()),
        };
        DeviceCtx {
            device_config,
            adapter,
        }
    }

    pub async fn valid_capture(&self) -> bool {
        self.adapter.valid_capture().await
    }

    pub async fn get_screenshot(&self) -> Option<RgbaImage> {
        self.adapter.capture_screen().await
    }

    pub async fn change_cap_method(&self, method: CaptureMethod) -> bool {
        self.adapter.change_cap_method(method).await
    }

    pub async fn click(&self, point: Point<u16>) -> Result<(), String> {
        self.adapter.click(point).await
    }

    pub async fn swipe(
        &self,
        from: Point<u16>,
        to: Point<u16>,
        duration: u64,
    ) -> Result<(), String> {
        self.adapter.swipe(from, to, duration).await
    }

    pub async fn reboot(&self) -> Result<(), String> {
        self.adapter.reboot().await
    }

    pub async fn launch_app(&self, pkg_name: &str, activity_name: &str) -> Result<(), String> {
        self.adapter.launch_app(pkg_name, activity_name).await
    }

    pub async fn stop_app(&self, pkg_name: &str) -> Result<(), String> {
        self.adapter.stop_app(pkg_name).await
    }
}
