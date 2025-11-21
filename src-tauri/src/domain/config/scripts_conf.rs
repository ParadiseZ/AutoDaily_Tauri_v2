use crate::constant::sys_conf_path::SCRIPTS_CONFIG_PATH;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::config::conf_error::ConfigResult;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::config::conf_write_guard::ConfigCategory;
use crate::infrastructure::core::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScriptsConfig {
    pub dir: PathBuf,
}

impl ConfigCategory for ScriptsConfig {
    fn default() -> Self {
        Self {
            dir: get_app_handle()
                .path()
                .app_config_dir()
                .unwrap()
                .join("scripts"),
        }
    }
}

impl ScriptsConfig {
    pub async fn get_dir() -> PathBuf {
        ConfigManager::get::<ScriptsConfig>(SCRIPTS_CONFIG_PATH)
            .await
            .dir
    }

    pub async fn set_dir(dir: PathBuf) -> ConfigResult<()> {
        ConfigManager::get_mut::<ScriptsConfig>(SCRIPTS_CONFIG_PATH)
            .await
            .dir = dir;
        Ok(())
    }
}
