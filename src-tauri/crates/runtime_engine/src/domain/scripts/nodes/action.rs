use crate::domain::scripts::point::{PointF32, PointU16};
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum SwipeMode {
    Percent {
        from: PointF32,
        to: PointF32,
    },
    Point{
        from : PointU16,
        to: PointU16,
    },
    LabelIdx{
        from : u16,
        to: u16,
    },
    Txt{
        from : Option<String>,
        to: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum ClickMode {
    Point {
        p: PointU16
    },
    Percent {
        p: PointF32
    },
    Txt {
        txt: Option<String>
    },
    LabelIdx {
        idx: Option<u32>
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "ac")]
pub enum Action {
    Click {
        #[serde(flatten)]
        mode: ClickMode,
    },
    Swipe {
        duration: u64,
        #[serde(flatten)]
        mode: SwipeMode,
    },
    Capture{
        output_var: String,
    },
    Reboot,
    LaunchApp{
        pkg_name: String,
    },
    StopApp{
        pkg_name: String,
    },
}