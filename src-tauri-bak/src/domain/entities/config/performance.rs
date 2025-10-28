use crate::domain::trait_ad::config_category::ConfigCategory;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Performance {
    pub cores_per_device: u32,
    pub max_devices: usize,
}

impl ConfigCategory for Performance{
    fn default() -> Self {
        Self{
            cores_per_device: 4,
            max_devices :1
        }
    }
}