use std::net::SocketAddrV4;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug,Clone,Deserialize)]
struct AdbServerConfig{
    pub(crate) adb_path : Option<String>,
    pub(crate) server_connect : Option<SocketAddrV4>,
}

impl AdbServerConfig {
    pub fn valid(&self) -> bool {
        self.adb_path.is_some() && PathBuf::from(self.adb_path.as_ref().unwrap()).exists() && self.server_connect.is_some()
    }
}

#[derive(Debug,Clone,Deserialize)]
pub struct AdbServerConnectName{
    pub adb_config : AdbServerConfig,
    pub device_name: Option<String>,
}

impl AdbServerConnectName{
    pub fn valid(&self) -> bool{
        self.adb_config.valid() && self.device_name.is_some()
    }
}

#[derive(Debug,Clone,Deserialize)]
pub struct AdbServerConnectIp{
    pub(crate) adb_config : AdbServerConfig,
    pub(crate) client_connect : Option<SocketAddrV4>,
}

impl AdbServerConnectIp{
    pub fn valid(&self) -> bool{
        self.adb_config.valid() && self.client_connect.is_some()
    }
}

#[derive(Debug,Clone,Deserialize)]
pub struct DirectUsbConnect{
    vendor_id : u16,
    product_id: u16
}

#[derive(Debug,Clone,Deserialize)]
pub enum ADBConnectConfig{
    ServerConnectByName(AdbServerConnectName),
    ServerConnectByIp(AdbServerConnectIp),
    DirectTcp(Option<SocketAddrV4>),
    DirectUsb(DirectUsbConnect)
}