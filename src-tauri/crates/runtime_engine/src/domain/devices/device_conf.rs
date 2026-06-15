use crate::infrastructure::core::{Deserialize, DeviceId, Serialize};
use crate::infrastructure::image::compression::ImageCompression;
use crate::infrastructure::logging::LogLevel;
use sqlx::types::Json;
use sqlx::FromRow;
use std::net::SocketAddrV4;
//
#[derive(Clone, Debug, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTable {
    // 设备ID
    pub id: DeviceId,
    // 设备配置（以 JSON 格式存储在数据库中）
    #[ts(as = "DeviceConfig")]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig {
    // 设备名称
    pub device_name: String,
    #[serde(default)]
    pub platform: DevicePlatform,
    #[serde(default)]
    pub transport_kind: DeviceTransportKind,
    #[serde(default)]
    pub emulator_connect_mode: EmulatorConnectMode,
    #[serde(default = "default_startup_delay_secs")]
    pub startup_delay_secs: u32,
    #[serde(default)]
    #[ts(as = "Option<String>")]
    pub connect_address: Option<SocketAddrV4>,
    #[serde(default)]
    pub connect_identifier: Option<String>,
    #[serde(default)]
    pub adb_path: Option<String>,
    #[serde(default = "default_adb_server_connect")]
    #[ts(as = "Option<String>")]
    pub adb_server_connect: Option<SocketAddrV4>,

    // 执行路径
    pub exe_path: Option<String>,
    // 执行参数
    pub exe_args: Option<String>,
    // 核心
    pub cores: Vec<u8>,
    // 日志级别
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    // 日志是否写入文件（禁用时仅输出到前端）
    #[serde(default = "default_log_to_file")]
    pub log_to_file: bool,

    // 截图方式
    pub cap_method: CapMethod,
    // 图像压缩方式
    pub image_compression: ImageCompression,
    // 是否启用
    pub enable: bool,
    // 启用时是否自动运行设备队列
    pub auto_start: bool,
    #[serde(default)]
    pub execution_policy: DeviceExecutionPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceExecutionPolicy {
    pub action_wait_ms: u32,
    pub progress_timeout_enabled: bool,
    pub progress_timeout_ms: u32,
    pub timeout_action: TimeoutAction,
    pub timeout_notify_channels: Vec<TimeoutNotifyChannel>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutAction {
    StopExecution,
    RunRecoveryTask,
    SkipCurrentTask,
}

impl<'de> serde::Deserialize<'de> for TimeoutAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "stopExecution" | "notifyOnly" | "pauseExecution" | "restartApp" => {
                Ok(Self::StopExecution)
            }
            "runRecoveryTask" => Ok(Self::RunRecoveryTask),
            "skipCurrentTask" => Ok(Self::SkipCurrentTask),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["stopExecution", "runRecoveryTask", "skipCurrentTask"],
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutNotifyChannel {
    SystemNotification,
    Email,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CapMethod {
    Window { title: String },
    Adb,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum DevicePlatform {
    Android,
    Desktop,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum DeviceTransportKind {
    EmulatorTcp,
    AdbUsb,
    AdbWireless,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum EmulatorConnectMode {
    TcpAddress,
    Identifier,
}

impl Default for DeviceTransportKind {
    fn default() -> Self {
        Self::EmulatorTcp
    }
}

impl Default for EmulatorConnectMode {
    fn default() -> Self {
        Self::TcpAddress
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_name: "MuMu模拟器12".into(),
            platform: DevicePlatform::default(),
            transport_kind: DeviceTransportKind::EmulatorTcp,
            emulator_connect_mode: EmulatorConnectMode::default(),
            startup_delay_secs: default_startup_delay_secs(),
            connect_address: None,
            connect_identifier: None,
            adb_path: None,
            adb_server_connect: default_adb_server_connect(),
            exe_path: None,
            exe_args: None,
            cores: vec![0, 1],
            log_level: LogLevel::Off,
            log_to_file: true,
            cap_method: CapMethod::Window {
                title: "AutoDaily".into(),
            },
            image_compression: ImageCompression::WindowOriginal,
            enable: false,
            auto_start: false,
            execution_policy: DeviceExecutionPolicy::default(),
        }
    }
}

impl Default for DevicePlatform {
    fn default() -> Self {
        Self::Android
    }
}

impl Default for DeviceExecutionPolicy {
    fn default() -> Self {
        Self {
            action_wait_ms: 500,
            progress_timeout_enabled: false,
            progress_timeout_ms: 30_000,
            timeout_action: TimeoutAction::StopExecution,
            timeout_notify_channels: Vec::new(),
        }
    }
}

fn default_startup_delay_secs() -> u32 {
    15
}

fn default_adb_server_connect() -> Option<SocketAddrV4> {
    "127.0.0.1:5037".parse().ok()
}

fn default_log_level() -> LogLevel {
    LogLevel::Off
}

fn default_log_to_file() -> bool {
    true
}

impl DeviceConfig {
    pub fn uses_emulator_transport(&self) -> bool {
        matches!(self.transport_kind, DeviceTransportKind::EmulatorTcp)
    }

    pub fn uses_emulator_identifier_connect(&self) -> bool {
        self.uses_emulator_transport()
            && matches!(self.emulator_connect_mode, EmulatorConnectMode::Identifier)
    }
}
