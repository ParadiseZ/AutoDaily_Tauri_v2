use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    // 现有消息类型...
    #[serde(rename = "adb_command")]
    AdbCommand {
        device_id: String,
        command: String,
    },
    #[serde(rename = "adb_screenshot")]
    AdbScreenshot {
        device_id: String,
    },
    #[serde(rename = "adb_devices")]
    AdbDevices,
    #[serde(rename = "adb_status")]
    AdbStatus(bool),
}
