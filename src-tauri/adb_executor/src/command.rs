use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {} {}", self.x, self.y)
    }
}

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

pub fn click_cmd(p: &Point) -> String {
    format!("{} {}", CLICK, p.to_string())
}
pub fn long_click_cmd(p: &Point, duration: &u32) -> String {
    swipe_with_duration_cmd(p, &Point::new(p.x + 1, p.y + 1), duration)
}
pub fn swipe_cmd(p1: &Point, p2: &Point) -> String {
    format!("{} {} {}", SWIPE, p1.to_string(), p2.to_string())
}

pub fn swipe_with_duration_cmd(p1: &Point, p2: &Point, duration: &u32) -> String {
    format!(
        "{} {} {} {}",
        SWIPE,
        p1.to_string(),
        p2.to_string(),
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

#[derive(Debug, Clone)]
pub enum ADBCommand {
    Click(Point),
    Swipe(Point, Point),
    SwipeWithDuration(Point, Point, u32),
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

    Pause,
    Resume,
}

#[derive(Debug, Clone)]
pub enum ADBCmdConv {
    ADBShellCommand(String),
    ADBClientCommand(ADBCommand),
    ADBSleepCommand(ADBCommand),
}

#[derive(Debug)]
pub enum ADBCommandResult {
    Success,
    Failed(String),
    Output(Vec<u8>),
    Image(RgbaImage),
}
