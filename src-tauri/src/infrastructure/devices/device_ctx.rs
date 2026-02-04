use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::capture::window_cap::WindowInfo;
use crate::domain::devices::device_conf::DeviceConfig;
use crate::infrastructure::logging::log_trait::Log;
use image::RgbaImage;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use crate::infrastructure::adb_cli_local::adb_context::get_adb_ctx;
use crate::infrastructure::context::init_error::{InitError, InitResult};

static DEVICE_CTX: OnceLock<Arc<DeviceCtx>> = OnceLock::new();

pub fn get_device_ctx() -> Arc<DeviceCtx> {
    DEVICE_CTX.get().expect("DeviceCtx not initialized").clone()
}

pub fn init_device_ctx(ctx: Arc<DeviceCtx>)-> InitResult<()> {
    DEVICE_CTX.set(ctx).map_err(|_| InitError::InitChildAppCtxFailed { e: "DeviceCtx already initialized".to_string() })?;
    Ok(())
}

pub struct DeviceCtx {
    //设备配置
    pub device_config: Arc<RwLock<DeviceConfig>>,

    //截图方式
    pub capture_method: Arc<AtomicU8>,

    //ADB上下文
    //pub adb_ctx: Option<ADBCtx>,

    //截图通道
    pub cap_tx: crossbeam_channel::Sender<RgbaImage>,
    pub cap_rx: crossbeam_channel::Receiver<RgbaImage>,

    // 窗口信息
    pub window_info: Arc<WindowInfo>,
}

impl DeviceCtx {
    pub fn new(
        device_config: Arc<RwLock<DeviceConfig>>,
        capture_method: CaptureMethod,
        window_title: Option<String>,
    ) -> DeviceCtx {
        Log::debug("初始化设备上下文数据...");
        let (tx, rx) = crossbeam_channel::bounded(1);
        DeviceCtx {
            device_config,
            capture_method: Arc::new(AtomicU8::new(capture_method as u8)),
            //adb_ctx,
            cap_tx: tx,
            cap_rx: rx,
            window_info: Arc::new(WindowInfo::init(window_title)),
        }
    }

    pub async fn valid_capture(&self) -> bool {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("验证窗口截图设置...");
                let win = self.window_info.window.read().await;
                if !win.is_some(){
                    Log::error("验证截图设置失败：未初始化窗口信息！");
                    return false;
                }
                true
            }
            2 => {
                Log::debug("验证adb截图设置...");
                get_adb_ctx().adb_executor.validate_config()
            }
            _ => {
                Log::error("不支持的截图设置！");
                false
            }
        }
    }

    pub async fn get_screenshot(&self) -> Option<RgbaImage> {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("窗口方式截图...");
                if let Some(win) = self.window_info.window.read().await.as_ref(){
                    match win.capture_image() {
                        Ok(img) => {
                            return Some(img);
                        },
                        Err(e) => {
                            Log::error(&format!("截图失败：{}", e.to_string()));
                        }
                    }
                } else {
                    Log::error("截图失败：未初始化目标窗口信息！");
                }
                None
            }
            2 => {
                Log::debug("ADB方式截图...");
                get_adb_ctx().send_adb_cmd(&ADBCommand::Capture(self.cap_tx.clone()));
                Log::debug("等待获取图像数据...");
                if let Ok(img) = self.cap_rx.recv() {
                    Some(img)
                } else {
                    Log::error("截图失败：截图数据接收错误！");
                    None
                }
            }
            _ => {
                Log::error("截图失败：不支持的截图方式！");
                None
            }
        }
    }

    pub async fn change_cap_method(&self, method: CaptureMethod)-> bool{
        Log::debug(format!("切换截图方式为：{}",method).as_str());
        self.capture_method.store(method as u8, Ordering::Release);
        self.valid_capture().await
    }
}
