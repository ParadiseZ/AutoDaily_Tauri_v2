use crate::domain::scripts::script_decision::Step;
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
#[serde(rename_all = "camelCase", tag = "type")]
pub enum VarValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
}
