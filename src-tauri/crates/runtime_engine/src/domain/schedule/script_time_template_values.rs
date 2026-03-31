use crate::infrastructure::core::{Deserialize, ScriptId, ScriptTemplateValueId, Serialize, TemplateId};
use sqlx::FromRow;

/// 某脚本在某个时间模板下保存的一整套输入变量值。
///
/// 这层数据不是时间模板本身，也不是脚本定义本身，而是：
/// `script_id + time_template_id` 这对组合下的“用户配置快照”。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTimeTemplateValues {
    /// 记录主键，便于后续单条保存、更新和删除。
    pub id: ScriptTemplateValueId,
    /// 关联的脚本 ID，表示这套值属于哪个脚本。
    pub script_id: ScriptId,
    /// 关联的时间模板 ID，表示这套值在哪个时间模板下生效。
    pub time_template_id: TemplateId,
    /// 变量值快照。
    ///
    /// 建议按 `variableCatalog.variables[*].id` 作为 key 存储，而不是按 UI 字段存储。
    /// UI 只是变量的配置入口，真正持久化的是变量值。
    #[ts(type = "any")]
    pub values_json: sqlx::types::Json<serde_json::Value>,
    /// 记录创建时间。
    pub created_at: String,
    /// 最后一次保存时间。
    pub updated_at: String,
}

impl Default for ScriptTimeTemplateValues {
    fn default() -> Self {
        Self {
            id: ScriptTemplateValueId::new_v7(),
            script_id: ScriptId::new_v7(),
            time_template_id: TemplateId::new_v7(),
            values_json: sqlx::types::Json(serde_json::json!({})),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
