use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::adb_cli_local::adb_context::ADBCtx;
use crate::infrastructure::capture::capture_method::CaptureMethod;
use crate::infrastructure::capture::window_cap::WindowInfo;
use crate::infrastructure::devices::device_conf::DeviceConfig;
use crate::infrastructure::logging::log_trait::Log;
use image::RgbaImage;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

pub struct DeviceCtx {
    //设备配置
    pub device_config: Arc<RwLock<DeviceConfig>>,

    //截图方式
    pub capture_method: Arc<AtomicU8>,

    //ADB上下文
    pub adb_ctx: Arc<RwLock<ADBCtx>>,

    //截图通道
    pub cap_tx: crossbeam_channel::Sender<RgbaImage>,
    pub cap_rx: crossbeam_channel::Receiver<RgbaImage>,

    // 窗口信息
    pub window_info: Arc<RwLock<Option<WindowInfo>>>,
}

impl DeviceCtx {
    pub fn new(
        device_config: Arc<RwLock<DeviceConfig>>,
        capture_method: CaptureMethod,
        adb_ctx: Arc<RwLock<ADBCtx>>,
    ) -> DeviceCtx {
        Log::debug("初始化设备上下文数据...");
        let (tx, rx) = crossbeam_channel::bounded(1);
        DeviceCtx {
            device_config,
            capture_method: Arc::new(AtomicU8::new(capture_method as u8)),
            adb_ctx,
            cap_tx: tx,
            cap_rx: rx,
            window_info: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn valid_capture(&self) -> bool {
        match self.capture_method.load(Ordering::Acquire) {
            1 => {
                Log::debug("验证窗口截图设置...");
                if let Ok(win) = self.window_info.read() {
                    win.is_some()
                } else {
                    Log::error("验证截图设置失败：未初始化窗口信息！");
                    false
                }
            }
            2 => {
                Log::debug("验证adb截图设置...");
                if let Ok(executor) = self.adb_ctx.clone().read() {
                    executor.adb_executor.validate_config()
                } else {
                    Log::error("验证截图设置失败：adb上下文未初始化！");
                    false
                }
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
                if let Ok(win) = self.window_info.read() {
                    if win.is_some() && win.unwrap().window.is_some() {
                        let img_get = win
                            .unwrap()
                            .window
                            .as_ref()
                            .map(|window| window.capture_image());
                        if let Some(img) = img_get {
                            if let Err(e) = img {
                                Log::error(&format!("截图失败：{}", e.to_string()))
                            } else {
                                return img.ok();
                            }
                        }
                    } else {
                        Log::error("截图失败：未初始化目标窗口信息！")
                    }
                } else {
                    Log::error("截图失败：获取窗口数据锁失败！")
                }
                None
            }
            2 => {
                Log::debug("ADB方式截图...");
                if let Ok(adb_ctx) = self.adb_ctx.clone().read() {
                    adb_ctx.send_adb_cmd(&ADBCommand::Capture(self.cap_tx.clone()));
                    Log::debug("等待获取图像数据...");
                    if let Ok(img) = self.cap_rx.recv() {
                        Some(img)
                    } else {
                        Log::error("截图失败：截图数据接收错误！");
                        None
                    }
                } else {
                    Log::error("截图失败：adb命令执行器错误！");
                    None
                }
            }
            _ => {
                Log::error("截图失败：不支持的截图方式！");
                None
            }
        }
    }
}
