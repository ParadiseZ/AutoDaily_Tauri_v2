use crate::{
    RuntimeType, SCRIPT_RUNTIME_SCHEMA, ScriptPlatform, ScriptType, ScriptVariableCatalog,
    supported_script_features,
};
use ad_kernel::ids::{ScriptId, TaskId, UserId};
use domain_vision::{DetectorType, RecognizerType};
use serde::{Deserialize, Serialize};

fn default_allow_clone() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInfo {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub content_md: Option<String>,
    pub user_id: UserId,
    pub user_name: Option<String>,
    pub runtime_type: RuntimeType,
    #[serde(default)]
    pub platform: ScriptPlatform,
    pub sponsorship_qr: Option<String>,
    pub sponsorship_url: Option<String>,
    pub contact_info: Option<String>,
    pub img_det_model: Option<DetectorType>,
    pub txt_det_model: Option<DetectorType>,
    pub txt_rec_model: Option<RecognizerType>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    pub ver_name: String,
    pub ver_num: u32,
    pub latest_ver: u32,
    pub download_count: u32,
    pub script_type: ScriptType,
    #[serde(default)]
    pub is_valid: bool,
    #[serde(default = "default_allow_clone")]
    pub allow_clone: bool,
    #[serde(default)]
    pub min_app_version: Option<String>,
    #[serde(default)]
    pub min_runtime_schema: Option<u32>,
    #[serde(default)]
    pub required_features: Vec<String>,
    pub variable_catalog: ScriptVariableCatalog,
    pub cloud_id: Option<ScriptId>,
    #[serde(default)]
    pub runtime_settings: ScriptRuntimeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptRuntimeSettings {
    pub recovery_task_id: Option<TaskId>,
    #[serde(default)]
    pub click_random_offset: u16,
}

impl Default for ScriptInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            content_md: None,
            user_id: UserId::new_v7(),
            user_name: None,
            runtime_type: RuntimeType::Rhai,
            platform: ScriptPlatform::default(),
            sponsorship_qr: None,
            sponsorship_url: None,
            contact_info: None,
            img_det_model: None,
            txt_det_model: None,
            txt_rec_model: None,
            create_time: None,
            update_time: None,
            ver_name: String::new(),
            ver_num: 0,
            latest_ver: 0,
            download_count: 0,
            script_type: ScriptType::Dev,
            is_valid: false,
            allow_clone: true,
            min_app_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            min_runtime_schema: Some(SCRIPT_RUNTIME_SCHEMA),
            required_features: supported_script_features(),
            variable_catalog: ScriptVariableCatalog::default(),
            cloud_id: None,
            runtime_settings: ScriptRuntimeSettings::default(),
        }
    }
}

impl Default for ScriptRuntimeSettings {
    fn default() -> Self {
        Self {
            recovery_task_id: None,
            click_random_offset: 0,
        }
    }
}
