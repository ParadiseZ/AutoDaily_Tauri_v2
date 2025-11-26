use crate::infrastructure::core::{
    Deserialize, GuardId, HashMap, PolicyId, ScriptId, Serialize, SubFlowId, TaskId,
};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum DecisionError {
    #[error("Repository error: {0}")]
    Repository(String),
    #[error("脚本{script_id}不存在！")]
    DecisionNotFound(String),
}

pub type DecisionResult<T> = Result<T, DecisionError>;

// 逻辑组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicOp {
    And,
    Or,
}

// 条件叶子（表达式由解释器处理，表达式语言后续可替换）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionLeaf {
    pub expr: String,
}

// 条件组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub op: LogicOp,
    pub items: Vec<ConditionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConditionNode {
    Leaf { leaf: ConditionLeaf },
    Group { group: ConditionGroup },
}

// 动作引用：可调用内置动作，或引用可复用子流程
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardDef {
    pub id: GuardId,
    pub name: String,
    pub condition: ConditionNode,
    pub action: ActionRef,
    #[serde(default)]
    pub priority: u32,
}

// 策略条目：在特定目标下，命中条件时执行对应动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDef {
    pub id: PolicyId,
    pub when_goal: String,
    pub task_id: TaskId,
    pub priority: u32,
    pub condition: ConditionNode,
    pub action: ActionRef,
}

// 可复用子流程（小型图/序列），供 ActionRef::SubFlow 复用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubFlowDef {
    pub id: SubFlowId,
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Step {
    //Router { to: Option<Uuid> },
    Sequence {
        steps: Vec<Step>,
        reverse: bool,
    },
    If {
        cond: String,
        then_steps: Box<Step>,
        else_steps: Option<Box<Step>>,
    },
    While {
        cond: String,
        steps: Box<Step>,
        max_loop: Option<u32>,
    },

    ForEachActivity {
        filter: Option<Vec<String>>, // 可选：只处理指定活动
        body: Box<Step>,             // 可用 {{activity.id}} 等变量
    },
    WaitMs {
        ms: u64,
    },
    WaitUntil {
        cond: String,
        timeout_ms: u64,
    },

    SetVar {
        name: String,
        value_expr: String,
    }, // value_expr 是 Rhai 表达式
    GetVar {
        name: String,
    },
    //操作
    Click {
        pos_idx: i32,
        verify: Option<Box<Step>>,
    },
    ClickLabelIdx {
        label_idx: i32,
        pos_idx: Option<i32>,
        verify: Option<Box<Step>>,
    },
    ClickLabel {
        name: String,
        pos_idx: Option<i32>,
        verify: Option<Box<Step>>,
    },
    ClickText {
        text: String,
        pos_idx: Option<i32>,
        verify: Option<Box<Step>>,
    },
    ClickPoint {
        x: i32,
        y: i32,
        verify: Option<Box<Step>>,
    },
    ClickPercent {
        x: f32,
        y: f32,
        verify: Option<Box<Step>>,
    },
    //安卓
    SwipeDet {
        from: LabelType,
        to: LabelType,
        verify: Option<Box<Step>>,
    },
    SwipeTxt {
        from: String,
        to: String,
        verify: Option<Box<Step>>,
    },
    SwipePoint {
        from: Point,
        to: Point,
        verify: Option<Box<Step>>,
    },
    SwipePercent {
        from: PointPercent,
        to: PointPercent,
        verify: Option<Box<Step>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum LabelType {
    LabelIdx { idx: i32 },
    LabelName { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "op")]
pub struct Point {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 输出格式为 "x y"，直接适配ADB的坐标参数
        write!(f, " {} {}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub struct PointPercent {
    x: f32,
    y: f32,
}

// DDD 仓储接口（同步，便于简化依赖；调用方如需并发可自行 spawn）
pub trait GuardRepository: Send + Sync {
    fn load_common_guards(&self) -> DecisionResult<Vec<GuardDef>>;
    fn load_script_guards(&self, _script_id: ScriptId) -> DecisionResult<Vec<GuardDef>>;
}

pub trait PolicyRepository: Send + Sync {
    fn load_common_policies(&self) -> DecisionResult<Vec<PolicyDef>>;
    fn load_script_policies(&self, _script_id: ScriptId) -> DecisionResult<Vec<PolicyDef>>;
}

pub trait SubFlowRepository: Send + Sync {
    fn load_common_subflows(&self) -> DecisionResult<Vec<SubFlowDef>>;
    fn load_script_subflows(&self, _script_id: ScriptId) -> DecisionResult<Vec<SubFlowDef>>;
}
