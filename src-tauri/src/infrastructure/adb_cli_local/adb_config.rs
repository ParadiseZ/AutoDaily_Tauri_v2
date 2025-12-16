use std::net::SocketAddrV4;
use std::path::PathBuf;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdbServerConfig {
    pub(crate) adb_path: Option<String>,
    pub(crate) server_connect: Option<SocketAddrV4>,
}

impl AdbServerConfig {
    pub fn valid(&self) -> bool {
        self.adb_path.is_some()
            && PathBuf::from(self.adb_path.as_ref().unwrap()).exists()
            && self.server_connect.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdbServerConnectName {
    pub adb_config: AdbServerConfig,
    pub device_name: Option<String>,
}

impl AdbServerConnectName {
    pub fn valid(&self) -> bool {
        self.adb_config.valid() && self.device_name.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdbServerConnectIp {
    pub(crate) adb_config: AdbServerConfig,
    pub(crate) client_connect: Option<SocketAddrV4>,
}

impl AdbServerConnectIp {
    pub fn valid(&self) -> bool {
        self.adb_config.valid() && self.client_connect.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectUsbConnect {
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ADBConnectConfig {
    ServerConnectByName(AdbServerConnectName),
    ServerConnectByIp(AdbServerConnectIp),
    DirectTcp(Option<SocketAddrV4>),
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
}
