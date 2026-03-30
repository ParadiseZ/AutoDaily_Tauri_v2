use crate::infrastructure::core::{Deserialize, Serialize, StepId, TaskId};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptVariableNamespace {
    /// 用户可配置、可持久化、可用于 UI 绑定的输入变量。
    Input,
    /// 步骤运行过程中产生的临时变量，例如 OCR 结果、计数器、命中结果。
    Runtime,
    /// 引擎注入的只读系统变量，例如运行次数、当前任务、时间等。
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptVariableValueType {
    /// 32 位整数。
    Int,
    /// 32 位浮点数。
    Float,
    /// 布尔值。
    Bool,
    /// 字符串。
    String,
    /// 任意 JSON 结构，通常用于复杂输入或兼容旧数据。
    Json,
    /// 列表值。
    List,
    /// 对象值。
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptVariableSourceType {
    /// 由脚本作者手工定义。
    Manual,
    /// 由某个步骤的输出自动产生。
    StepOutput,
    /// 由运行时环境内建提供。
    SystemBuiltin,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptVariableCatalog {
    /// 变量目录结构版本，后续迁移和兼容处理依赖此字段。
    pub version: u32,
    /// 当前脚本声明的全部变量定义。
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

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptVariableDef {
    /// 变量定义的稳定主键，供 UI 绑定和步骤引用使用。
    pub id: String,
    /// 运行时使用的变量名，例如 input.sweepCount / runtime.ocrText。
    pub key: String,
    /// 编辑器里展示的变量名称。
    pub name: String,
    /// 变量所属命名空间：输入、运行时或系统。
    pub namespace: ScriptVariableNamespace,
    /// 变量值类型，用于编辑器过滤、校验和控件生成。
    pub value_type: ScriptVariableValueType,
    /// 所属任务 ID，null 表示脚本级公共变量。
    pub owner_task_id: Option<TaskId>,
    /// 变量来源类型：手工定义、步骤输出或系统内建。
    pub source_type: ScriptVariableSourceType,
    /// 如果变量来自某个步骤输出，这里记录步骤 ID。
    pub source_step_id: Option<StepId>,
    /// 是否允许被步骤或表达式读取。
    pub readable: bool,
    /// 是否允许被 setVar 等步骤写入。
    pub writable: bool,
    /// 是否持久化变量值。输入变量通常为 true，运行时和系统变量通常为 false。
    pub persisted: bool,
    /// 是否允许 UI 字段直接绑定该变量。
    pub ui_bindable: bool,
    /// 变量默认值。输入变量常用，运行时和系统变量通常为空。
    pub default_value: Option<Value>,
    /// 变量用途说明，帮助脚本作者理解该变量的业务含义。
    pub description: String,
}
