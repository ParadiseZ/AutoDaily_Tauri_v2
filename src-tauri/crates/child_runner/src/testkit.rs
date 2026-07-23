use crate::infra::context::runtime_context::{RuntimeContext, get_runtime_ctx, init_runtime_ctx};
use crate::infra::context::runtime_control::clear_stop_request;
use crate::infra::context::{PolicyGroupBindingSource, PolicySetBindingSource};
use crate::infra::scripts::scheduler::ScriptScheduler;
use crate::infra::session::runtime_session::{clear_runtime_session, replace_runtime_session};
use ad_kernel::{
    LogLevel,
    ids::{AssignmentId, DeviceId, DispatchId, SessionId, TaskId},
};
use domain_device::{DeviceConfig, DeviceOperation, DevicePlatform, TimeoutAction};
use domain_script::Step;
use domain_vision::VisionTextCacheRuntimeConfig;
use image::RgbaImage;
use infra_adb::ADBCtx;
use infra_device_runtime::{
    DeviceCtx, ensure_device_connection_with_progress, init_device_ctx, try_get_device_ctx,
};
use infra_vision::OcrService;
use runner_protocol::message::{
    DispatchKind, DispatchSource, RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem,
    RuntimeSessionSnapshot, ScriptBundleSnapshot,
};
use serde_json::{Map, Value, json};
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

static TEST_RUN_MUTEX: OnceLock<Arc<Mutex<()>>> = OnceLock::new();
static TEST_RUNTIME_INITIALIZED: OnceLock<()> = OnceLock::new();

pub struct TestScriptRunRequest {
    pub bundle: ScriptBundleSnapshot,
    pub task_id: Option<TaskId>,
    pub device_id: DeviceId,
    pub device_config: Option<DeviceConfig>,
    pub template_values_json: Option<String>,
}

pub(crate) struct TestRuntimeHooks {
    operations: Mutex<Vec<Value>>,
    prints: Mutex<Vec<Value>>,
    step_trace: Mutex<Vec<Value>>,
}

impl TestRuntimeHooks {
    fn new() -> Self {
        Self {
            operations: Mutex::new(Vec::new()),
            prints: Mutex::new(Vec::new()),
            step_trace: Mutex::new(Vec::new()),
        }
    }

    pub(crate) async fn record_operation(&self, operation: &DeviceOperation, error: Option<&str>) {
        let mut value = operation_to_value(operation.clone());
        if let Some(object) = value.as_object_mut() {
            object.insert(
                "status".to_string(),
                Value::String(if error.is_some() { "failed" } else { "success" }.to_string()),
            );
            if let Some(error) = error {
                object.insert("error".to_string(), Value::String(error.to_string()));
            }
        }
        self.operations.lock().await.push(value);
    }

    pub(crate) async fn record_operations(
        &self,
        operations: &[DeviceOperation],
        error: Option<&str>,
    ) {
        for operation in operations {
            self.record_operation(operation, error).await;
        }
    }

    pub(crate) async fn record_print(&self, level: &LogLevel, message: String) {
        self.prints.lock().await.push(json!({
            "level": format!("{level:?}"),
            "message": message,
        }));
    }

    pub(crate) async fn record_step(
        &self,
        phase: &str,
        step: &Step,
        outcome: Option<&str>,
        error: Option<&str>,
    ) {
        let mut value = json!({
            "phase": phase,
            "stepId": step.id.map(|id| id.to_string()),
            "label": step.label,
            "kind": step_capability(step),
        });
        if let Some(object) = value.as_object_mut() {
            if let Some(outcome) = outcome {
                object.insert("outcome".to_string(), Value::String(outcome.to_string()));
            }
            if let Some(error) = error {
                object.insert("error".to_string(), Value::String(error.to_string()));
            }
        }
        self.step_trace.lock().await.push(value);
    }
}

