// 时间模板

use crate::infrastructure::core::{Deserialize, Serialize, TemplateId};
use sqlx::FromRow;

/// 时间模板：定义可选的运行时间窗口
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TimeTemplate {
    pub id: TemplateId,
    pub name: String,
    /// 开始时间 (HH:MM 格式)
    pub start_time: Option<String>,
    /// 结束时间 (HH:MM 格式)
    pub end_time: Option<String>,
}

impl Default for TimeTemplate {
    fn default() -> Self {
        Self {
            id: TemplateId::new_v7(),
            name: String::new(),
            start_time: None,
            end_time: None,
        }
    }
}
