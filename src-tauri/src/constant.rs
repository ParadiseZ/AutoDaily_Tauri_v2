// 临时常量模块 - 保持向后兼容

pub mod project {
    use std::time::Duration;

    pub const MAIN_WINDOW: &str = "AutoDaily";

    pub const SCREENSHOT_DIR: &str = "screencap_test";

    pub const SOCKET_NAME: &str = "com.auto.daily";
    pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 10;
    pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
    pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(5);
}

pub mod sys_conf_path {
    pub const PERFORMANCE_CONFIG_PATH: &str = "performance.json";
    pub const SYSTEM_SETTINGS_PATH: &str = "system_settings.json";
    pub const LOG_CONFIG_PATH: &str = "logs.json";

    pub const SCRIPTS_CONFIG_PATH: &str = "scripts.json";

    pub const DEVICES_CONFIG_PATH: &str = "devices.json";

    pub const EMAIL_CONFIG_PATH: &str = "email.json";
}

pub mod scripts_files {
    pub const SCRIPT_INFO_FILE: &str = "info.json";

    pub const COMMON_GUARD_FILE: &str = "guards.common.json";

    pub const GUARD_FILE: &str = "guards.json";

    pub const COMMON_POLICIES_FILE: &str = "policies.common.json";

    pub const POLICIES_FILE: &str = "policies.json";

    pub const COMMON_SUBFLOW_FILE: &str = "subflows.common.json";

    pub const SUBFLOW_FILE: &str = "subflows.json";
}
