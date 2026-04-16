use crate::domain::scripts::nodes::action::{Action, ClickMode, SwipeMode};
use crate::domain::scripts::nodes::data_handing::{DataHanding, FilterMode, VarValue};
use crate::domain::scripts::nodes::flow_control::{
    CompareOp, ConditionNode, FlowControl, PolicySetResultCompareOp, PolicySetResultField,
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
use crate::domain::scripts::script_task::ScriptTaskTable;
use crate::domain::vision::ocr_search::{OcrSearcher, SearchHit, VisionSnapshot};
use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult};
use crate::infrastructure::context::runtime_context::{SharedRuntimeContext, TaskState};
use crate::infrastructure::core::{
    ExecutionId, HashMap, PolicyGroupId, PolicyId, PolicySetId, ScheduleId, StepId, TaskId,
};
use crate::infrastructure::devices::device_ctx::get_device_ctx;
use crate::infrastructure::ipc::message::{
    RuntimeLifecyclePhase, RuntimeProgressPhase, SessionCheckpointReason, TimeoutAction,
};
use crate::infrastructure::ipc::runtime_reporter::{
    emit_lifecycle_event, emit_progress_event,
};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::script_error::{ExecuteResult, ScriptError};
use crate::infrastructure::session::recovery_checkpoint_store::prepare_and_persist_checkpoint;
use crate::infrastructure::session::runtime_session::{
    get_runtime_execution_policy, get_script_bundle_snapshot,
};
use image::{DynamicImage, RgbaImage};
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{Array, Dynamic, Engine, Map, Scope, FLOAT, INT};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::hash::Hasher;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;
use twox_hash::XxHash3_64;

include!("executor/action.rs");
include!("executor/policy.rs");
include!("executor/flow.rs");
include!("executor/runtime.rs");

#[cfg(test)]
mod tests;

const FILTER_ITEM_VAR: &str = "filter_item";
const FILTER_INDEX_VAR: &str = "filter_index";
const ITEM_VAR: &str = "item";
const ITEM_INDEX_VAR: &str = "item_index";
const MAX_LOOP_ITERATIONS: usize = 10_000;

#[derive(Debug)]
pub enum ControlFlow {
    Continue,
    Break,
    Link(TaskId),
    Next,
    Return,
}

#[derive(Debug, Clone, Copy)]
struct StepFrame {
    previous_step_id: Option<StepId>,
}

#[derive(Debug)]
struct PolicyBundle {
    policies: Vec<PolicyTable>,
    policy_groups: Vec<PolicyGroupTable>,
    policy_sets: Vec<PolicySetTable>,
    group_policies: Vec<GroupPolicyRelation>,
    set_groups: Vec<SetGroupRelation>,
}

#[derive(Debug, Clone)]
struct PolicyCandidate {
    policy_set_id: Option<PolicySetId>,
    policy_group_id: Option<PolicyGroupId>,
    policy: PolicyTable,
}

#[derive(Debug, Default, Clone)]
struct ActivePolicyRoundTrace {
    page_fingerprints: Vec<String>,
    action_signatures: Vec<String>,
    actions: Vec<PolicyActionTrace>,
}

#[derive(Debug, Clone)]
struct ActionProgressProbe {
    page_fingerprint: String,
    action_signature: String,
    task_id: Option<TaskId>,
    step_id: Option<StepId>,
    stagnant_since: Instant,
    notified: bool,
}

pub struct ScriptExecutor {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub runtime_ctx: SharedRuntimeContext,
    pub node_indices: HashMap<StepId, usize>,
    active_policy_round: Option<ActivePolicyRoundTrace>,
    last_progress_probe: Option<ActionProgressProbe>,
}

impl ScriptExecutor {
    pub fn new(runtime_ctx: SharedRuntimeContext) -> Self {
        Self {
            engine: Engine::new(),
            scope: Scope::new(),
            runtime_ctx,
            node_indices: HashMap::new(),
            active_policy_round: None,
            last_progress_probe: None,
        }
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
                return Ok(ControlFlow::Next);
            }

            let frame = self.enter_step(step).await;
            let result = self.execute_step_inner(step).await;
            self.leave_step(frame).await;
            result
        })
    }

    async fn enter_step(&mut self, step: &Step) -> StepFrame {
        let (previous_step_id, assignment_id, script_id, task_id) = {
            let mut ctx = self.runtime_ctx.write().await;
            let previous_step_id = ctx.execution.current_step_id;
            ctx.execution.current_step_id = step.id;
            (
                previous_step_id,
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
            Some(format!(
                "执行步骤{}",
                step.id.map(|id| format!("[{}]", id)).unwrap_or_default()
            )),
        );

        if let Some(id) = step.id {
            let idx = self.get_node_index(&id);
            self.scope.set_value(format!("idx_{}", id), idx as i64);
        }

        StepFrame { previous_step_id }
    }

    async fn leave_step(&mut self, frame: StepFrame) {
        let mut ctx = self.runtime_ctx.write().await;
        ctx.execution.current_step_id = frame.previous_step_id;
    }

    async fn execute_step_inner(&mut self, step: &Step) -> ExecuteResult<ControlFlow> {
        match &step.kind {
            StepKind::Sequence { steps } => self.execute_sequence(steps).await,
            StepKind::Action { exec_max, a } => self.execute_action_step(step.id, *exec_max, a).await,
            StepKind::DataHanding { a } => self.execute_data_handling_step(a).await,
            StepKind::FlowControl { a } => self.execute_flow_control_step(a).await,
            StepKind::TaskControl { a } => self.execute_task_control_step(a).await,
            StepKind::Vision { a } => self.execute_vision_step(a).await,
        }
    }

    async fn execute_sequence(&mut self, steps: &[Step]) -> ExecuteResult<ControlFlow> {
        for step in steps {
            let flow = self.execute_step(step).await?;
            if !matches!(flow, ControlFlow::Next) {
                return Ok(flow);
            }
        }

        Ok(ControlFlow::Next)
    }
}
