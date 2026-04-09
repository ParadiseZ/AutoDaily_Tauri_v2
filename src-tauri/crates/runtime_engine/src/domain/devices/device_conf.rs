use crate::infrastructure::core::{Deserialize, DeviceId, Serialize};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::image::compression::ImageCompression;
use crate::infrastructure::logging::LogLevel;
use sqlx::types::Json;
use sqlx::FromRow;
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

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    // 日志是否写入文件（禁用时仅输出到前端）
    pub log_to_file: bool,

    // ADB 连接配置（adb_path 运行时从全局设置注入，不存储在此）
    pub adb_connect: Option<ADBConnectConfig>,
    // 截图方式
    pub cap_method: CapMethod,
    // 图像压缩方式
    pub image_compression: ImageCompression,
    // 是否启用
    pub enable: bool,
    // 启用时是否自动启动（启动设备+连接+调度脚本）
    pub auto_start: bool,
    #[serde(default)]
    pub execution_policy: DeviceExecutionPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceExecutionPolicy {
    pub action_wait_ms: u64,
    pub progress_timeout_enabled: bool,
    pub progress_timeout_ms: u64,
    pub timeout_action: TimeoutAction,
    pub timeout_notify_channels: Vec<TimeoutNotifyChannel>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutAction {
    NotifyOnly,
    PauseExecution,
    StopExecution,
    RestartApp,
    RunRecoveryTask,
    SkipCurrentTask,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TimeoutNotifyChannel {
    SystemNotification,
    Email,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum  CapMethod {
    Window(String),
    Adb
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device_name: "MuMu模拟器12".into(),
            exe_path: None,
            exe_args: None,
            cores: vec![0,1],
            log_level: LogLevel::Off,
            log_to_file: true,
            adb_connect: None,
            cap_method: CapMethod::Window("AutoDaily".into()),
            image_compression: ImageCompression::WindowOriginal,
            enable: false,
            auto_start: false,
            execution_policy: DeviceExecutionPolicy::default(),
        }
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
