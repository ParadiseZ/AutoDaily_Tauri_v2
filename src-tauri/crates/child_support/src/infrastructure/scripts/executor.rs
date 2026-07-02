use crate::constant::table_name::SCRIPT_TIME_TEMPLATE_VALUES_TABLE;
use crate::domain::scripts::nodes::action::{Action, ClickMode, SwipeMode, SwipeTarget};
use crate::domain::scripts::nodes::data_handing::{
    ColorCompareMethod, ColorRgb, DataHanding, FilterMode, RegionPoint, VarValue,
};
use crate::domain::scripts::nodes::flow_control::{
    CompareOp, ConditionNode, CurrentTaskRule, FlowControl, PolicySetResultCompareOp,
    PolicySetResultField,
};
use crate::domain::scripts::nodes::policy_execution::{
    PolicyActionKind, PolicyActionSource, PolicyActionTarget, PolicyActionTargetRole,
    PolicyActionTrace, PolicyExecutionResult, PolicyExecutionRound,
};
use crate::domain::scripts::nodes::task_control::{StateStatus, StateTarget, TaskControl};
use crate::domain::scripts::nodes::vision_node::VisionNode;
use crate::domain::scripts::point::{Point, PointF32, PointU16};
use crate::domain::scripts::policy::{
    GroupPolicyRelation, PolicyGroupTable, PolicySetTable, PolicyTable, SetGroupRelation,
};
use crate::domain::scripts::script_decision::{Step, StepKind};
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::domain::scripts::script_variable::{
    ScriptVariableCatalog, ScriptVariableDef, ScriptVariableNamespace,
};
use crate::domain::vision::ocr_search::{
    OcrSearcher, RelativeAnchorType, RelativeDirection, RelativeTargetKind, SearchHit, SearchRule,
    VisionLayoutItem, VisionLayoutSource, VisionSnapshot,
};
use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult};
use crate::infrastructure::context::runtime_context::{
    PolicyCandidate, PolicyGroupBindingOp, PolicyGroupBindingSource, PolicySetBindingOp,
    PolicySetBindingSource, SharedRuntimeContext, TaskState,
};
use crate::infrastructure::core::{
    AccountId, AssignmentId, DeviceId, ExecutionId, HashMap, PolicyGroupId, PolicyId, PolicySetId,
    ScriptId, ScriptTemplateValueId, StepId, TaskId, TemplateId,
};
use crate::infrastructure::db::get_pool;
use crate::infrastructure::devices::device_ctx::get_device_ctx;
use crate::infrastructure::devices::device_runtime::DeviceOperation;
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeLifecyclePhase, RuntimeProgressPhase, TimeoutAction,
};
use crate::infrastructure::ipc::runtime_reporter::{emit_lifecycle_event, emit_progress_event};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::script_error::{ExecuteResult, ScriptError};
use crate::infrastructure::session::runtime_session::{
    get_runtime_execution_policy, get_script_bundle_snapshot,
};
use crate::infrastructure::vision::ocr_service::OcrService;
use image::{DynamicImage, RgbaImage};
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{Array, Dynamic, Engine, EvalAltResult, Map, Scope, AST, FLOAT, INT};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Map as JsonMap, Value};
use sqlx::types::Json as SqlJson;
use std::future::Future;
use std::hash::Hasher;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::Duration;
use twox_hash::XxHash3_64;

const DEVICE_EXTERNAL_TIMEOUT_MS: u64 = 5_000;
const VISION_INFERENCE_TIMEOUT_MS: u64 = 10_000;

include!("executor/action_target_resolver.rs");
include!("executor/action_planner.rs");
include!("executor/action_plan_click.rs");
include!("executor/action_plan_swipe.rs");
include!("executor/sequence_operation_compiler.rs");
include!("executor/action_trace.rs");
include!("executor/action_dispatcher.rs");
include!("executor/action_observer.rs");
include!("executor/action_business.rs");
include!("executor/action_capture_context.rs");
include!("executor/action_target_matcher.rs");
include!("executor/action_geometry.rs");
include!("executor/action_runtime_results.rs");
include!("executor/policy_bundle.rs");
include!("executor/policy_debug.rs");
include!("executor/policy_runner.rs");
include!("executor/policy.rs");
include!("executor/flow_wait.rs");
include!("executor/flow_condition.rs");
include!("executor/flow_data_region.rs");
include!("executor/flow_data.rs");
include!("executor/flow_data_relative.rs");
include!("executor/flow_data_color.rs");
include!("executor/flow_task_vision.rs");
include!("executor/flow.rs");
include!("executor/runtime.rs");
include!("executor/rhai_bridge.rs");

#[cfg(test)]
mod tests;

const FILTER_ITEM_VAR: &str = "filter_item";
const FILTER_INDEX_VAR: &str = "filter_index";
const ITEM_VAR: &str = "item";
const ITEM_INDEX_VAR: &str = "item_index";
const MAX_LOOP_ITERATIONS: usize = 10_000;
const WAIT_TIMEOUT_CHECK_SLICE_MS: u64 = 500;

