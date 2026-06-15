use crate::domain::config::shortcut_conf::ShortCut;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StartMode {
    Normal,
    Minimized,
    Tray,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IdleAction {
    None,
    Shutdown,
    Sleep,
    Hibernate,
}

fn default_dispatch_schedule_retention_days() -> u16 {
    7
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct SystemConfig {
    pub start_mode: StartMode,
    pub close_exit: bool,
    pub always_on_top: bool,
    pub idle_action: IdleAction,
    pub max_idle_retry_num: u8,
    pub auto_start: bool,
    pub dispatch_schedule_retention_days: u16,
    pub shortcut: ShortCut,
}

impl Default for SystemConfig {
    /// 创建基本默认配置（同步版本，使用临时路径）
    fn default() -> Self {
        Self {
            start_mode: StartMode::Normal,
            close_exit: true,
            always_on_top: false,
            idle_action: IdleAction::None,
            max_idle_retry_num: 3, // 默认最大重试3次
            auto_start: false,
            dispatch_schedule_retention_days: default_dispatch_schedule_retention_days(),
            shortcut: ShortCut::default(),
        }
    }
}
