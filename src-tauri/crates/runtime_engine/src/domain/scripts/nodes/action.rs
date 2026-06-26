use crate::domain::scripts::point::{PointF32, PointU16};
use crate::infrastructure::core::{Deserialize, PolicyId, Serialize};

fn default_click_offset() -> i32 {
    0
}

fn default_enable_filter() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "source")]
pub enum SwipeTarget {
    Txt {
        input_var: String,
        value: Option<String>,
        #[serde(default)]
        value_expr: Option<String>,
    },
    LabelIdx {
        input_var: String,
        idx: u16,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum SwipeMode {
    Percent {
        from: PointF32,
        to: PointF32,
    },
    Point {
        from: PointU16,
        to: PointU16,
    },
    LabelIdx {
        input_var: String,
        from: u16,
        to: u16,
    },
    Txt {
        input_var: String,
        from: Option<String>,
        to: Option<String>,
        #[serde(default)]
        from_expr: Option<String>,
        #[serde(default)]
        to_expr: Option<String>,
    },
    Mixed {
        from: SwipeTarget,
        to: SwipeTarget,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum ClickMode {
    Point {
        p: PointU16,
    },
    Percent {
        p: PointF32,
    },
    Txt {
        input_var: String,
        txt: Option<String>,
        #[serde(default)]
        txt_expr: Option<String>,
        #[serde(default = "default_enable_filter")]
        enable_filter: bool,
    },
    LabelIdx {
        input_var: String,
        idx: Option<u32>,
        #[serde(default)]
        idx_expr: Option<String>,
        #[serde(default = "default_enable_filter")]
        enable_filter: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "ac")]
pub enum Action {
    Click {
        #[serde(default = "default_click_offset")]
        offset_x: i32,
        #[serde(default = "default_click_offset")]
        offset_y: i32,
        #[serde(flatten)]
        mode: ClickMode,
    },
    Swipe {
        duration: u64,
        #[serde(flatten)]
        mode: SwipeMode,
    },
    LongClick {
        #[serde(default = "default_click_offset")]
        offset_x: i32,
        #[serde(default = "default_click_offset")]
        offset_y: i32,
        #[serde(flatten)]
        mode: ClickMode,
    },
    Capture {
        output_var: String,
    },
    Reboot,
    Back,
    Home,
    InputText {
        text: String,
    },
    PosAdd {
        target: PolicyId,
    },
    PosMinus {
        target: PolicyId,
    },
    DropSetNext {
        task: crate::infrastructure::core::TaskId,
        variable_id: String,
    },
    LaunchApp {
        pkg_name: String,
        activity_name: String,
    },
    StopApp {
        pkg_name: String,
    },
}
