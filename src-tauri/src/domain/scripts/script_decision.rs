use crate::domain::scripts::Point::Point;
use crate::infrastructure::core::{
    Deserialize, GuardId, PolicyId, ScriptId, Serialize, SubFlowId, TaskId,
};
use crate::infrastructure::scripts::script_error::ScriptResult;

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
    Continue,
    Break,
    Filter{
        cond:String,
        then_steps: Box<Step>,
        output_var: Option<String>,
    },
    If {
        cond: String,
        then_steps: Vec<Step>,
        else_steps: Option<Box<Step>>,
    },
    While {
        cond: String,
        steps: Vec<Step>,
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
    // 视觉/设备操作
    TakeScreenshot {
        output_var: String, // 存储 ImageHandle/Path 的变量名
    },
    DetRec {
        det_var: String, // 输入图片变量
        output_var: String, // 输出结果变量 (DetectionResult)
    },
    Ocr {
        image_var: String, // 输入图片变量
        output_var: String, // 输出结果变量 (DetectionResult)
    },
    FindObject {
        image_var: String, // 输入图片
        query: String, // 查找内容 (文本 regex 或 模板名称)
        output_var: String, // 输出坐标/区域变量
    },
    //操作
    Click {
        pos_idx: Option<u8>, // Deprecated?
        target_var: Option<String>, // 从变量获取坐标 (x, y) 或 (x, y, w, h)
        verify: Option<Vec<Step>>,
    },
    ClickLabelIdx {
        label_idx: u32,
        pos_idx: Option<u8>,
        verify: Option<Vec<Step>>,
    },
    ClickLabel {
        name: String,
        pos_idx: Option<u8>,
        verify: Option<Vec<Step>>,
    },
    ClickText {
        text: String,
        pos_idx: Option<u8>,
        verify: Option<Vec<Step>>,
    },
    ClickPoint {
        x: u32,
        y: u32,
        verify: Option<Vec<Step>>,
    },
    ClickPercent {
        x: f32,
        y: f32,
        verify: Option<Vec<Step>>,
    },
    //安卓
    SwipeDet {
        from: LabelType,
        to: LabelType,
        verify: Option<Vec<Step>>,
    },
    SwipeTxt {
        from: String,
        to: String,
        verify: Option<Vec<Step>>,
    },
    SwipePoint {
        from: Point<u16>,
        to: Point<u16>,
        verify: Option<Vec<Step>>,
    },
    SwipePercent {
        from: PointPercent,
        to: PointPercent,
        verify: Option<Vec<Step>>,
    },
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum LabelType {
    LabelIdx { idx: i32 },
    LabelName { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub struct PointPercent {
    x: f32,
    y: f32,
}

// DDD 仓储接口（同步，便于简化依赖；调用方如需并发可自行 spawn）
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
