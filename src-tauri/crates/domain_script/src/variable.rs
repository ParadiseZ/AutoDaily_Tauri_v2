use ad_kernel::ids::{StepId, TaskId};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptVariableNamespace {
    Input,
    Runtime,
    System,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptVariableValueType {
    Int,
    Float,
    Bool,
    String,
    Json,
    List,
    Object,
    Image,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptVariableSourceType {
    Manual,
    StepOutput,
    SystemBuiltin,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptVariableCatalog {
    pub version: u32,
    pub variables: Vec<ScriptVariableDef>,
}

impl Default for ScriptVariableCatalog {
    fn default() -> Self {
        Self {
            version: 1,
            variables: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptVariableDef {
    pub id: String,
    pub key: String,
    pub name: String,
    pub namespace: ScriptVariableNamespace,
    pub value_type: ScriptVariableValueType,
    pub owner_task_id: Option<TaskId>,
    pub source_type: ScriptVariableSourceType,
    pub source_step_id: Option<StepId>,
    pub readable: bool,
    pub writable: bool,
    pub persisted: bool,
    pub ui_bindable: bool,
    pub default_value: Option<Value>,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_the_first_catalog_schema() {
        assert_eq!(ScriptVariableCatalog::default().version, 1);
    }
}
