use crate::domain::scripts::script_decision::Step;
use crate::domain::vision::ocr_search::{
    RelativeAnchorType, RelativeDirection, RelativeTargetKind,
};
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DataHanding {
    SetVar {
        name: String,
        val: Option<VarValue>, // 如果有提供，则设置静态值
        expr: Option<String>,  // 否则通过执行 rhai 表达式获取值
    },
    GetVar {
        name: String,
        default_val: Option<VarValue>, // 如果找不到该值时的默认值
    },
    Filter {
        input_var: String,
        out_name: String,
        mode: FilterMode, // Filter 或是 Map 模式
        logic_expr: String,
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
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FilterMode {
    Filter,
    Map,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ColorCompareMethod {
    OklabDistance { threshold: f32 },
}

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum VarValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
}
