use crate::Step;
use domain_vision::SearchRule;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInfo {
    pub name: String,
    pub note: String,
    pub log_print: Option<String>,
    pub cur_pos: u16,
    #[serde(default)]
    pub skip_flag: bool,
    pub exec_max: u16,
    pub before_action: Vec<Step>,
    pub cond: SearchRule,
    pub after_action: Vec<Step>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupInfo {
    pub name: String,
    pub note: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetInfo {
    pub name: String,
    pub note: String,
}