#[derive(Debug)]
pub enum ControlFlow {
    Continue,
    Break,
    Link(TaskId),
    Next,
    Return,
    StopScript,
}

#[derive(Debug, Clone)]
struct StepFrame {
    previous_step_id: Option<StepId>,
    previous_step_name: Option<String>,
}

#[derive(Debug)]
struct PolicyBundle {
    policies: Vec<PolicyTable>,
    policy_groups: Vec<PolicyGroupTable>,
    policy_sets: Vec<PolicySetTable>,
    group_policies: Vec<GroupPolicyRelation>,
    set_groups: Vec<SetGroupRelation>,
}

#[derive(Debug, Default, Clone)]
struct ActivePolicyRoundTrace {
    page_fingerprints: Vec<String>,
    action_signatures: Vec<String>,
    actions: Vec<PolicyActionTrace>,
}

#[derive(Debug, Clone)]
struct ActivePolicyContext {
    policy_id: PolicyId,
    policy_name: String,
    base_click_pos: u16,
}

#[derive(Debug, Clone, Copy)]
struct RegionRect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

#[derive(Debug, Clone)]
struct ProgressProbe {
    page_fingerprint: Option<String>,
    evidence_signature: String,
    task_id: Option<TaskId>,
    step_id: Option<StepId>,
    stagnant_since: Instant,
    notified: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct OklabColor {
    l: f32,
    a: f32,
    b: f32,
}

#[derive(Debug, Clone)]
struct ColorCluster {
    center: OklabColor,
    count: usize,
    mean_distance: f32,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RuntimeTemplateValuesSnapshot {
    #[serde(default)]
    variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
enum QueuedRhaiOp {
    Step(Step),
    LinkTaskByName {
        task_name: String,
    },
    SetTaskStateByName {
        task_name: String,
        status: StateStatus,
    },
    SetPolicyStateByName {
        policy_name: String,
        status: StateStatus,
    },
    AddPoliciesByName {
        source_name: String,
        target_name: String,
        top: bool,
        reverse: bool,
    },
    RemovePoliciesByName {
        source_name: String,
        target_name: String,
    },
    BindPolicyGroupByName {
        source_name: String,
        target_name: String,
        top: bool,
        reverse: bool,
    },
    RemovePolicyGroupByName {
        source_name: String,
        target_name: String,
    },
    AddPolicyGroupsByName {
        source_name: String,
        target_name: String,
        top: bool,
        reverse: bool,
    },
    UnloadPolicyGroupByName {
        source_name: String,
        target_name: String,
    },
    BindPolicyByName {
        source_name: String,
        target_name: String,
        top: bool,
        reverse: bool,
    },
    UnloadPolicyByName {
        source_name: String,
        target_name: String,
    },
    HandlePolicySetByName {
        target_names: Vec<String>,
        det_input_var: String,
        ocr_input_var: String,
        search_hits_var: String,
        out_var: String,
    },
    HandlePolicyByName {
        target_names: Vec<String>,
        input_var: String,
        out_var: String,
    },
}

#[derive(Debug, Clone)]
struct QueuedRhaiStep {
    helper_name: &'static str,
    op: QueuedRhaiOp,
}

pub struct ScriptExecutor {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub runtime_ctx: SharedRuntimeContext,
    pub node_indices: HashMap<StepId, usize>,
    compiled_rhai_blocks: HashMap<u64, AST>,
    rhai_step_queue: Arc<StdMutex<Vec<Vec<QueuedRhaiStep>>>>,
    active_policy_round: Option<ActivePolicyRoundTrace>,
    active_policy_context: Option<ActivePolicyContext>,
    last_progress_probe: Option<ProgressProbe>,
}

impl ScriptExecutor {
    pub fn new(runtime_ctx: SharedRuntimeContext) -> Self {
        let rhai_step_queue = Arc::new(StdMutex::new(Vec::new()));
        let mut executor = Self {
            engine: Engine::new(),
            scope: Scope::new(),
            runtime_ctx,
            node_indices: HashMap::new(),
            compiled_rhai_blocks: HashMap::new(),
            rhai_step_queue,
            active_policy_round: None,
            active_policy_context: None,
            last_progress_probe: None,
        };
        executor.register_rhai_step_helpers();
        executor
    }

    pub fn reset_node_indices(&mut self) {
        self.node_indices.clear();
    }

    pub fn get_node_index(&self, id: &StepId) -> usize {
        self.node_indices.get(id).cloned().unwrap_or(0)
    }

    pub fn set_node_index(&mut self, id: &StepId, val: usize) {
        self.node_indices.insert(*id, val);
    }

    pub fn inc_node_index(&mut self, id: &StepId, amount: usize) {
        let current = self.get_node_index(id);
        self.set_node_index(id, current + amount);
    }

    pub fn reset_scope(&mut self) {
        self.scope.clear();
    }

    pub async fn execute(&mut self, steps: &[Step]) -> ExecuteResult<ControlFlow> {
        for step in steps {
            match self.execute_step(step).await? {
                ControlFlow::Next => continue,
                ControlFlow::Continue => return Ok(ControlFlow::Continue),
                ControlFlow::Break => return Ok(ControlFlow::Break),
                ControlFlow::Link(target) => return Ok(ControlFlow::Link(target)),
                ControlFlow::Return => return Ok(ControlFlow::Return),
                ControlFlow::StopScript => return Ok(ControlFlow::StopScript),
            }
        }
        Ok(ControlFlow::Next)
    }

    fn execute_step<'a>(
        &'a mut self,
        step: &'a Step,
    ) -> Pin<Box<dyn Future<Output = ExecuteResult<ControlFlow>> + 'a>> {
        Box::pin(async move {
            if step.skip_flag {
                let step_name = self.resolve_step_display_name(step).await;
                self.log_step_debug("skip", step, &step_name, Some("skip_flag=true"));
                return Ok(ControlFlow::Next);
            }

            let step_name = self.resolve_step_display_name(step).await;
            self.log_step_debug("enter", step, &step_name, None);
            let frame = self.enter_step(step).await;
            let result = self.execute_step_inner(step).await;
            self.leave_step(frame).await;
            match &result {
                Ok(flow) => self.log_step_debug(
                    "leave",
                    step,
                    &step_name,
                    Some(Self::describe_control_flow(flow)),
                ),
                Err(error) => {
                    self.log_step_debug("error", step, &step_name, Some(&error.to_string()))
                }
            }
            result
        })
    }

