use serde::{Deserialize, Serialize};

pub const SCRIPT_RUNTIME_SCHEMA: u32 = 1;

pub fn supported_script_features() -> Vec<String> {
    ["onnxInference", "runtime:rhai", "device:android"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptType {
    /// 本地开发模式。
    Dev,
    /// 已发布或云端模式。
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeType {
    Rhai,
    JavaScript,
    Lua,
    AIAndVision,
    AI,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptPlatform {
    Android,
    Desktop,
}

impl Default for ScriptPlatform {
    fn default() -> Self {
        Self::Android
    }
}

#[cfg(test)]
mod tests {
    use super::{ScriptPlatform, supported_script_features};

    #[test]
    fn defaults_to_android_with_the_current_runtime_capabilities() {
        assert_eq!(ScriptPlatform::default(), ScriptPlatform::Android);
        assert_eq!(
            supported_script_features(),
            vec!["onnxInference", "runtime:rhai", "device:android"]
        );
    }
}
