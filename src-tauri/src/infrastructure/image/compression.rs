use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum ImageCompression {
    WindowOriginal,
    AdbOriginal,
    ScreenCap,
}
