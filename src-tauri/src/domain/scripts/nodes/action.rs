use crate::domain::scripts::point::Point;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "ac")]
pub enum Action {
    ClickPoint {
        #[ts(as = "PointU16")]
        p: Point<u16>
    },
    ClickPercent {
        p: Point<f32>
    },
    ClickTxt {
        txt: Option<String>
    },
    ClickLabelIdx {
        idx: Option<u32>
    },
    SwipePercent {
        from: Point<f32>,
        to: Point<f32>,
        duration: u64,
    },
    SwipePoint{
        #[ts(as = "PointU16")]
        from : Point<u16>,
        #[ts(as = "PointU16")]
        to: Point<u16>,
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