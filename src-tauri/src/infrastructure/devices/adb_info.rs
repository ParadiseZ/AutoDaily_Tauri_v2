use crate::infrastructure::core::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum AdbConnectStatus {
    Connecting,
    Connected,
    Disconnect,
    Executing,
}

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AdbInfo {
    #[ts(as = "String")]
    pub ip_addr: Ipv4Addr,
    pub port: u16,
    pub states: AdbConnectStatus,
}
