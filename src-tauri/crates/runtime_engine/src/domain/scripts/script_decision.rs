use crate::domain::scripts::nodes::action::Action;
use crate::domain::scripts::nodes::data_handing::DataHanding;
use crate::domain::scripts::nodes::flow_control::FlowControl;
use crate::domain::scripts::nodes::task_control::TaskControl;
use crate::domain::scripts::nodes::vision_node::VisionNode;
use crate::infrastructure::core::{Deserialize, Serialize, StepId};

/*// 逻辑组合
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum LogicOp {
    And,
    Or,
}

// 条件叶子（表达式由解释器处理，表达式语言后续可替换）
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ConditionLeaf {
    pub expr: String,
}

// 条件组
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ConditionGroup {
    pub op: LogicOp,
    pub items: Vec<ConditionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ConditionNode {
    Leaf { leaf: ConditionLeaf },
    Group { group: ConditionGroup },
}

// 动作引用：可调用内置动作，或引用可复用子流程
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum ActionRef {
    Builtin {
        name: String
    },
    SubFlow {
        id: SubFlowId
    },
}

// 守卫：高优先级全局拦截处理
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GuardDef {
    pub id: GuardId,
    pub name: String,
    pub condition: ConditionNode,
    pub action: ActionRef,
    #[serde(default)]
    pub priority: u32,
}

// 策略条目：在特定目标下，命中条件时执行对应动作
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PolicyDef {
    pub id: PolicyId,
    pub when_goal: String,
    pub task_id: TaskId,
    pub priority: u32,
    pub condition: ConditionNode,
    pub action: ActionRef,
}

// 可复用子流程（小型图/序列），供 ActionRef::SubFlow 复用
#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SubFlowDef {
    pub id: SubFlowId,
    pub name: String,
    pub steps: Vec<Step>,
}
*/
#[derive(Debug, Serialize, Deserialize,Clone, ts_rs::TS)]
#[ts(export)]
pub struct Step {
    pub id: Option<StepId>,
    pub source_id : Option<StepId>,
    pub target_id: Option<StepId>,
    pub label: Option<String>,

    #[serde(default)]
    pub skip_flag: bool,

    #[serde(flatten)]
    pub kind: StepKind,
}

#[derive(Debug, Serialize, Deserialize,Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag="op")]
pub enum StepKind {
    //Router { to: Option<Uuid> },
    Sequence {
        steps: Vec<Step>,
    },

    Action{
        exec_max: u32,
        a: Action
    },

    DataHanding{
        a: DataHanding
    },

    FlowControl{
        a: FlowControl
    },
    TaskControl{
        a: TaskControl,
    },
    Vision{
        a:VisionNode
    }
}

/*// DDD 仓储接口（同步，便于简化依赖；调用方如需并发可自行 spawn）
pub trait GuardRepository: Send + Sync {
    fn load_common_guards(&self) -> ScriptResult<Vec<GuardDef>>;
    fn load_script_guards(&self, _script_id: ScriptId) -> ScriptResult<Vec<GuardDef>>;
}

pub trait PolicyRepository: Send + Sync {
    fn load_common_policies(&self) -> ScriptResult<Vec<PolicyDef>>;
    fn load_script_policies(&self, _script_id: ScriptId) -> ScriptResult<Vec<PolicyDef>>;
}

pub trait SubFlowRepository: Send + Sync {
    fn load_common_subflows(&self) -> ScriptResult<Vec<SubFlowDef>>;
    fn load_script_subflows(&self, script_id: ScriptId) -> ScriptResult<Vec<SubFlowDef>>;
}
*/
