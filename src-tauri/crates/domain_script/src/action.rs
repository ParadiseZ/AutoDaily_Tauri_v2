use crate::{PointF32, PointU16};
use ad_kernel::ids::{PolicyId, TaskId};

fn default_click_offset() -> i32 {
    0
}

fn default_enable_filter() -> bool {
    true
}

fn default_drop_set_cycle() -> bool {
    true
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, ts_rs::TS, Default)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum DropSetDirection {
    #[default]
    Increase,
    Decrease,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum SwipeMode {
    Percent {
        from: PointF32,
        to: PointF32,
        #[serde(default)]
        from_expr: Option<String>,
        #[serde(default)]
        to_expr: Option<String>,
    },
    Point {
        from: PointU16,
        to: PointU16,
        #[serde(default)]
        from_expr: Option<String>,
        #[serde(default)]
        to_expr: Option<String>,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum ClickMode {
    Point {
        p: PointU16,
        #[serde(default)]
        p_expr: Option<String>,
    },
    Percent {
        p: PointF32,
        #[serde(default)]
        p_expr: Option<String>,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
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
        task: TaskId,
        variable_id: String,
        #[serde(default)]
        direction: DropSetDirection,
        #[serde(default = "default_drop_set_cycle")]
        cycle: bool,
    },
    LaunchApp {
        pkg_name: String,
        #[serde(default)]
        pkg_name_expr: Option<String>,
        activity_name: String,
        #[serde(default)]
        activity_name_expr: Option<String>,
    },
    StopApp {
        pkg_name: String,
        #[serde(default)]
        pkg_name_expr: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_click_offsets_when_loading_existing_actions() {
        let action: Action = serde_json::from_value(serde_json::json!({
            "ac": "click",
            "mode": "point",
            "p": { "x": 1, "y": 2 }
        }))
        .unwrap();

        assert!(matches!(
            action,
            Action::Click {
                offset_x: 0,
                offset_y: 0,
                ..
            }
        ));
    }
}
