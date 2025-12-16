use std::ops::Add;
use image::RgbaImage;
use crate::domain::scripts::point::Point;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;

// Constants
pub const BACK: &str = "input keyevent 4";
pub const HOME: &str = "input keyevent 3";
pub const POWER: &str = "input keyevent 26";
pub const MUTE: &str = "input keyevent 164";
pub const CLICK: &str = "input tap";
pub const SWIPE: &str = "input swipe";
pub const STOP_APP: &str = "am force-stop";
pub const COMMAND_WRITE_ERROR_MSG: &str = "ADB命令写入缓冲区数据失败！";

pub fn sleep_cmd(interval: u64) -> String {
    format!("sleep {}", interval)
}

pub fn click_cmd(p: &Point<u16>) -> String {
    format!("{} {},{}", CLICK, p.x,p.y)
}
pub fn long_click_cmd(p: &Point<u16>, duration: &u32) -> String {
    swipe_with_duration_cmd(p, &p.add(Point::new(1, 1)), duration)
}
pub fn swipe_cmd(p1: &Point<u16>, p2: &Point<u16>) -> String {
    format!("{} {},{} {},{}", SWIPE, p1.x,p1.y, p2.x,p2.y)
}

pub fn swipe_with_duration_cmd(p1: &Point<u16>, p2: &Point<u16>, duration: &u32) -> String {
    format!(
        "{} {},{} {},{} {}",
        SWIPE,
        p1.x,
        p1.y,
        p2.x,
        p2.y,
        duration
    )
}

pub fn press_cmd(key: &str) -> String {
    format!("input keyevent {}", key)
}

pub fn input_text_cmd(text: &str) -> String {
    format!("input text {}", text)
}

pub fn stop_app_cmd(package_name: &str) -> String {
    format!("{} {}", STOP_APP, package_name)
}

#[derive(Clone)]
pub enum ADBCommand {
    Click(Point<u16>),
    Swipe(Point<u16>, Point<u16>),
    SwipeWithDuration(Point<u16>, Point<u16>, u32),
    Reboot,
    StartActivity(String, String),
    Capture(crossbeam_channel::Sender<RgbaImage>),
    StopApp(String),
    InputText(String),
    Back,
    Home,

    //合并为单条指令
    Sequence(Vec<ADBCommand>),
    //Sequence则合并，其他则睡眠
    Duration(u64),

    //以下命令不参与执行器的操作执行
    Loop(Vec<ADBCommand>),
    StopLoop(bool),
    ChangeConnectConfig(ADBConnectConfig),

    Pause,
    Resume,
}

impl std::fmt::Display for ADBCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBCommand::Click(p) => write!(f, "{} {} {}", CLICK, p.x, p.y),
            ADBCommand::Swipe(p1, p2) => write!(f, "{} {},{} {},{}", SWIPE, p1.x, p1.y, p2.x, p2.y),
            ADBCommand::SwipeWithDuration(p1, p2, duration) => write!(f, "{} {},{} {},{} {}", SWIPE, p1.x, p1.y, p2.x, p2.y, duration),
            ADBCommand::Reboot => write!(f, "reboot:{}", POWER),
            ADBCommand::StartActivity(package_name, activity_name) => write!(f, "am start -n {}/{}", package_name, activity_name),
            ADBCommand::Capture(_) => write!(f, "capture"),
            ADBCommand::StopApp(package_name) => write!(f, "{} {}", STOP_APP, package_name),
            ADBCommand::InputText(text) => write!(f, "input:{}", text),
            ADBCommand::Back => write!(f, "back:{}", BACK),
            ADBCommand::Home => write!(f, "home:{}", HOME),
            ADBCommand::Sequence(commands) => write!(f, "sequence:{}", adb_cmd_vec_to_string(commands).as_str()),
            ADBCommand::Duration(duration) => write!(f, "{}", duration),
            ADBCommand::Loop(commands) => write!(f, "sequence:{}", adb_cmd_vec_to_string(commands).as_str()),
            ADBCommand::StopLoop(is_stop) => write!(f, "stop_loop:{}", is_stop),
            ADBCommand::ChangeConnectConfig(config) => write!(f, "change_connect_config:{}", config),
            ADBCommand::Pause => write!(f, "pause"),
            ADBCommand::Resume => write!(f, "resume"),
        }
    }
}

fn adb_cmd_vec_to_string(commands: &Vec<ADBCommand>) -> String {
    let mut cmds = String::new();
    for command in commands {
        cmds.push_str(&command.to_string());
        cmds.push_str(",");
    }
    cmds.pop();
    cmds
}

#[derive(Clone)]
pub enum ADBCmdConv {
    ADBShellCommand(String),
    ADBClientCommand(ADBCommand),
    ADBSleepCommand(ADBCommand),
}

impl std::fmt::Display for ADBCmdConv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADBCmdConv::ADBShellCommand(cmd) => write!(f, "{}", cmd),
            ADBCmdConv::ADBClientCommand(cmd) => write!(f, "{}", cmd),
            ADBCmdConv::ADBSleepCommand(cmd) => write!(f, "{}", cmd),
        }
    }
}

#[derive(Debug)]
pub enum ADBCommandResult {
    Success,
    Failed(String),
    Output(Vec<u8>),
    Image(RgbaImage),
}
