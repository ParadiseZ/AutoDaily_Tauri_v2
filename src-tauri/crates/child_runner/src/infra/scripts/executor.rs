use crate::infra::context::runtime_context::{
    PolicyCandidate, PolicyGroupBindingOp, PolicyGroupBindingSource, PolicySetBindingOp,
    PolicySetBindingSource, RuntimeContext, SharedRuntimeContext, TaskState,
};
use crate::infra::context::{
    PolicyActionKind, PolicyActionSource, PolicyActionTarget, PolicyActionTargetRole,
    PolicyActionTrace, PolicyExecutionResult, PolicyExecutionRound,
};
use crate::infra::ipc::runtime_reporter::{emit_lifecycle_event, emit_progress_event};
use crate::infra::logging::log_trait::Log;
use crate::infra::session::runtime_session::{
    get_runtime_execution_policy, get_runtime_session_store, get_script_bundle_snapshot,
};
use ad_kernel::{
    Point,
    ids::{
        AccountId, AssignmentId, DeviceId, ExecutionId, PolicyGroupId, PolicyId, PolicySetId,
        ScriptId, StepId, TaskId, TemplateId,
    },
};
use domain_device::{DeviceOperation, TimeoutAction};
use domain_script::{
    Action, ClickMode, ColorCompareMethod, ColorRgb, CompareOp, ConditionNode,
    CurrentTaskCondition, DataHanding, ExecuteResult, FilterMode, FlowControl, PointF32, PointU16,
    PolicySetResultCompareOp, PolicySetResultField, RegionPoint, ScriptError, StateStatus,
    StateTarget, Step, StepKind, SwipeMode, SwipeTarget, TaskControl, VarValue, VisionNode,
};
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptProfile, ScriptTaskProfile,
};
use domain_script::{ScriptVariableCatalog, ScriptVariableDef, ScriptVariableNamespace};
use domain_vision::{BoundingBox, DetResult, OcrResult};
use domain_vision::{OcrSearcher, VisionSnapshot};
use domain_vision::{
    RelativeAnchorType, RelativeDirection, RelativeTargetKind, SearchHit, SearchRule,
    VisionLayoutItem, VisionLayoutSource,
};
use image::RgbaImage;
use infra_device_runtime::get_device_ctx;
use infra_vision::OcrService;
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{AST, Array, Dynamic, Engine, EvalAltResult, FLOAT, INT, Map, Scope};
use runner_protocol::message::{
    RunTarget, RuntimeLifecyclePhase, RuntimeProgressPhase, ScriptBundleSnapshot,
};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map as JsonMap, Value, json};
use std::collections::HashMap;
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
pub(crate) enum ControlFlow {
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
    policies: Vec<PolicyProfile>,
    policy_groups: Vec<PolicyGroupProfile>,
    policy_sets: Vec<PolicySetProfile>,
    group_policies: Vec<PolicyGroupPolicyLink>,
    set_groups: Vec<PolicySetGroupLink>,
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

pub(crate) struct ScriptExecutor {
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
    pub(crate) fn new(runtime_ctx: SharedRuntimeContext) -> Self {
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

    pub(crate) fn reset_node_indices(&mut self) {
        self.node_indices.clear();
    }

    pub(crate) fn get_node_index(&self, id: &StepId) -> usize {
        self.node_indices.get(id).cloned().unwrap_or(0)
    }

    fn stop_requested_flow() -> Option<ControlFlow> {
        crate::infra::context::runtime_control::stop_requested().then_some(ControlFlow::StopScript)
    }

    pub(crate) async fn execute(&mut self, steps: &[Step]) -> ExecuteResult<ControlFlow> {
        for step in steps {
            if let Some(flow) = Self::stop_requested_flow() {
                return Ok(flow);
            }
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
            if let Some(flow) = Self::stop_requested_flow() {
                return Ok(flow);
            }
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
        if let Some(flow) = Self::stop_requested_flow() {
            return Ok(flow);
        }
        if let Some(flow) = self.try_execute_action_sequence(steps).await? {
            return Ok(flow);
        }

        for step in steps {
            if let Some(flow) = Self::stop_requested_flow() {
                return Ok(flow);
            }
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
