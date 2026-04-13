use crate::domain::scripts::nodes::action::{Action, ClickMode, SwipeMode};
use crate::domain::scripts::nodes::data_handing::{DataHanding, FilterMode, VarValue};
use crate::domain::scripts::nodes::flow_control::{CompareOp, ConditionNode, FlowControl};
use crate::domain::scripts::point::{Point, PointF32, PointU16};
use crate::domain::scripts::nodes::task_control::{StateStatus, StateTarget, TaskControl};
use crate::domain::scripts::nodes::vision_node::VisionNode;
use crate::domain::scripts::script_decision::{Step, StepKind};
use crate::domain::vision::ocr_search::{OcrSearcher, SearchHit};
use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::adb_cli_local::adb_context::get_adb_ctx;
use crate::infrastructure::context::runtime_context::{SharedRuntimeContext, TaskState};
use crate::infrastructure::core::{HashMap, StepId};
use crate::infrastructure::devices::device_ctx::get_device_ctx;
use crate::infrastructure::ipc::message::RuntimeProgressPhase;
use crate::infrastructure::ipc::runtime_reporter::emit_progress_event;
use crate::infrastructure::scripts::script_error::{ExecuteResult, ScriptError};
use rhai::{Array, Dynamic, Engine, FLOAT, INT, Map, Scope};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::time::Duration;

const FILTER_ITEM_VAR: &str = "filter_item";
const FILTER_INDEX_VAR: &str = "filter_index";
const ITEM_VAR: &str = "item";
const ITEM_INDEX_VAR: &str = "item_index";
const MAX_LOOP_ITERATIONS: usize = 10_000;

#[derive(Debug)]
pub enum ControlFlow {
    Continue,
    Break,
    Next,
    Return,
}

#[derive(Debug, Clone, Copy)]
struct StepFrame {
    previous_step_id: Option<StepId>,
}

pub struct ScriptExecutor {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub runtime_ctx: SharedRuntimeContext,
    pub node_indices: HashMap<StepId, usize>,
}

