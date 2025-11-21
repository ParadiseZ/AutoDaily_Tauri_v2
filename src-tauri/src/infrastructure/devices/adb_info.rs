use crate::infrastructure::core::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AdbConnectSatus {
    Connecting,
    Connected,
    Disconnect,
    Executing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbInfo {
    pub ip_addr: Ipv4Addr,
    pub port: usize,
    pub states: AdbConnectSatus,
}
