use regex::Regex;
use crate::domain::scripts::nodes::flow_control::VarValue;
use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum DataHanding{
    SetVar {
        name: String,
        val: VarValue,
    }, // value_expr 是 Rhai 表达式
    GetVar {
        name: String,
        val: VarValue,
    },
    Filter{
        cond:String,
        then_steps: Box<Step>,
        out:VarValue,
    },
    /// 结果过滤与逻辑处理 (e.g. 筛选数字并比较)
    FilterHits {
        input_var: String,  // Vec<SearchHit>
        out_name: String, // 根据逻辑输出 bool 或 filtered hits
        out_val: VarValue,
        logic_expr: String, // Rhai 表达式，用于进一步过滤或判定
        then_steps: Box<Step>,
    },
    FilterByRegex{
        cond:String,
        regex: Regex,
        out_var: String,
    }
}