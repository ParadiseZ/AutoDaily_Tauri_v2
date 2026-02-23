use regex::Regex;
use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DataHanding{
    SetVar {
        name: String,
        value_expr: String,
    }, // value_expr 是 Rhai 表达式
    GetVar {
        name: String,
    },
    Filter{
        cond:String,
        then_steps: Box<Step>,
        output_var: Option<String>,
    },
    /// 结果过滤与逻辑处理 (e.g. 筛选数字并比较)
    FilterHits {
        input_var: String,  // Vec<SearchHit>
        output_var: String, // 根据逻辑输出 bool 或 filtered hits
        logic_expr: String, // Rhai 表达式，用于进一步过滤或判定
    },
    FilterByRegex{
        show: String,
        regex: Regex,
        out_var: String,
    }
}