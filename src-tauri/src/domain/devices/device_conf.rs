use std::net::Ipv4Addr;
use crate::infrastructure::core::{Deserialize, DeviceId, Serialize};
use crate::infrastructure::devices::adb_info::{AdbConnectStatus, AdbInfo};
use crate::infrastructure::image::compression::ImageCompression;
use crate::infrastructure::logging::LogLevel;
use sqlx::types::Json;
use sqlx::FromRow;
// 
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTable {
    // 设备ID
    pub id: DeviceId,
    // 设备配置（以 JSON 格式存储在数据库中）
    pub data: Json<DeviceConfig>,
}

impl Default for DeviceTable {
    fn default() -> Self {
        Self {
            id: DeviceId::new_v7(),
            data: Json(DeviceConfig::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    // 设备名称
    pub device_name: String,

    // 执行路径
    pub exe_path: Option<String>,
    // 执行参数
    pub exe_args: Option<String>,
    // 核心
    pub cores: Vec<u8>,
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
pub enum  CapMethod {
    Window(String),
    ADB
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_name: "MuMu模拟器12".into(),
            exe_path: None,
            exe_args: None,
            cores: vec![0,1],
            log_level: LogLevel::Off,
            adb_info: Some(AdbInfo {
                ip_addr: Ipv4Addr::new(127, 0, 0, 1),
                port: 16416,
                states: AdbConnectStatus::Disconnect,
            }),
            cap_method: CapMethod::Window("AutoDaily".into()),
            image_compression: ImageCompression::WindowOriginal,
            enable: false,
        }
    }
}