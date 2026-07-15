use crate::ScriptInfo;
use ad_kernel::ids::ScriptId;

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptProfile {
    pub id: ScriptId,
    pub info: ScriptInfo,
}

impl Default for ScriptProfile {
    fn default() -> Self {
        Self {
            id: ScriptId::new_v7(),
            info: ScriptInfo::default(),
        }
    }
}
