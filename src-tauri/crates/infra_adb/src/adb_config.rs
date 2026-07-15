//! ADB connection configuration.
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use std::path::PathBuf;

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
pub struct AdbServeByIdentifier {
    pub adb_config: AdbServerConfig,
    pub identifier: Option<String>,
}

impl AdbServeByIdentifier {
    pub fn valid(&self) -> bool {
        self.adb_config.valid()
            && self
                .identifier
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ADBConnectConfig {
    DirectTcp(#[ts(as = "Option<String>")] Option<SocketAddrV4>),
    ServeByIdentifier(AdbServeByIdentifier),
}

impl std::fmt::Display for ADBConnectConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBConnectConfig::DirectTcp(config) => write!(
                f,
                "DirectTcp--> {}",
                config.map(|addr| addr.to_string()).as_deref().unwrap_or("")
            ),
            ADBConnectConfig::ServeByIdentifier(config) => write!(
                f,
                "ServeByIdentifier--> identifier:{},server_ip:{},adb_path:{}",
                config.identifier.as_deref().unwrap_or(""),
                config
                    .adb_config
                    .server_connect
                    .map(|addr| addr.to_string())
                    .as_deref()
                    .unwrap_or(""),
                config.adb_config.adb_path.as_deref().unwrap_or("")
            ),
        }
    }
}

impl ADBConnectConfig {
    pub fn valid(&self) -> bool {
        match self {
            ADBConnectConfig::DirectTcp(config) => config.is_some(),
            ADBConnectConfig::ServeByIdentifier(config) => config.valid(),
        }
    }
}