    async fn enter_step(&mut self, step: &Step) -> StepFrame {
        let step_name = self.resolve_step_display_name(step).await;
        let (previous_step_id, previous_step_name, assignment_id, script_id, task_id) = {
            let mut ctx = self.runtime_ctx.write().await;
            let previous_step_id = ctx.execution.current_step_id;
            let previous_step_name = ctx.execution.current_step_name.clone();
            ctx.execution.current_step_id = step.id;
            ctx.execution.current_step_name = Some(step_name.clone());
            (
                previous_step_id,
                previous_step_name,
                ctx.execution.current_assignment_id,
                Some(ctx.execution.script_id),
                ctx.execution.current_task.as_ref().map(|task| task.id),
            )
        };

        emit_progress_event(
            RuntimeProgressPhase::Executing,
            assignment_id,
            script_id,
            task_id,
            step.id,
            Some(format!("开始执行步骤: {}", step_name)),
        );

        if let Some(id) = step.id {
            let idx = self.get_node_index(&id);
            self.scope.set_value(format!("idx_{}", id), idx as i64);
        }

        StepFrame {
            previous_step_id,
            previous_step_name,
        }
    }

    async fn leave_step(&mut self, frame: StepFrame) {
        let mut ctx = self.runtime_ctx.write().await;
        ctx.execution.current_step_id = frame.previous_step_id;
        ctx.execution.current_step_name = frame.previous_step_name;
    }

    async fn execute_step_inner(&mut self, step: &Step) -> ExecuteResult<ControlFlow> {
        match &step.kind {
            StepKind::Sequence { steps } => self.execute_sequence(steps).await,
            StepKind::Action { exec_max, a } => {
                self.execute_action_step(step.id, step.label.as_deref(), *exec_max, a)
                    .await
            }
            StepKind::DataHanding { a } => self.execute_data_handling_step(a).await,
            StepKind::FlowControl { a } => self.execute_flow_control_step(a).await,
            StepKind::TaskControl { a } => self.execute_task_control_step(a).await,
            StepKind::Vision { a } => self.execute_vision_step(a).await,
        }
    }

    async fn execute_sequence(&mut self, steps: &[Step]) -> ExecuteResult<ControlFlow> {
        if let Some(flow) = self.try_execute_action_sequence(steps).await? {
            return Ok(flow);
        }

        for step in steps {
            let flow = self.execute_step(step).await?;
            if !matches!(flow, ControlFlow::Next) {
                return Ok(flow);
            }
        }

        Ok(ControlFlow::Next)
    }

    fn describe_step_kind(step: &Step) -> &'static str {
        match &step.kind {
            StepKind::Sequence { .. } => "sequence",
            StepKind::Action { .. } => "action",
            StepKind::DataHanding { .. } => "dataHanding",
            StepKind::FlowControl { .. } => "flowControl",
            StepKind::TaskControl { .. } => "taskControl",
            StepKind::Vision { .. } => "vision",
        }
    }

    fn describe_control_flow(flow: &ControlFlow) -> &'static str {
        match flow {
            ControlFlow::Continue => "continue",
            ControlFlow::Break => "break",
            ControlFlow::Link(_) => "link",
            ControlFlow::Next => "next",
            ControlFlow::Return => "return",
            ControlFlow::StopScript => "stopScript",
        }
    }

    fn log_step_debug(&self, stage: &str, step: &Step, step_name: &str, detail: Option<&str>) {
        let detail = detail
            .map(|value| format!(", detail={}", value))
            .unwrap_or_default();
        Log::debug(&format!(
            "[ executor ] step.{}: kind={}, label={}{}",
            stage,
            Self::describe_step_kind(step),
            step_name,
            detail
        ));
    }
}
