use crate::domain::scripts::point::{PointF32, PointU16};
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "ac")]
pub enum Action {
    ClickPoint {
        p: PointU16
    },
    ClickPercent {
        p: PointF32
    },
    ClickTxt {
        txt: Option<String>
    },
    ClickLabelIdx {
        idx: Option<u32>
    },
    SwipePercent {
        from: PointF32,
        to: PointF32,
        duration: u64,
    },
    SwipePoint{
        from : PointU16,
        to: PointU16,
        duration: u64
    },
    SwipeLabelIdx{
        from : u16,
        to: u16,
        duration: u64
    },
    SwipeTxt{
        from : Option<String>,
        to: Option<String>,
        duration: u64
    },
    Capture{
        output_var: String,
    },
    Reboot,
    LaunchApp{
        pkg_name: String,
    },
}