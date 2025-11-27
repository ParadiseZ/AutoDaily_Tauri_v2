use crate::constant::sys_conf_path::SCRIPTS_CONFIG_PATH;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::config::conf_error::ConfigResult;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::config::conf_write_guard::ConfigCategory;
use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::logging::log_trait::Log;
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
        match get_app_handle()
            .state::<ConfigManager>()
            .get_conf::<ScriptsConfig>(SCRIPTS_CONFIG_PATH)
            .await{
            Ok(conf) => conf.dir,
            Err(_) => ScriptsConfig::default().dir,
        }
    }

    pub async fn set_dir(dir: PathBuf) -> ConfigResult<()> {
        match get_app_handle()
            .state::<ConfigManager>()
            .get_conf_mut::<ScriptsConfig>(SCRIPTS_CONFIG_PATH)
            .await{
            Ok(mut conf) => conf.dir =  dir,
            Err(e) => {
                Log::error( format!(    "设置脚本根目录{}失败:{}", dir.to_string_lossy().to_string(), e.to_string()).as_str())
            },
        }
        Ok(())
    }
}
