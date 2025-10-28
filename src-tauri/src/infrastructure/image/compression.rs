use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ImageCompression{
    WindowOriginal,
    AdbOriginal,
    ScreenCap
}