use crate::domain::services::capture_basis::CaptureHandler;
use crate::domain::trait_ad::config_category::ConfigCategory;
use crate::infrastructure::entities::devices::capture_basis::{CaptureBasis, ImageCompression};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};


pub type DeviceId = Uuid;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum  DeviceType{
    UsbPhone,
    WifiPhoneLan,
    WifiPhoneRemote,
    EmulatorLocal,
    EmulatorLan,
    EmulatorRemote,
    Window
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DeviceInfo{
    /// 设备唯一标识
    pub device_id: DeviceId,
    pub device_name : String,
    pub device_type : DeviceType,
    pub capture_basis : CaptureBasis,
    pub capture_handler : Option<dyn CaptureHandler>,
    pub image_compression: ImageCompression,
}

impl ConfigCategory for DeviceInfo{
    fn default() -> Self {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let ts = Timestamp::from_unix(NoContext, duration.as_secs(), duration.subsec_nanos());
        Self{
            device_id: Uuid::new_v7(ts),
            device_name : "".into(),
            device_type : DeviceType::EmulatorLocal,
            capture_basis : CaptureBasis::Window,
            capture_handler : None,
            image_compression: ImageCompression::WindowOriginal,
        }
    }
}