use ad_kernel::Point;

/// 与控制协议无关的设备操作。
#[derive(Debug, Clone)]
pub enum DeviceOperation {
    Click(Point<u16>),
    LongClick(Point<u16>),
    Swipe {
        from: Point<u16>,
        to: Point<u16>,
        duration: u64,
    },
    LaunchApp {
        pkg_name: String,
        activity_name: String,
    },
    StopApp {
        pkg_name: String,
    },
    InputText(String),
    Back,
    Home,
    Reboot,
    Delay(u64),
}
