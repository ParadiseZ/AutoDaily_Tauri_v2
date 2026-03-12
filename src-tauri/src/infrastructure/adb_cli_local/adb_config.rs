use std::net::SocketAddrV4;
use std::path::PathBuf;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AdbServerConfig {
    pub adb_path: Option<String>,
    #[ts(as = "Option<String>")]
    pub server_connect: Option<SocketAddrV4>,
}

impl AdbServerConfig {
    pub fn valid(&self) -> bool {
        self.adb_path.is_some()
            && PathBuf::from(self.adb_path.as_ref().unwrap()).exists()
            && self.server_connect.is_some()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AdbServerConnectName {
    pub adb_config: AdbServerConfig,
    pub device_name: Option<String>,
}

impl AdbServerConnectName {
    pub fn valid(&self) -> bool {
        self.adb_config.valid() && self.device_name.is_some()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AdbServerConnectIp {
    pub adb_config: AdbServerConfig,
    #[ts(as = "Option<String>")]
    pub client_connect: Option<SocketAddrV4>,
}

impl AdbServerConnectIp {
    pub fn valid(&self) -> bool {
        self.adb_config.valid() && self.client_connect.is_some()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DirectUsbConnect {
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ADBConnectConfig {
    ServerConnectByName(AdbServerConnectName),
    ServerConnectByIp(AdbServerConnectIp),
    DirectTcp(#[ts(as = "Option<String>")] Option<SocketAddrV4>),
    DirectUsb(DirectUsbConnect),
}

impl std::fmt::Display for ADBConnectConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBConnectConfig::ServerConnectByName(config) =>
                write!(f,
                       "ServerConnectByName-→ name:{},ip:{},adb_path:{}",
                       config.device_name.as_deref().unwrap_or(""),
                       config.adb_config.server_connect.map(|addr| addr.to_string())
                           .as_deref()
                           .unwrap_or(""),
                       config.adb_config.adb_path.as_deref().unwrap_or("")),
            ADBConnectConfig::ServerConnectByIp(config) =>
                write!(f, "ServerConnectByIp-→ device_ip:{},server_ip:{},adb_path:{}",
                    config.client_connect.map(|addr| addr.to_string())
                        .as_deref()
                        .unwrap_or(""),
                    config.adb_config.server_connect.map(|addr| addr.to_string())
                        .as_deref()
                        .unwrap_or(""),
                    config.adb_config.adb_path.as_deref().unwrap_or("")),
            ADBConnectConfig::DirectTcp(config) => write!(f, "DirectTcp--> {}", config.map(|addr| addr.to_string()).as_deref().unwrap_or("")),
            ADBConnectConfig::DirectUsb(config) => write!(f, "DirectUsb: {}:{}", config.vendor_id, config.product_id),
        }
    }
}


impl ADBConnectConfig {
    pub fn valid(&self) -> bool {
        match self {
            ADBConnectConfig::ServerConnectByName(config) => config.valid(),
            ADBConnectConfig::ServerConnectByIp(config) => config.valid(),
            ADBConnectConfig::DirectTcp(config) => config.is_some(),
            ADBConnectConfig::DirectUsb(_) => false,
        }
    }

    /// 更新 adb_path（仅对 Server 类型有效）
    pub fn update_adb_path(&mut self, path: Option<String>) {
        match self {
            ADBConnectConfig::ServerConnectByName(config) => {
                config.adb_config.adb_path = path;
            }
            ADBConnectConfig::ServerConnectByIp(config) => {
                config.adb_config.adb_path = path;
            }
            _ => {
                // DirectTcp / DirectUsb 不使用 adb_path
            }
        }
    }

    /// 更新 server_connect 地址（仅对 Server 类型有效）
    pub fn update_server_addr(&mut self, addr: Option<String>) {
        use std::net::SocketAddrV4;
        let parsed: Option<SocketAddrV4> = addr
            .as_deref()
            .and_then(|s| s.parse::<SocketAddrV4>().ok());
        match self {
            ADBConnectConfig::ServerConnectByName(config) => {
                config.adb_config.server_connect = parsed;
            }
            ADBConnectConfig::ServerConnectByIp(config) => {
                config.adb_config.server_connect = parsed;
            }
            _ => {
                // DirectTcp / DirectUsb 不使用 server_connect
            }
        }
    }
}