pub async fn run_script_test(request: TestScriptRunRequest) -> Result<Value, String> {
    let _guard = TEST_RUN_MUTEX
        .get_or_init(|| Arc::new(Mutex::new(())))
        .clone()
        .lock_owned()
        .await;
    clear_stop_request();
    if let Some(device_config) = request.device_config.as_ref() {
        prepare_test_device(device_config).await?;
    }
    let runtime_ctx = ensure_test_runtime_context(request.device_id)?;
    let hooks = Arc::new(TestRuntimeHooks::new());
    let script_id = request.bundle.script_id;
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
    let run_target = request
        .task_id
        .map_or(RunTarget::FullScript { script_id }, |task_id| {
            RunTarget::Task { script_id, task_id }
        });
    let session = RuntimeSessionSnapshot {
        session_id: SessionId::new_v7(),
        device_id: request.device_id,
        run_target,
        runtime_policy: request
            .device_config
            .as_ref()
            .map(|config| RuntimeExecutionPolicy {
                action_wait_ms: config.execution_policy.action_wait_ms.into(),
                progress_timeout_enabled: config.execution_policy.progress_timeout_enabled,
                progress_timeout_ms: config.execution_policy.progress_timeout_ms.into(),
                timeout_action: config.execution_policy.timeout_action.clone(),
                timeout_notify_channels: config.execution_policy.timeout_notify_channels.clone(),
            })
            .unwrap_or(RuntimeExecutionPolicy {
                action_wait_ms: 0,
                progress_timeout_enabled: false,
                progress_timeout_ms: 30_000,
                timeout_action: TimeoutAction::StopExecution,
                timeout_notify_channels: Vec::new(),
            }),
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

async fn prepare_test_device(device_config: &DeviceConfig) -> Result<(), String> {
    if let Some(device_ctx) = try_get_device_ctx() {
        device_ctx.apply_device_config(device_config.clone()).await;
    } else {
        let device_ctx =
            Arc::new(DeviceCtx::new(Arc::new(RwLock::new(device_config.clone()))).await);
        init_device_ctx(device_ctx)?;
    }
    if matches!(device_config.platform, DevicePlatform::Android) {
        let runtime_connect =
            ensure_device_connection_with_progress(device_config, |_, _| {}).await?;
        ADBCtx::new(runtime_connect).await?;
    }
    Ok(())
}

fn ensure_test_runtime_context(device_id: DeviceId) -> Result<Arc<RwLock<RuntimeContext>>, String> {
    if TEST_RUNTIME_INITIALIZED.set(()).is_ok() {
        let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
            device_id,
            RunTarget::DeviceQueue,
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
            let value = runtime_value_to_json(value);
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

    let operations = hooks.operations.lock().await.drain(..).collect::<Vec<_>>();
    let prints = hooks.prints.lock().await.drain(..).collect::<Vec<_>>();
    let step_trace = hooks.step_trace.lock().await.drain(..).collect::<Vec<_>>();
    json!({
        "execution": {
            "outcome": outcome,
            "error": error,
        },
        "operations": operations,
        "prints": prints,
        "stepTrace": step_trace,
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

fn runtime_value_to_json(value: &rhai::Dynamic) -> Value {
    if let Some(image) = value.clone().try_cast::<Arc<RgbaImage>>() {
        return json!({
            "type": "image",
            "width": image.width(),
            "height": image.height(),
        });
    }
    rhai::serde::from_dynamic::<Value>(value).unwrap_or_else(|_| {
        json!({
            "type": value.type_name(),
        })
    })
}

fn step_capability(step: &Step) -> String {
    let Ok(value) = serde_json::to_value(&step.kind) else {
        return "unknown".to_string();
    };
    let Some(object) = value.as_object() else {
        return "unknown".to_string();
    };
    let Some(op) = object.get("op").and_then(Value::as_str) else {
        return "unknown".to_string();
    };
    if op == "sequence" {
        return op.to_string();
    }
    let Some(action) = object.get("a").and_then(Value::as_object) else {
        return op.to_string();
    };
    let tag = if op == "action" { "ac" } else { "type" };
    action
        .get(tag)
        .and_then(Value::as_str)
        .map(|kind| format!("{op}.{kind}"))
        .unwrap_or_else(|| op.to_string())
}

fn operation_to_value(operation: DeviceOperation) -> Value {
    match operation {
        DeviceOperation::Click(point) => json!({
            "type": "click", "x": point.x, "y": point.y,
            "adbShell": format!("adb shell input tap {} {}", point.x, point.y),
        }),
        DeviceOperation::LongClick(point) => {
            json!({
                "type": "longClick", "x": point.x, "y": point.y,
                "adbShell": format!("adb shell input swipe {0} {1} {0} {1} 800", point.x, point.y),
            })
        }
        DeviceOperation::Swipe { from, to, duration } => json!({
            "type": "swipe",
            "from": { "x": from.x, "y": from.y },
            "to": { "x": to.x, "y": to.y },
            "duration": duration,
            "adbShell": format!("adb shell input swipe {} {} {} {} {}", from.x, from.y, to.x, to.y, duration),
        }),
        DeviceOperation::LaunchApp {
            pkg_name,
            activity_name,
        } => json!({
            "type": "launchApp",
            "package": pkg_name,
            "activity": activity_name,
            "adbShell": format!("adb shell am start -n {pkg_name}/{activity_name}"),
        }),
        DeviceOperation::StopApp { pkg_name } => {
            json!({
                "type": "stopApp", "package": pkg_name,
                "adbShell": format!("adb shell am force-stop {pkg_name}"),
            })
        }
        DeviceOperation::InputText(text) => json!({
            "type": "inputText", "text": text,
            "adbShell": format!("adb shell input text {}", text.replace(' ', "%s")),
        }),
        DeviceOperation::Back => {
            json!({ "type": "back", "adbShell": "adb shell input keyevent 4" })
        }
        DeviceOperation::Home => {
            json!({ "type": "home", "adbShell": "adb shell input keyevent 3" })
        }
        DeviceOperation::Reboot => json!({ "type": "reboot", "adbShell": "adb reboot" }),
        DeviceOperation::Delay(ms) => json!({
            "type": "delay", "ms": ms,
            "adbShell": format!("# runtime wait {ms}ms (no adb command)"),
        }),
    }
}
