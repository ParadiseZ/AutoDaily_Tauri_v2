use crate::domain::entities::config::shortcut::ShortCut;
use crate::domain::trait_ad::config_category::ConfigCategory;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StartModel {
    Normal,
    Minimized,
    Tray
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdleAction {
    None,
    Shutdown,
    Sleep,
    Hibernate
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub start_mode: StartModel,
    pub close_exit: bool,
    pub always_on_top: bool,
    pub idle_action: IdleAction,
    pub max_idle_retry_num: u8,
    pub auto_start: bool,
    pub rem_size_position: bool,
    pub shortcut: ShortCut,
}

impl ConfigCategory for SystemConfig{
    /// 创建基本默认配置（同步版本，使用临时路径）
    fn default() -> Self {
        Self{
            start_mode: StartModel::Normal,
            close_exit: true,
            always_on_top: false,
            idle_action: IdleAction::None,
            max_idle_retry_num : 3, // 默认最大重试3次
            auto_start: false,
            rem_size_position: false,
            shortcut: ShortCut::default()
        }
    }
}