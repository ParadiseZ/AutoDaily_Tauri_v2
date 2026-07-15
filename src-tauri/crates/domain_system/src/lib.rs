use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShortCut {
    pub toggle_window: String,
    pub toggle_all_scripts: String,
    pub capture: String,
}

impl Default for ShortCut {
    fn default() -> Self {
        Self {
            toggle_window: "CommandOrControl+H".into(),
            toggle_all_scripts: "Alt+R".into(),
            capture: "Alt+A".into(),
        }
    }
}

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
    fn default() -> Self {
        Self {
            start_mode: StartMode::Normal,
            close_exit: true,
            always_on_top: false,
            idle_action: IdleAction::None,
            max_idle_retry_num: 3,
            auto_start: false,
            dispatch_schedule_retention_days: default_dispatch_schedule_retention_days(),
            shortcut: ShortCut::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_a_normal_local_session() {
        let config = SystemConfig::default();
        assert!(matches!(config.start_mode, StartMode::Normal));
        assert_eq!(config.dispatch_schedule_retention_days, 7);
    }
}
