use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
pub enum ImageCompression {
    WindowOriginal,
    AdbOriginal,
    ScreenCap,
}
