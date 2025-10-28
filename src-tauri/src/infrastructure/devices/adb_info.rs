use std::net::{IpAddr, Ipv4Addr};
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AdbConnectSatus{
    Connecting,
    Connected,
    Disconnect,
    Executing
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbInfo{
    pub ip_addr: Ipv4Addr,
    pub port : usize,
    pub states : AdbConnectSatus
}