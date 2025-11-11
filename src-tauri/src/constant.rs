// 临时常量模块 - 保持向后兼容

pub mod project {
    use std::time::Duration;

    pub const MAIN_WINDOW: &str = "AutoDaily";

    pub const SCREENSHOT_DIR : &str = "screencap_test";

    pub const SOCKET_NAME:&str = "com.auto.daily";
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

pub mod scripts_files{
    pub const SCRIPT_INFO_FILE: &str = "info.json";

    pub const COMMON_GUARD_FILE :&str = "guards.common.json";

    pub const GUARD_FILE :&str = "guards.json";

    pub const COMMON_POLICIES_FILE:&str  = "policies.common.json";

    pub const POLICIES_FILE :&str  = "policies.json";

    pub const COMMON_SUBFLOW_FILE:&str = "subflows.common.json";

    pub const SUBFLOW_FILE :&str = "subflows.json";
}

pub mod adb_command {
    use crate::domain::scripts::script_decision::Point;
    pub const BACK: &str = "input keyevent 4";
    pub const HOME: &str = "input keyevent 3";
    pub const POWER: &str = "input keyevent 26";
    pub const MUTE : &str = "input keyevent 164";
    pub const CLICK: &str = "input tap";
    pub const SWIPE: &str = "input swipe";
    pub const STOP_APP: &str = "am force-stop";
    pub const COMMAND_WRITE_ERROR_MSG : &str = "ADB命令写入缓冲区数据失败！";

    pub fn sleep_cmd(interval: u64) -> String {
        format!("sleep {}" , interval)
    }

    pub fn click_cmd(p : &Point) -> String {
        format!("{} {}", CLICK, p.to_string())
    }
    pub fn long_click_cmd(p : &Point,duration : &u32) -> String {
        swipe_with_duration_cmd(p, &Point::new(p.x + 1, p.y + 1), duration)
    }
    pub fn swipe_cmd(p1: &Point, p2: &Point) -> String {
        format!( "{} {} {}", SWIPE, p1.to_string(), p2.to_string())
    }

    pub fn swipe_with_duration_cmd(p1: &Point, p2: &Point, duration: &u32) -> String {
        format!( "{} {} {} {}", SWIPE, p1.to_string(), p2.to_string(), duration)
    }

    pub fn press_cmd(key: &str) -> String {
        format!( "input keyevent {}", key)
    }

    pub fn input_text_cmd(text: &str) -> String {
        format!( "input text {}", text)
    }

    pub fn stop_app_cmd(package_name: &str) -> String {
        format!( "{} {}", STOP_APP, package_name)
    }
}