use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::core::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScriptsConfig {
    pub dir: PathBuf,
}

impl Default for ScriptsConfig {
    fn default() -> Self {
        Self {
            dir: get_app_handle()
                .path()
                .app_data_dir()
                .unwrap()
                .join("scripts"),
        }
    }
}