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
    pub const APP_STORE: &str = "autodaily.config.json";
    pub const SYSTEM_SETTINGS_KEY: &str = "system_settings";
    pub const LOG_CONFIG_KEY: &str = "log_config";
    pub const SCRIPTS_CONFIG_KEY: &str = "scripts_config";
    pub const EMAIL_CONFIG_KEY: &str = "email_config";
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

pub mod table_name{
    pub const DEVICE_TABLE: &str = "devices";
    pub const SCRIPT_TABLE: &str = "scripts";
    pub const POLICY_TABLE: &str = "policies";
    pub const POLICY_GROUP_TABLE: &str = "policy_groups";
    pub const POLICY_SET_TABLE: &str = "policy_sets";

    pub const GROUP_POLICIES:&str =  "group_policies";
    pub const SET_GROUPS:&str =  "set_groups";
}