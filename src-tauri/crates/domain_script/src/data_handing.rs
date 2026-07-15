use crate::{PointF32, PointU16, Step};
use domain_vision::{RelativeAnchorType, RelativeDirection, RelativeTargetKind};
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "mode")]
pub enum RegionPoint {
    Point { p: PointU16 },
    Percent { p: PointF32 },
}

impl Default for RegionPoint {
    fn default() -> Self {
        Self::Point {
            p: PointU16 { x: 0, y: 0 },
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DataHanding {
    SetVar {
        name: String,
        val: Option<VarValue>,
        json_val: Option<Value>,
        expr: Option<String>,
    },
    ClearVars {
        names: Vec<String>,
    },
    GetVar {
        name: String,
        default_val: Option<VarValue>,
    },
    Filter {
        input_var: String,
        out_name: String,
        mode: FilterMode,
        logic_expr: String,
        #[serde(default)]
        region_top_left: RegionPoint,
        #[serde(default)]
        region_bottom_right: RegionPoint,
        then_steps: Vec<Step>,
    },
    ColorCompare {
        input_var: String,
        out_var: String,
        target_text: Option<String>,
        is_font: bool,
        target_color: ColorRgb,
        method: ColorCompareMethod,
        #[serde(default)]
        region_top_left: RegionPoint,
        #[serde(default)]
        region_bottom_right: RegionPoint,
        #[serde(default)]
        then_steps: Vec<Step>,
    },
    RelativeFilter {
        input_var: String,
        out_var: String,
        anchor_type: RelativeAnchorType,
        anchor_text: String,
        anchor_idx: i32,
        direction: RelativeDirection,
        target_kind: RelativeTargetKind,
        max_offset_x: Option<i32>,
        max_offset_y: Option<i32>,
        target_index: Option<usize>,
        #[serde(default)]
        then_steps: Vec<Step>,
    },
    Rhai {
        code: String,
        out_var: Option<String>,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FilterMode {
    Filter,
    Map,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ColorCompareMethod {
    OklabDistance { threshold: f32 },
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum VarValue {
    Int { value: i32 },
    Float { value: f32 },
    Bool { value: bool },
    String { value: String },
}
