use crate::{CompareOp, Step};
use domain_vision::SearchRule;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum VisionNode {
    Detect {
        input_var: String,
        out_var: String,
    },
    Ocr {
        input_var: String,
        out_var: String,
    },
    CountCompare {
        input_var: String,
        out_var: String,
        target_value: Option<String>,
        op: CompareOp,
        expected_count: i32,
        then_steps: Vec<Step>,
    },
    VisionSearch {
        #[serde(default)]
        det_res_var: Option<String>,
        #[serde(default)]
        ocr_res_var: Option<String>,
        rule: SearchRule,
        out_var: String,
        #[serde(default)]
        out_det_var: Option<String>,
        #[serde(default)]
        out_ocr_var: Option<String>,
        then_steps: Vec<Step>,
    },
}
