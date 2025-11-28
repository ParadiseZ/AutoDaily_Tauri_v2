use crate::infrastructure::core::{Deserialize, DeviceId, HashMap, Serialize};
use crate::infrastructure::devices::adb_info::{AdbConnectSatus, AdbInfo};
use crate::infrastructure::image::compression::ImageCompression;
use crate::infrastructure::logging::LogLevel;
use std::net::Ipv4Addr;
use std::sync::{Arc, RwLock};

pub type DeviceConfMap = HashMap<Arc<DeviceId>, Arc<RwLock<DeviceConfig>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    // 设备名称
    pub device_name: String,

    // 执行路径
    pub exe_path: Option<String>,
    // 执行参数
    pub exe_args: Option<String>,
    // 核心数量
    pub cores: u8,
    // 日志级别
    pub log_level: LogLevel,

    // 模拟器信息
    pub adb_info: Option<AdbInfo>,
    // 截图方式
    pub cap_method: CapMethod,
    // 图像压缩方式
    pub image_compression: ImageCompression,
    // 是否启用
    pub enable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CapMethod {
    Window(String),
    Adb(String),
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_name: "MuMu模拟器12".into(),
            exe_path: None,
            exe_args: None,
            cores: 4,
            log_level: LogLevel::Off,
            adb_info: Some(AdbInfo {
                ip_addr: Ipv4Addr::new(127, 0, 0, 1),
                port: 16416,
                states: AdbConnectSatus::Disconnect,
            }),
            cap_method: CapMethod::Window("AutoDaily".into()),
            image_compression: ImageCompression::WindowOriginal,
            enable: false,
        }
    }
}