impl ScriptExecutor {
    pub fn new(runtime_ctx: SharedRuntimeContext) -> Self {
        Self {
            engine: Engine::new(),
            scope: Scope::new(),
            runtime_ctx,
            node_indices: HashMap::new(),
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
                step.id
                    .map(|id| format!("[{}]", id))
                    .unwrap_or_default()
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
            StepKind::Action {
                cur_exec_num,
                max_exec_num,
                a,
            } => self.execute_action_step(*cur_exec_num, *max_exec_num, a).await,
            StepKind::DataHanding { a } => self.execute_data_handling_step(a).await,
            StepKind::FlowControl {
                cur_exec_num,
                max_exec_num,
                a,
            } => self.execute_flow_control_step(*cur_exec_num, *max_exec_num, a).await,
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

    async fn execute_action_step(
        &mut self,
        cur_exec_num: u32,
        max_exec_num: u32,
        action: &Action,
    ) -> ExecuteResult<ControlFlow> {
        if cur_exec_num > max_exec_num {
            return Ok(ControlFlow::Next);
        }

        self.before_action(action).await?;
        let result = self.dispatch_action(action).await;
        self.after_action(action).await?;
        result
    }

    async fn before_action(&mut self, _action: &Action) -> ExecuteResult<()> {
        Ok(())
    }

    async fn after_action(&mut self, _action: &Action) -> ExecuteResult<()> {
        Ok(())
    }

    async fn dispatch_action(&mut self, action: &Action) -> ExecuteResult<ControlFlow> {
        match action {
            Action::Capture { output_var } => {
                let image = Arc::new(get_device_ctx().get_screenshot().await.ok_or_else(|| {
                    Self::execute_error("action.capture", "获取设备截图失败".to_string())
                })?);
                let screen_size = (image.width(), image.height());
                self.set_runtime_var(output_var, Dynamic::from(image.clone())).await?;
                let mut ctx = self.runtime_ctx.write().await;
                ctx.observation.last_capture_image = Some(image);
                ctx.observation.screen_size = screen_size;
                ctx.observation.last_snapshot = None;
                ctx.observation.last_hits.clear();
                Ok(ControlFlow::Next)
            }
            Action::Click { mode } => self.execute_click(mode).await,
            Action::Swipe { duration, mode } => self.execute_swipe(mode, *duration).await,
            Action::Reboot => {
                get_adb_ctx().send_adb_cmd(&ADBCommand::Reboot);
                Ok(ControlFlow::Next)
            }
            Action::LaunchApp { pkg_name } => Err(Self::execute_error(
                "action.launchApp",
                format!(
                    "LaunchApp 当前只有 pkg_name={}，缺少 activity/launch target，暂不执行隐式启动",
                    pkg_name
                ),
            )),
            Action::StopApp { pkg_name } => {
                get_adb_ctx().send_adb_cmd(&ADBCommand::StopApp(pkg_name.clone()));
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_click(&mut self, mode: &ClickMode) -> ExecuteResult<ControlFlow> {
        let point = match mode {
            ClickMode::Point { p } => Self::to_device_point(p),
            ClickMode::Percent { p } => {
                let screen_size = self.ensure_screen_size().await?;
                Self::percent_to_device_point(p, screen_size)?
            }
            ClickMode::Txt { txt } => {
                return Err(Self::execute_error(
                    "action.click",
                    format!("文字点击尚未接入执行器动作适配: {}", txt.clone().unwrap_or_default()),
                ));
            }
            ClickMode::LabelIdx { idx } => {
                return Err(Self::execute_error(
                    "action.click",
                    format!("标签点击尚未接入执行器动作适配: {:?}", idx),
                ));
            }
        };
        get_adb_ctx().send_adb_cmd(&ADBCommand::Click(point));
        Ok(ControlFlow::Next)
    }

    async fn execute_swipe(
        &mut self,
        mode: &SwipeMode,
        duration: u64,
    ) -> ExecuteResult<ControlFlow> {
        let (from, to) = match mode {
            SwipeMode::Point { from, to } => {
                (Self::to_device_point(from), Self::to_device_point(to))
            }
            SwipeMode::Percent { from, to } => {
                let screen_size = self.ensure_screen_size().await?;
                (
                    Self::percent_to_device_point(from, screen_size)?,
                    Self::percent_to_device_point(to, screen_size)?,
                )
            }
            SwipeMode::Txt { from, to } => {
                return Err(Self::execute_error(
                    "action.swipe",
                    format!("文字滑动尚未接入执行器动作适配: {:?} -> {:?}", from, to),
                ));
            }
            SwipeMode::LabelIdx { from, to } => {
                return Err(Self::execute_error(
                    "action.swipe",
                    format!("标签滑动尚未接入执行器动作适配: {} -> {}", from, to),
                ));
            }
        };
        get_adb_ctx().send_adb_cmd(&ADBCommand::SwipeWithDuration(from, to, duration));
        Ok(ControlFlow::Next)
    }

    async fn ensure_screen_size(&self) -> ExecuteResult<(u32, u32)> {
        let cached = {
            let ctx = self.runtime_ctx.read().await;
            if ctx.observation.screen_size.0 > 0 && ctx.observation.screen_size.1 > 0 {
                return Ok(ctx.observation.screen_size);
            }
            ctx.observation
                .last_capture_image
                .as_ref()
                .map(|image| (image.width(), image.height()))
        };
        if let Some(screen_size) = cached {
            let mut ctx = self.runtime_ctx.write().await;
            ctx.observation.screen_size = screen_size;
            return Ok(screen_size);
        }
        let image = get_device_ctx().get_screenshot().await.ok_or_else(|| {
            Self::execute_error("action.screenSize", "获取屏幕尺寸失败".to_string())
        })?;
        let screen_size = (image.width(), image.height());
        let mut ctx = self.runtime_ctx.write().await;
        ctx.observation.last_capture_image = Some(Arc::new(image));
        ctx.observation.screen_size = screen_size;
        Ok(screen_size)
    }

    fn to_device_point(point: &PointU16) -> Point<u16> {
        Point::new(point.x, point.y)
    }

    fn percent_to_device_point(
        point: &PointF32,
        screen_size: (u32, u32),
    ) -> ExecuteResult<Point<u16>> {
        let (width, height) = screen_size;
        if width == 0 || height == 0 {
            return Err(Self::execute_error(
                "action.percentPoint",
                "屏幕尺寸无效，无法换算百分比坐标".to_string(),
            ));
        }
        let max_x = width.saturating_sub(1) as f32;
        let max_y = height.saturating_sub(1) as f32;
        let x = (point.x.clamp(0.0, 1.0) * max_x).round() as u16;
        let y = (point.y.clamp(0.0, 1.0) * max_y).round() as u16;
        Ok(Point::new(x, y))
    }

    async fn execute_flow_control_step(
        &mut self,
        cur_exec_num: u32,
        max_exec_num: u32,
        flow: &FlowControl,
    ) -> ExecuteResult<ControlFlow> {
        if cur_exec_num > max_exec_num {
            return Ok(ControlFlow::Next);
        }

        match flow {
            FlowControl::If {
                con,
                then,
                else_steps,
            } => {
                if self.evaluate_condition(con).await? {
                    self.execute(then).await
                } else if let Some(else_steps) = else_steps {
                    self.execute(else_steps).await
                } else {
                    Ok(ControlFlow::Next)
                }
            }
            FlowControl::While { con, flow } | FlowControl::For { con, flow } => {
                let mut iteration = 0usize;
                while self.evaluate_condition(con).await? {
                    iteration += 1;
                    if iteration > MAX_LOOP_ITERATIONS {
                        return Err(Self::execute_error(
                            "flow.loop",
                            format!("循环次数超过上限 {}", MAX_LOOP_ITERATIONS),
                        ));
                    }

                    match self.execute(flow).await? {
                        ControlFlow::Next => continue,
                        ControlFlow::Continue => continue,
                        ControlFlow::Break => break,
                        ControlFlow::Return => return Ok(ControlFlow::Return),
                    }
                }
                Ok(ControlFlow::Next)
            }
            FlowControl::Continue => Ok(ControlFlow::Continue),
            FlowControl::Break => Ok(ControlFlow::Break),
            FlowControl::WaitMs { ms } => {
                tokio::time::sleep(Duration::from_millis(*ms)).await;
                Ok(ControlFlow::Next)
            }
            FlowControl::Link { target } => Err(Self::execute_error(
                "flow.link",
                format!("跳转任务[{}]尚未接入调度器切换逻辑", target),
            )),
            FlowControl::AddPolicies { .. } => Err(Self::execute_error(
                "flow.addPolicies",
                "动态策略集拼装尚未接入运行时".to_string(),
            )),
            FlowControl::HandlePolicySet { .. } => Err(Self::execute_error(
                "flow.handlePolicySet",
                "策略集执行入口尚未接入执行器".to_string(),
            )),
            FlowControl::HandlePolicy { .. } => Err(Self::execute_error(
                "flow.handlePolicy",
                "策略执行入口尚未接入执行器".to_string(),
            )),
        }
    }

    async fn execute_data_handling_step(
        &mut self,
        data: &DataHanding,
    ) -> ExecuteResult<ControlFlow> {
        match data {
            DataHanding::SetVar { name, val, expr } => {
                let value = if let Some(expr) = expr.as_ref().filter(|value| !value.trim().is_empty())
                {
                    self.eval_dynamic(expr, "data.setVar")?
                } else if let Some(val) = val {
                    Self::var_value_to_dynamic(val)
                } else {
                    Dynamic::UNIT
                };
                self.set_runtime_var(name, value).await?;
                Ok(ControlFlow::Next)
            }
            DataHanding::GetVar { name, default_val } => {
                if self.read_runtime_var(name).await.is_none() {
                    if let Some(default_val) = default_val {
                        self.set_runtime_var(name, Self::var_value_to_dynamic(default_val))
                            .await?;
                    }
                }
                Ok(ControlFlow::Next)
            }
            DataHanding::Filter {
                input_var,
                out_name,
                mode,
                logic_expr,
                then_steps,
            } => {
                let Some(input) = self.read_runtime_var(input_var).await else {
                    self.set_runtime_var(out_name, Dynamic::from(Array::new())).await?;
                    return Ok(ControlFlow::Next);
                };

                let Some(items) = input.clone().try_cast::<Array>() else {
                    return Err(Self::execute_error(
                        "data.filter",
                        format!("输入变量[{}]不是数组，无法执行过滤", input_var),
                    ));
                };

                let mut output = Array::new();
                for (index, item) in items.into_iter().enumerate() {
                    self.scope.set_value(FILTER_ITEM_VAR, item.clone());
                    self.scope.set_value(ITEM_VAR, item.clone());
                    self.scope.set_value(FILTER_INDEX_VAR, index as i64);
                    self.scope.set_value(ITEM_INDEX_VAR, index as i64);

                    let matched = if logic_expr.trim().is_empty() {
                        true
                    } else {
                        self.eval_bool(logic_expr, "data.filter.logicExpr")?
                    };

                    if !matched {
                        continue;
                    }

                    if !then_steps.is_empty() {
                        match self.execute(then_steps).await? {
                            ControlFlow::Next => {}
                            ControlFlow::Continue => continue,
                            ControlFlow::Break => break,
                            ControlFlow::Return => return Ok(ControlFlow::Return),
                        }
                    }

                    match mode {
                        FilterMode::Filter => output.push(item),
                        FilterMode::Map => {
                            let current = self
                                .scope
                                .get_value::<Dynamic>(ITEM_VAR)
                                .unwrap_or_else(|| Dynamic::UNIT);
                            output.push(current);
                        }
                    }
                }

                self.set_runtime_var(out_name, Dynamic::from(output)).await?;
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_task_control_step(
        &mut self,
        task_control: &TaskControl,
    ) -> ExecuteResult<ControlFlow> {
        match task_control {
            TaskControl::SetState { target, status } => {
                self.set_state_value(target, status).await?;
                Ok(ControlFlow::Next)
            }
            TaskControl::GetState { target, status: _ } => Err(Self::execute_error(
                "taskControl.getState",
                format!(
                    "GetState 只应用于条件节点 TaskStatus，不应作为步骤执行[target={}]",
                    Self::state_target_label(target)
                ),
            )),
        }
    }

    async fn execute_vision_step(
        &mut self,
        vision: &VisionNode,
    ) -> ExecuteResult<ControlFlow> {
        match vision {
            VisionNode::VisionSearch {
                rule,
                out_var,
                then_steps,
            } => {
                let (hits, matched) = {
                    let ctx = self.runtime_ctx.read().await;
                    if let Some(snapshot) = ctx.observation.last_snapshot.as_ref() {
                        let searcher = OcrSearcher::new(std::slice::from_ref(rule));
                        let hits = searcher.search(snapshot);
                        let matched = rule.evaluate(&hits, &snapshot.det_items);
                        (hits, matched)
                    } else {
                        (Vec::new(), false)
                    }
                };

                {
                    let mut ctx = self.runtime_ctx.write().await;
                    ctx.observation.last_hits = hits.clone();
                }

                self.set_runtime_var(out_var, Self::search_hits_to_dynamic(&hits))
                    .await?;

                if matched && !then_steps.is_empty() {
                    return self.execute(then_steps).await;
                }

                Ok(ControlFlow::Next)
            }
        }
    }

    fn evaluate_condition<'a>(
        &'a mut self,
        condition: &'a ConditionNode,
    ) -> Pin<Box<dyn Future<Output = ExecuteResult<bool>> + 'a>> {
        Box::pin(async move {
            match condition {
                ConditionNode::RawExpr { expr } => self.eval_bool(expr, "condition.rawExpr"),
                ConditionNode::Group { op, items } => match op {
                    crate::domain::vision::ocr_search::LogicOp::And => {
                        for item in items {
                            if !self.evaluate_condition(item).await? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    crate::domain::vision::ocr_search::LogicOp::Or => {
                        for item in items {
                            if self.evaluate_condition(item).await? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }
                    crate::domain::vision::ocr_search::LogicOp::Not => {
                        if let Some(first) = items.first() {
                            Ok(!self.evaluate_condition(first).await?)
                        } else {
                            Ok(true)
                        }
                    }
                },
                ConditionNode::VarCompare {
                    var_name,
                    op,
                    value,
                } => {
                    let Some(lhs) = self.read_runtime_var(var_name).await else {
                        return Ok(false);
                    };
                    let rhs = Self::var_value_to_dynamic(value);
                    Ok(Self::compare_dynamic(&lhs, op, &rhs))
                }
                ConditionNode::TaskStatus { a } => self.match_state_status(a).await,
                ConditionNode::PolicyCondition { rule, .. } => {
                    let ctx = self.runtime_ctx.read().await;
                    Ok(ctx
                        .observation
                        .last_snapshot
                        .as_ref()
                        .map(|snapshot| rule.evaluate(snapshot))
                        .unwrap_or(false))
                }
                ConditionNode::ExecNumCompare { .. } => Err(Self::execute_error(
                    "condition.execNumCompare",
                    "执行次数条件尚未定义比较阈值，当前不执行隐式推断".to_string(),
                )),
                ConditionNode::ColorCompare { .. } => Err(Self::execute_error(
                    "condition.colorCompare",
                    "颜色比较尚未接入视觉颜色分析".to_string(),
                )),
                ConditionNode::PolicySetResult { .. } => Err(Self::execute_error(
                    "condition.policySetResult",
                    "策略结果条件尚未接入执行器".to_string(),
                )),
            }
        })
    }

    async fn set_state_value(
        &mut self,
        target: &StateTarget,
        status: &StateStatus,
    ) -> ExecuteResult<()> {
        let mut ctx = self.runtime_ctx.write().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx.execution.task_states.entry(*id).or_default();
                match status {
                    StateStatus::Enabled { value } => state.enabled_flag = *value,
                    StateStatus::Skip { value } => state.skip_flag = *value,
                    StateStatus::Done { value } => state.done_flag = *value,
                }
            }
            StateTarget::Policy { id } => {
                let state = ctx.execution.policy_states.entry(*id).or_default();
                match status {
                    StateStatus::Enabled { .. } => {
                        return Err(Self::execute_error(
                            "taskControl.setState",
                            format!("策略[{}]不支持 enabled 状态", id),
                        ));
                    }
                    StateStatus::Skip { value } => state.skip_flag = *value,
                    StateStatus::Done { value } => state.done_flag = *value,
                }
            }
        }
        Ok(())
    }

    async fn match_state_status(&mut self, task_control: &TaskControl) -> ExecuteResult<bool> {
        let (target, status) = match task_control {
            TaskControl::GetState { target, status }
            | TaskControl::SetState { target, status } => (target, status),
        };

        let ctx = self.runtime_ctx.read().await;
        match target {
            StateTarget::Task { id } => {
                let state = ctx.execution.task_states.get(id).cloned().unwrap_or_else(TaskState::default);
                Ok(match status {
                    StateStatus::Enabled { value } => state.enabled_flag == *value,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                })
            }
            StateTarget::Policy { id } => {
                let state = ctx.execution.policy_states.get(id).cloned().unwrap_or_default();
                Ok(match status {
                    StateStatus::Enabled { .. } => false,
                    StateStatus::Skip { value } => state.skip_flag == *value,
                    StateStatus::Done { value } => state.done_flag == *value,
                })
            }
        }
    }

    async fn set_runtime_var(&mut self, name: &str, value: Dynamic) -> ExecuteResult<()> {
        if name.trim().is_empty() {
            return Ok(());
        }

        let root = name
            .split('.')
            .next()
            .unwrap_or(name)
            .trim()
            .to_string();
        let root_value = {
            let mut ctx = self.runtime_ctx.write().await;
            ctx.execution.var_map.insert(name.to_string(), value);
            Self::build_scope_root_value(&ctx.execution.var_map, &root)
        };
        self.scope.set_value(root, root_value);
        Ok(())
    }

    async fn read_runtime_var(&self, name: &str) -> Option<Dynamic> {
        {
            let ctx = self.runtime_ctx.read().await;
            if let Some(value) = ctx.execution.var_map.get(name) {
                return Some(value.clone());
            }
        }

        if name.contains('.') {
            None
        } else {
            self.scope.get_value::<Dynamic>(name)
        }
    }

    fn build_scope_root_value(var_map: &HashMap<String, Dynamic>, root: &str) -> Dynamic {
        let nested_prefix = format!("{}.", root);
        let mut nested = Map::new();

        for (key, value) in var_map {
            if let Some(suffix) = key.strip_prefix(&nested_prefix) {
                let partial = Self::build_nested_map(suffix, value.clone());
                Self::merge_map(&mut nested, partial);
            }
        }

        if nested.is_empty() {
            var_map.get(root).cloned().unwrap_or(Dynamic::UNIT)
        } else {
            Dynamic::from(nested)
        }
    }

    fn build_nested_map(path: &str, value: Dynamic) -> Map {
        let mut current = value;
        for segment in path.split('.').rev() {
            let mut map = Map::new();
            map.insert(segment.into(), current);
            current = Dynamic::from(map);
        }

        current.try_cast::<Map>().unwrap_or_default()
    }

    fn merge_map(target: &mut Map, source: Map) {
        for (key, value) in source {
            if let Some(existing) = target.get_mut(&key) {
                let left = existing.clone().try_cast::<Map>();
                let right = value.clone().try_cast::<Map>();
                match (left, right) {
                    (Some(mut left_map), Some(right_map)) => {
                        Self::merge_map(&mut left_map, right_map);
                        *existing = Dynamic::from(left_map);
                    }
                    _ => *existing = value,
                }
            } else {
                target.insert(key, value);
            }
        }
    }

    fn var_value_to_dynamic(value: &VarValue) -> Dynamic {
        match value {
            VarValue::Int(value) => Dynamic::from_int((*value).into()),
            VarValue::Float(value) => Dynamic::from_float((*value).into()),
            VarValue::Bool(value) => Dynamic::from_bool(*value),
            VarValue::String(value) => Dynamic::from(value.clone()),
        }
    }

    fn search_hits_to_dynamic(hits: &[SearchHit]) -> Dynamic {
        let mut array = Array::new();
        for hit in hits {
            let mut item = Map::new();
            item.insert("pattern".into(), Dynamic::from(hit.pattern.clone()));
            item.insert("ocrIndex".into(), Dynamic::from_int(hit.ocr_index as INT));
            item.insert("text".into(), Dynamic::from(hit.ocr_item.txt.clone()));
            array.push(Dynamic::from(item));
        }
        Dynamic::from(array)
    }

    fn compare_dynamic(lhs: &Dynamic, op: &CompareOp, rhs: &Dynamic) -> bool {
        match op {
            CompareOp::Contains => Self::dynamic_to_string(lhs)
                .zip(Self::dynamic_to_string(rhs))
                .map(|(lhs, rhs)| lhs.contains(&rhs))
                .unwrap_or(false),
            CompareOp::NotContains => Self::dynamic_to_string(lhs)
                .zip(Self::dynamic_to_string(rhs))
                .map(|(lhs, rhs)| !lhs.contains(&rhs))
                .unwrap_or(false),
            CompareOp::Eq => Self::dynamic_eq(lhs, rhs),
            CompareOp::Ne => !Self::dynamic_eq(lhs, rhs),
            CompareOp::Lt => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs < rhs)
                .unwrap_or(false),
            CompareOp::Le => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs <= rhs)
                .unwrap_or(false),
            CompareOp::Gt => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs > rhs)
                .unwrap_or(false),
            CompareOp::Ge => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs >= rhs)
                .unwrap_or(false),
        }
    }

    fn dynamic_eq(lhs: &Dynamic, rhs: &Dynamic) -> bool {
        if let (Some(lhs), Some(rhs)) = (lhs.clone().try_cast::<bool>(), rhs.clone().try_cast::<bool>())
        {
            return lhs == rhs;
        }
        if let (Some(lhs), Some(rhs)) = (Self::dynamic_to_number(lhs), Self::dynamic_to_number(rhs))
        {
            return (lhs - rhs).abs() < f64::EPSILON;
        }
        if let (Some(lhs), Some(rhs)) = (Self::dynamic_to_string(lhs), Self::dynamic_to_string(rhs))
        {
            return lhs == rhs;
        }
        false
    }

    fn dynamic_to_number(value: &Dynamic) -> Option<f64> {
        if let Some(value) = value.clone().try_cast::<INT>() {
            return Some(value as f64);
        }
        if let Some(value) = value.clone().try_cast::<FLOAT>() {
            return Some(value as f64);
        }
        if let Some(value) = value.clone().try_cast::<String>() {
            return value.parse::<f64>().ok();
        }
        None
    }

    fn dynamic_to_string(value: &Dynamic) -> Option<String> {
        if let Some(value) = value.clone().try_cast::<String>() {
            return Some(value);
        }
        if let Some(value) = value.clone().try_cast::<bool>() {
            return Some(value.to_string());
        }
        if let Some(value) = value.clone().try_cast::<INT>() {
            return Some(value.to_string());
        }
        if let Some(value) = value.clone().try_cast::<FLOAT>() {
            return Some(value.to_string());
        }
        None
    }

    fn eval_bool(&mut self, expr: &str, step_type: &str) -> ExecuteResult<bool> {
        self.engine
            .eval_expression_with_scope::<bool>(&mut self.scope, expr)
            .map_err(|error| Self::execute_error(step_type, error.to_string()))
    }

    fn eval_dynamic(&mut self, expr: &str, step_type: &str) -> ExecuteResult<Dynamic> {
        self.engine
            .eval_expression_with_scope::<Dynamic>(&mut self.scope, expr)
            .map_err(|error| Self::execute_error(step_type, error.to_string()))
    }

    fn state_target_label(target: &StateTarget) -> String {
        match target {
            StateTarget::Task { id } => format!("task:{}", id),
            StateTarget::Policy { id } => format!("policy:{}", id),
        }
    }

    fn execute_error(step_type: &str, e: String) -> ScriptError {
        ScriptError::ExecuteErr {
            step_type: step_type.to_string(),
            e,
        }
    }
}
