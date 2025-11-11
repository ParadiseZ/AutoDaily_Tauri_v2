use crate::domain::scripts::script_decision::Point;
use image::RgbaImage;

#[derive(Debug, Clone,)]
pub enum ADBCommand {
    Click(Point),
    Swipe(Point, Point),
    SwipeWithDuration(Point, Point, u32),
    Reboot,
    StartActivity(String,String),
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
    Resume
}

#[derive(Debug, Clone)]
pub enum ADBCmdConv{
    ADBShellCommand(String),
    ADBClientCommand(ADBCommand),
    ADBSleepCommand(ADBCommand::Duration(u64))
}

pub enum ADBCommandResult{
    Success,
    Failed(String),
    Output(Vec<u8>),
    Image(RgbaImage)
}