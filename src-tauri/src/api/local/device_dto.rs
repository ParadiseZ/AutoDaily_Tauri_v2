use ad_kernel::ids::DeviceId;
use domain_device::{DeviceConfig, DeviceProfile};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTable {
    pub id: DeviceId,
    pub data: DeviceConfig,
}

impl From<DeviceProfile> for DeviceTable {
    fn from(device: DeviceProfile) -> Self {
        Self {
            id: device.id,
            data: device.config,
        }
    }
}

impl From<DeviceTable> for DeviceProfile {
    fn from(device: DeviceTable) -> Self {
        Self {
            id: device.id,
            config: device.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_table_serializes_the_frontend_data_field() {
        let value = serde_json::to_value(DeviceTable::from(DeviceProfile::default())).unwrap();
        assert!(value.get("data").is_some());
        assert!(value.get("config").is_none());
    }
}
