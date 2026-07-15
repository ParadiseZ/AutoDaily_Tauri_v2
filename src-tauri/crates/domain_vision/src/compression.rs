use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
pub enum ImageCompression {
    WindowOriginal,
    AdbOriginal,
    ScreenCap,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_image_compression() {
        assert_eq!(
            serde_json::to_string(&ImageCompression::WindowOriginal).unwrap(),
            "\"WindowOriginal\""
        );
    }
}
