use crate::infra::context::runtime_context::{RuntimeContext, get_runtime_ctx, init_runtime_ctx};
use crate::infra::context::runtime_control::clear_stop_request;
use crate::infra::context::{PolicyGroupBindingSource, PolicySetBindingSource};
use crate::infra::scripts::scheduler::ScriptScheduler;
use crate::infra::session::runtime_session::{clear_runtime_session, replace_runtime_session};
use ad_kernel::ids::{AssignmentId, DeviceId, DispatchId, SessionId, TaskId, UuidV7};
use domain_device::{DeviceOperation, TimeoutAction};
use domain_vision::{DetResult, OcrResult, VisionSnapshot, VisionTextCacheRuntimeConfig};
use image::RgbaImage;
use infra_vision::OcrService;
use runner_protocol::message::{
    DispatchKind, DispatchSource, RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem,
    RuntimeSessionSnapshot, ScriptBundleSnapshot,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

static TEST_RUN_MUTEX: OnceLock<Arc<Mutex<()>>> = OnceLock::new();
static TEST_RUNTIME_INITIALIZED: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestVisionFrame {
    #[serde(default)]
    pub det_results: Vec<DetResult>,
    #[serde(default)]
    pub ocr_results: Vec<OcrResult>,
}

impl TestVisionFrame {
    fn into_snapshot(self, grid_size: u16) -> Result<VisionSnapshot, String> {
        VisionSnapshot::new(self.det_results, grid_size)?.with_ocr_results(self.ocr_results)
    }
}

pub struct TestTaskRunRequest {
    pub bundle: ScriptBundleSnapshot,
    pub task_id: TaskId,
    pub screenshots: Vec<RgbaImage>,
    pub capture_vision_frames: Vec<TestVisionFrame>,
    pub detect_vision_frames: Vec<TestVisionFrame>,
    pub ocr_vision_frames: Vec<TestVisionFrame>,
    pub use_real_vision: bool,
    pub template_values_json: Option<String>,
}

pub(crate) struct TestRuntimeHooks {
    use_real_vision: bool,
    operations: Mutex<Vec<DeviceOperation>>,
    screenshots: Mutex<VecDeque<RgbaImage>>,
    capture_frames: Mutex<VecDeque<TestVisionFrame>>,
    detect_frames: Mutex<VecDeque<TestVisionFrame>>,
    ocr_frames: Mutex<VecDeque<TestVisionFrame>>,
}

impl TestRuntimeHooks {
    fn new(request: &mut TestTaskRunRequest) -> Self {
        Self {
            use_real_vision: request.use_real_vision,
            operations: Mutex::new(Vec::new()),
            screenshots: Mutex::new(request.screenshots.drain(..).collect()),
            capture_frames: Mutex::new(request.capture_vision_frames.drain(..).collect()),
            detect_frames: Mutex::new(request.detect_vision_frames.drain(..).collect()),
            ocr_frames: Mutex::new(request.ocr_vision_frames.drain(..).collect()),
        }
    }

    pub(crate) fn uses_real_vision(&self) -> bool {
        self.use_real_vision
    }

    pub(crate) async fn record_operation(&self, operation: DeviceOperation) -> Result<(), String> {
        self.operations.lock().await.push(operation);
        Ok(())
    }

    pub(crate) async fn record_operations(
        &self,
        operations: &[DeviceOperation],
    ) -> Result<(), String> {
        self.operations.lock().await.extend_from_slice(operations);
        Ok(())
    }

    pub(crate) async fn take_screenshot(&self) -> Result<RgbaImage, String> {
        self.screenshots
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| "测试任务没有剩余截图可供读取".to_string())
    }

    pub(crate) async fn take_capture_snapshot(
        &self,
        grid_size: u16,
    ) -> Result<Option<VisionSnapshot>, String> {
        self.capture_frames
            .lock()
            .await
            .pop_front()
            .map(|frame| frame.into_snapshot(grid_size))
            .transpose()
    }

    pub(crate) async fn take_detect_results(&self) -> Option<Vec<DetResult>> {
        self.detect_frames
            .lock()
            .await
            .pop_front()
            .map(|frame| frame.det_results)
    }

    pub(crate) async fn take_ocr_results(&self) -> Option<(Vec<DetResult>, Vec<OcrResult>)> {
        self.ocr_frames
            .lock()
            .await
            .pop_front()
            .map(|frame| (frame.det_results, frame.ocr_results))
    }
}

