use std::net::IpAddr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DevicesConfig{
    pub devices : Option<Vec<DeviceConfig>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfig{
    pub device_id : String,
    pub device_name : String,
    pub ip_addr: IpAddr,
    pub port : usize,
}