use crate::infrastructure::config::conf_write_guard::ConfigCategory;
use crate::infrastructure::core::{Deserialize, Serialize};

/// 热键/快捷键配置
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShortCut {
    pub toggle_window: String,
    pub toggle_all_scripts: String,
    pub capture: String,
}

impl ConfigCategory for ShortCut {
    fn default() -> Self {
        Self {
            toggle_window: "CommandOrControl+H".into(),
            toggle_all_scripts: "Alt+R".into(),
            capture: "Alt+A".into(),
        }
    }
}