pub async fn run_task_test(mut request: TestTaskRunRequest) -> Result<Value, String> {
    let _guard = TEST_RUN_MUTEX
        .get_or_init(|| Arc::new(Mutex::new(())))
        .clone()
        .lock_owned()
        .await;
    clear_stop_request();
    let runtime_ctx = ensure_test_runtime_context()?;
    let hooks = Arc::new(TestRuntimeHooks::new(&mut request));
    let script_id = request.bundle.script_id;
    let device_id = DeviceId::new_v7();
    let assignment_id = AssignmentId::new_v7();
    let queue_item = RuntimeQueueItem {
        dispatch_id: DispatchId::new_v7(),
        dispatch_kind: DispatchKind::TemporaryTask,
        dispatch_source: DispatchSource::Debug,
        assignment_id,
        script_id,
        time_template_id: None,
        account_id: None,
        account_data_json: None,
        order_index: 0,
        window_start_at: None,
        template_values_json: request.template_values_json,
        dedup_scope_base_hash: String::new(),
    };
    let session = RuntimeSessionSnapshot {
        session_id: SessionId::new_v7(),
        device_id,
        run_target: RunTarget::Task {
            script_id,
            task_id: request.task_id,
        },
        runtime_policy: RuntimeExecutionPolicy {
            action_wait_ms: 0,
            progress_timeout_enabled: false,
            progress_timeout_ms: 30_000,
            timeout_action: TimeoutAction::StopExecution,
            timeout_notify_channels: Vec::new(),
        },
        queue: vec![queue_item.clone()],
        script_bundles: vec![request.bundle],
        issued_at: chrono::Utc::now().to_rfc3339(),
    };
    replace_runtime_session(session).await;

    let scheduler = ScriptScheduler::new_with_test_hooks(CancellationToken::new(), hooks.clone());
    let execution = scheduler.execute_test_item(queue_item).await;
    let (outcome, error) = match execution {
        Ok(false) => ("completed", None),
        Ok(true) => ("stopped", None),
        Err(error) => ("failed", Some(error)),
    };
    let result = build_result(runtime_ctx, hooks, outcome, error).await;
    clear_runtime_session().await;
    clear_stop_request();
    Ok(result)
}

fn ensure_test_runtime_context() -> Result<Arc<RwLock<RuntimeContext>>, String> {
    if TEST_RUNTIME_INITIALIZED.set(()).is_ok() {
        let script_id = UuidV7::new_v7();
        let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
            script_id,
            RunTarget::FullScript { script_id },
            Arc::new(Mutex::new(OcrService::new())),
            Arc::new(Mutex::new(OcrService::new())),
            VisionTextCacheRuntimeConfig {
                enabled: false,
                dir: None,
                signature_grid_size: 8,
            },
        )));
        match init_runtime_ctx(runtime_ctx.clone()) {
            Ok(()) => return Ok(runtime_ctx),
            Err(_) => return Ok(get_runtime_ctx()),
        }
    }
    Ok(get_runtime_ctx())
}

async fn build_result(
    runtime_ctx: Arc<RwLock<RuntimeContext>>,
    hooks: Arc<TestRuntimeHooks>,
    outcome: &str,
    error: Option<String>,
) -> Value {
    let ctx = runtime_ctx.read().await;
    let variables = ctx
        .execution
        .var_map
        .iter()
        .map(|(key, value)| {
            let value = rhai::serde::from_dynamic::<Value>(value)
                .unwrap_or_else(|_| Value::String(format!("{value:?}")));
            (key.clone(), value)
        })
        .collect::<Map<_, _>>();
    let task_states = ctx
        .execution
        .task_states
        .iter()
        .map(|(id, state)| {
            (
                id.to_string(),
                json!({
                    "enabled": state.enabled_flag,
                    "skipped": state.skip_flag,
                    "done": state.done_flag,
                    "execCount": state.exec_cur,
                }),
            )
        })
        .collect::<Map<_, _>>();
    let policy_states = ctx
        .execution
        .policy_states
        .iter()
        .map(|(id, state)| {
            (
                id.to_string(),
                json!({
                    "skipped": state.skip_flag,
                    "done": state.done_flag,
                    "execCount": state.exec_cur,
                    "clickPosition": state.click_pos,
                }),
            )
        })
        .collect::<Map<_, _>>();
    let action_states = ctx
        .execution
        .action_states
        .iter()
        .map(|(id, state)| (id.to_string(), json!({ "execCount": state.exec_cur })))
        .collect::<Map<_, _>>();
    let policy_set_bindings = ctx
        .execution
        .policy_set_bindings
        .iter()
        .map(|(target, bindings)| {
            let items = bindings
                .iter()
                .map(|binding| {
                    let (source_type, source_id) = match binding.source {
                        PolicySetBindingSource::PolicySet(id) => ("policySet", id.to_string()),
                        PolicySetBindingSource::PolicyGroup(id) => ("policyGroup", id.to_string()),
                    };
                    json!({
                        "sourceType": source_type,
                        "sourceId": source_id,
                        "top": binding.top,
                        "reverse": binding.reverse,
                    })
                })
                .collect::<Vec<_>>();
            (target.to_string(), Value::Array(items))
        })
        .collect::<Map<_, _>>();
    let policy_group_bindings = ctx
        .execution
        .policy_group_bindings
        .iter()
        .map(|(target, bindings)| {
            let items = bindings
                .iter()
                .map(|binding| {
                    let (source_type, source_id) = match binding.source {
                        PolicyGroupBindingSource::Policy(id) => ("policy", id.to_string()),
                        PolicyGroupBindingSource::PolicyGroup(id) => {
                            ("policyGroup", id.to_string())
                        }
                    };
                    json!({
                        "sourceType": source_type,
                        "sourceId": source_id,
                        "top": binding.top,
                        "reverse": binding.reverse,
                    })
                })
                .collect::<Vec<_>>();
            (target.to_string(), Value::Array(items))
        })
        .collect::<Map<_, _>>();
    let (det_results, ocr_results) = ctx
        .observation
        .last_snapshot
        .as_ref()
        .map(|snapshot| (snapshot.det_items.clone(), snapshot.ocr_items.clone()))
        .unwrap_or_default();
    drop(ctx);

    let operations = hooks
        .operations
        .lock()
        .await
        .drain(..)
        .map(operation_to_value)
        .collect::<Vec<_>>();
    json!({
        "execution": {
            "outcome": outcome,
            "error": error,
        },
        "operations": operations,
        "variables": variables,
        "taskStates": task_states,
        "policyStates": policy_states,
        "actionStates": action_states,
        "policySetBindings": policy_set_bindings,
        "policyGroupBindings": policy_group_bindings,
        "vision": {
            "detResults": det_results,
            "ocrResults": ocr_results,
        }
    })
}

fn operation_to_value(operation: DeviceOperation) -> Value {
    match operation {
        DeviceOperation::Click(point) => json!({ "type": "click", "x": point.x, "y": point.y }),
        DeviceOperation::LongClick(point) => {
            json!({ "type": "longClick", "x": point.x, "y": point.y })
        }
        DeviceOperation::Swipe { from, to, duration } => json!({
            "type": "swipe",
            "from": { "x": from.x, "y": from.y },
            "to": { "x": to.x, "y": to.y },
            "duration": duration,
        }),
        DeviceOperation::LaunchApp {
            pkg_name,
            activity_name,
        } => json!({
            "type": "launchApp",
            "package": pkg_name,
            "activity": activity_name,
        }),
        DeviceOperation::StopApp { pkg_name } => {
            json!({ "type": "stopApp", "package": pkg_name })
        }
        DeviceOperation::InputText(text) => json!({ "type": "inputText", "text": text }),
        DeviceOperation::Back => json!({ "type": "back" }),
        DeviceOperation::Home => json!({ "type": "home" }),
        DeviceOperation::Reboot => json!({ "type": "reboot" }),
        DeviceOperation::Delay(ms) => json!({ "type": "delay", "ms": ms }),
    }
}
