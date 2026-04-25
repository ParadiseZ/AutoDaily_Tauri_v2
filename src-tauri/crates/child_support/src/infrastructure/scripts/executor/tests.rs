use super::ScriptExecutor;
use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
use crate::domain::devices::device_schedule::TaskCycle;
use crate::domain::scripts::nodes::data_handing::{ColorCompareMethod, ColorRgb};
use crate::domain::scripts::policy::{PolicyInfo, PolicyTable};
use crate::domain::scripts::nodes::flow_control::{ConditionNode, FlowControl};
use crate::domain::scripts::nodes::action::Action;
use crate::domain::scripts::nodes::task_control::{StateStatus, StateTarget, TaskControl};
use crate::domain::scripts::script_decision::{Step, StepKind};
use crate::domain::scripts::nodes::flow_control::PolicySetResultCompareOp;
use crate::domain::scripts::script_task::{
    ScriptTask, ScriptTaskTable, TaskRowType, TaskTone, TaskTriggerMode,
};
use crate::domain::scripts::script_variable::{
    ScriptVariableCatalog, ScriptVariableDef, ScriptVariableNamespace, ScriptVariableSourceType,
    ScriptVariableValueType,
};
use crate::domain::vision::ocr_search::{SearchRule, VisionSnapshot};
use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult};
use crate::infrastructure::context::runtime_context::RuntimeContext;
use crate::infrastructure::core::{PolicyId, TaskId, UuidV7};
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem, RuntimeSessionSnapshot,
    RuntimeVisionTextCachePolicy, TimeoutAction,
};
use crate::infrastructure::session::runtime_session::{clear_runtime_session, replace_runtime_session};
use crate::infrastructure::vision::ocr_service::OcrService;
use image::{Rgba, RgbaImage};
use rhai::serde::to_dynamic;
use serde_json::{json, Value};
use sqlx::types::Json;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::{Mutex, OwnedMutexGuard};
use tokio::sync::RwLock;

static RUNTIME_SESSION_TEST_MUTEX: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

fn build_ocr_result(txt: &str, x1: i32, y1: i32, x2: i32, y2: i32) -> OcrResult {
    OcrResult::new(
        BoundingBox::new(x1, y1, x2, y2),
        txt.to_string(),
        vec![0.9],
        vec![1],
        vec![txt.to_string()],
        8,
    )
}

fn build_det_result(index: i32, label: &str, x1: i32, y1: i32, x2: i32, y2: i32) -> DetResult {
    DetResult::new(
        BoundingBox::new(x1, y1, x2, y2),
        index,
        label.to_string(),
        0.9,
        8,
    )
}

fn fill_rect(image: &mut RgbaImage, bbox: &BoundingBox, color: [u8; 4]) {
    for y in bbox.y1..=bbox.y2 {
        for x in bbox.x1..=bbox.x2 {
            image.put_pixel(x as u32, y as u32, Rgba(color));
        }
    }
}

#[test]
fn select_ocr_result_prefers_exact_match_then_contains() {
    let items = vec![
        build_ocr_result("开始行动", 0, 0, 40, 20),
        build_ocr_result("开始", 50, 0, 90, 20),
    ];

    let exact = ScriptExecutor::select_ocr_result(&items, Some("开始")).unwrap();
    assert_eq!(exact.txt, "开始");

    let contains = ScriptExecutor::select_ocr_result(&items, Some("行动")).unwrap();
    assert_eq!(contains.txt, "开始行动");
}

#[test]
fn select_det_result_matches_label_index() {
    let items = vec![
        build_det_result(3, "enemy", 0, 0, 40, 40),
        build_det_result(7, "ally", 50, 0, 90, 40),
    ];

    let matched = ScriptExecutor::select_det_result(&items, Some(7)).unwrap();
    assert_eq!(matched.label, "ally");
}

#[test]
fn bounding_box_center_converts_to_device_point() {
    let point = ScriptExecutor::bounding_box_center_to_point(
        "action.click",
        "点击目标",
        &BoundingBox::new(10, 20, 30, 40),
    )
    .unwrap();

    assert_eq!(point.x, 20);
    assert_eq!(point.y, 30);
}

#[test]
fn result_vec_can_be_deserialized_from_dynamic() {
    let dynamic = to_dynamic(vec![build_ocr_result("开始", 0, 0, 20, 20)]).unwrap();
    let items = ScriptExecutor::deserialize_dynamic_value::<Vec<OcrResult>>(&dynamic).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].txt, "开始");
}

#[test]
fn capture_cache_key_changes_with_signature_grid_size() {
    let image = RgbaImage::from_pixel(4, 4, Rgba([12, 34, 56, 255]));

    let key_a = ScriptExecutor::build_capture_cache_key(&image, 4, "null", "null", "null");
    let key_b = ScriptExecutor::build_capture_cache_key(&image, 8, "null", "null", "null");

    assert_ne!(key_a, key_b);
}

#[test]
fn capture_cache_key_changes_with_model_config() {
    let image = RgbaImage::from_pixel(4, 4, Rgba([12, 34, 56, 255]));

    let key_a =
        ScriptExecutor::build_capture_cache_key(&image, 4, "{\"model\":\"a\"}", "null", "null");
    let key_b =
        ScriptExecutor::build_capture_cache_key(&image, 4, "{\"model\":\"b\"}", "null", "null");

    assert_ne!(key_a, key_b);
}

#[test]
fn compare_optional_id_supports_eq_and_ne() {
    let actual = Some(crate::infrastructure::core::UuidV7(42));

    assert!(ScriptExecutor::compare_optional_id(
        actual,
        &PolicySetResultCompareOp::Eq,
        &crate::infrastructure::core::UuidV7(42).to_string(),
    ));
    assert!(ScriptExecutor::compare_optional_id(
        actual,
        &PolicySetResultCompareOp::Ne,
        &crate::infrastructure::core::UuidV7(7).to_string(),
    ));
}

#[test]
fn compare_bool_supports_eq_and_ne() {
    assert!(ScriptExecutor::compare_bool(
        true,
        &PolicySetResultCompareOp::Eq,
        true,
    ));
    assert!(ScriptExecutor::compare_bool(
        true,
        &PolicySetResultCompareOp::Ne,
        false,
    ));
}

fn build_task_with_variables(task_id: TaskId, variables: Value) -> ScriptTaskTable {
    ScriptTaskTable {
        id: task_id,
        script_id: UuidV7(1),
        name: "task".to_string(),
        row_type: TaskRowType::Task,
        trigger_mode: TaskTriggerMode::RootOnly,
        record_schedule: false,
        section_id: None,
        indent_level: 0,
        default_task_cycle: Json(TaskCycle::EveryRun),
        exec_max: 0,
        show_enabled_toggle: true,
        default_enabled: true,
        task_tone: TaskTone::Normal,
        is_hidden: false,
        data: Json(ScriptTask {
            ui_data: Value::Null,
            variables,
            steps: Vec::new(),
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        is_deleted: false,
        index: 0,
    }
}

fn build_input_variable(task_id: TaskId) -> ScriptVariableDef {
    ScriptVariableDef {
        id: "var_pkg_name_id".to_string(),
        key: "input.pkg_name".to_string(),
        name: "包名".to_string(),
        namespace: ScriptVariableNamespace::Input,
        value_type: ScriptVariableValueType::String,
        owner_task_id: Some(task_id),
        source_type: ScriptVariableSourceType::Manual,
        source_step_id: None,
        readable: true,
        writable: true,
        persisted: true,
        ui_bindable: true,
        default_value: Some(json!("default-from-catalog")),
        description: String::new(),
    }
}

fn build_script_level_input_variable() -> ScriptVariableDef {
    ScriptVariableDef {
        id: "script_level_input_id".to_string(),
        key: "input.shared_flag".to_string(),
        name: "共享开关".to_string(),
        namespace: ScriptVariableNamespace::Input,
        value_type: ScriptVariableValueType::Bool,
        owner_task_id: None,
        source_type: ScriptVariableSourceType::Manual,
        source_step_id: None,
        readable: true,
        writable: true,
        persisted: true,
        ui_bindable: true,
        default_value: Some(json!(true)),
        description: String::new(),
    }
}

fn build_executor_with_target(run_target: RunTarget) -> ScriptExecutor {
    let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
        UuidV7(1),
        run_target,
        Arc::new(Mutex::new(OcrService::new())),
        Arc::new(Mutex::new(OcrService::new())),
        VisionTextCacheRuntimeConfig {
            enabled: false,
            dir: None,
            signature_grid_size: 8,
        },
    )));
    ScriptExecutor::new(runtime_ctx)
}

fn build_executor() -> ScriptExecutor {
    build_executor_with_target(RunTarget::FullScript {
        script_id: UuidV7(1),
    })
}

fn build_input_catalog(task_id: TaskId) -> ScriptVariableCatalog {
    ScriptVariableCatalog {
        version: 1,
        variables: vec![build_input_variable(task_id), build_script_level_input_variable()],
    }
}

async fn install_runtime_policy_for_test(timeout_action: TimeoutAction) {
    clear_runtime_session().await;
    replace_runtime_session(RuntimeSessionSnapshot {
        session_id: UuidV7(501),
        device_id: UuidV7(502),
        run_target: RunTarget::FullScript {
            script_id: UuidV7(1),
        },
        runtime_policy: RuntimeExecutionPolicy {
            ocr_text_cache: RuntimeVisionTextCachePolicy {
                enabled: false,
                dir: None,
                signature_grid_size: 8,
            },
            action_wait_ms: 0,
            progress_timeout_enabled: true,
            progress_timeout_ms: 1_000,
            timeout_action,
            timeout_notify_channels: Vec::new(),
        },
        queue: vec![RuntimeQueueItem {
            assignment_id: UuidV7(503),
            script_id: UuidV7(1),
            time_template_id: None,
            account_id: None,
            account_data_json: None,
            order_index: 0,
            template_values_json: None,
        }],
        script_bundles: Vec::new(),
        issued_at: chrono::Utc::now().to_rfc3339(),
    })
    .await;
}

async fn acquire_runtime_session_test_guard() -> OwnedMutexGuard<()> {
    RUNTIME_SESSION_TEST_MUTEX
        .get_or_init(|| Arc::new(Mutex::new(())))
        .clone()
        .lock_owned()
        .await
}

fn build_set_var_step(name: &str, expr: &str) -> Step {
    Step {
        id: None,
        source_id: None,
        target_id: None,
        label: None,
        skip_flag: false,
        kind: StepKind::DataHanding {
            a: crate::domain::scripts::nodes::data_handing::DataHanding::SetVar {
                name: name.to_string(),
                val: None,
                expr: Some(expr.to_string()),
            },
        },
    }
}

fn build_policy_table(
    policy_id: PolicyId,
    script_id: UuidV7,
    before_action: Vec<Step>,
    after_action: Vec<Step>,
) -> PolicyTable {
    PolicyTable {
        id: policy_id,
        script_id,
        order_index: 0,
        data: Json(PolicyInfo {
            name: "policy-debug".to_string(),
            note: String::new(),
            log_print: None,
            cur_pos: 0,
            skip_flag: false,
            exec_max: 0,
            before_action,
            cond: SearchRule::Txt {
                pattern: "开始".to_string(),
            },
            after_action,
        }),
    }
}

#[test]
fn input_variable_prefers_template_values_by_variable_id() {
    let task_id = UuidV7(7);
    let task = build_task_with_variables(
        task_id,
        json!({
            "pkg_name": "default-from-task"
        }),
    );
    let variable = build_input_variable(task_id);
    let template_values = ScriptExecutor::parse_runtime_template_values(Some(
        r#"{"variables":{"var_pkg_name_id":"templated-value"}}"#,
    ))
    .unwrap()
    .unwrap();

    let value = ScriptExecutor::resolve_input_variable_value(
        &variable,
        Some(&template_values),
        Some(&task),
    );
    assert_eq!(value, Some(json!("templated-value")));
}

#[test]
fn input_variable_falls_back_to_task_storage_key() {
    let task_id = UuidV7(8);
    let task = build_task_with_variables(
        task_id,
        json!({
            "pkg_name": "default-from-task"
        }),
    );
    let variable = build_input_variable(task_id);

    let value = ScriptExecutor::resolve_input_variable_value(&variable, None, Some(&task));
    assert_eq!(value, Some(json!("default-from-task")));
}

#[test]
fn input_variable_falls_back_to_catalog_default_without_template_or_task_value() {
    let variable = build_script_level_input_variable();

    let value = ScriptExecutor::resolve_input_variable_value(&variable, None, None);
    assert_eq!(value, Some(json!(true)));
}

#[test]
fn task_owned_input_variable_is_hidden_without_task_context() {
    let task_id = UuidV7(9);
    let variable = build_input_variable(task_id);

    assert!(!ScriptExecutor::input_variable_visible_for_task(
        &variable, None
    ));
}

#[test]
fn policy_debug_target_exposes_task_owned_input_without_task_context() {
    let task_id = UuidV7(10);
    let variable = build_input_variable(task_id);

    assert!(ScriptExecutor::input_variable_visible_for_target(
        &variable,
        &RunTarget::Policy {
            script_id: UuidV7(1),
            policy_id: UuidV7(2),
        },
        None,
    ));
    assert!(!ScriptExecutor::input_variable_visible_for_target(
        &variable,
        &RunTarget::FullScript {
            script_id: UuidV7(1),
        },
        None,
    ));
}

#[tokio::test]
async fn hydrate_input_scope_loads_task_owned_defaults_for_policy_debug_target() {
    let task_id = UuidV7(11);
    let mut executor = build_executor_with_target(RunTarget::Policy {
        script_id: UuidV7(1),
        policy_id: UuidV7(2),
    });

    executor
        .hydrate_input_scope(&build_input_catalog(task_id), None, None)
        .await
        .unwrap();

    let task_owned = executor.read_runtime_var("input.pkg_name").await.unwrap();
    let script_owned = executor.read_runtime_var("input.shared_flag").await.unwrap();

    assert_eq!(
        ScriptExecutor::deserialize_dynamic_value::<String>(&task_owned).unwrap(),
        "default-from-catalog"
    );
    assert!(ScriptExecutor::deserialize_dynamic_value::<bool>(&script_owned).unwrap());
}

#[tokio::test]
async fn hydrate_input_scope_prefers_template_values_for_policy_debug_target() {
    let task_id = UuidV7(12);
    let mut executor = build_executor_with_target(RunTarget::PolicySet {
        script_id: UuidV7(1),
        policy_set_id: UuidV7(3),
    });

    executor
        .hydrate_input_scope(
            &build_input_catalog(task_id),
            Some(r#"{"variables":{"var_pkg_name_id":"templated-from-policy-debug"}}"#),
            None,
        )
        .await
        .unwrap();

    let task_owned = executor.read_runtime_var("input.pkg_name").await.unwrap();
    assert_eq!(
        ScriptExecutor::deserialize_dynamic_value::<String>(&task_owned).unwrap(),
        "templated-from-policy-debug"
    );
}

#[tokio::test]
async fn policy_debug_candidate_steps_can_read_task_owned_inputs_after_hydration() {
    let script_id = UuidV7(1);
    let task_id = UuidV7(13);
    let policy_id = UuidV7(21);
    let policy_group_id = UuidV7(31);
    let policy_set_id = UuidV7(41);
    let mut executor = build_executor_with_target(RunTarget::PolicyGroup {
        script_id,
        policy_group_id,
    });

    executor
        .hydrate_input_scope(
            &build_input_catalog(task_id),
            Some(r#"{"variables":{"var_pkg_name_id":"templated-from-group-debug"}}"#),
            None,
        )
        .await
        .unwrap();

    {
        let mut ctx = executor.runtime_ctx.write().await;
        ctx.observation.last_snapshot = Some(
            VisionSnapshot::new(
                vec![build_ocr_result("开始", 0, 0, 30, 12)],
                Vec::new(),
                None,
                8,
            )
            .unwrap(),
        );
    }

    let candidate = super::PolicyCandidate {
        policy_set_id: Some(policy_set_id),
        policy_group_id: Some(policy_group_id),
        policy: build_policy_table(
            policy_id,
            script_id,
            vec![build_set_var_step("runtime.beforeValue", "input.pkg_name")],
            vec![build_set_var_step(
                "runtime.afterValue",
                r#"if input.shared_flag { input.pkg_name + "-after" } else { "disabled" }"#,
            )],
        ),
    };

    let flow = executor
        .execute_policy_candidates("debug.policy", vec![candidate], "runtime.policyDebugResult")
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Next));
    let before_value = executor.read_runtime_var("runtime.beforeValue").await.unwrap();
    let after_value = executor.read_runtime_var("runtime.afterValue").await.unwrap();

    assert_eq!(
        ScriptExecutor::deserialize_dynamic_value::<String>(&before_value).unwrap(),
        "templated-from-group-debug"
    );
    assert_eq!(
        ScriptExecutor::deserialize_dynamic_value::<String>(&after_value).unwrap(),
        "templated-from-group-debug-after"
    );
}

#[tokio::test]
async fn timeout_handling_resets_progress_probe_after_stop_execution() {
    let mut executor = build_executor();
    executor.last_progress_probe = Some(super::ProgressProbe {
        page_fingerprint: Some("page:v1:deadbeef".to_string()),
        evidence_signature: "evidence:v1".to_string(),
        task_id: None,
        step_id: None,
        stagnant_since: Instant::now() - std::time::Duration::from_secs(10),
        notified: false,
    });

    let runtime_policy = RuntimeExecutionPolicy {
        ocr_text_cache: RuntimeVisionTextCachePolicy {
            enabled: false,
            dir: None,
            signature_grid_size: 8,
        },
        action_wait_ms: 0,
        progress_timeout_enabled: true,
        progress_timeout_ms: 1_000,
        timeout_action: TimeoutAction::StopExecution,
        timeout_notify_channels: Vec::new(),
    };

    let error = executor
        .evaluate_progress_probe(
            &runtime_policy,
            Some("page:v1:deadbeef".to_string()),
            "evidence:v1".to_string(),
            "unit-test timeout".to_string(),
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("unit-test timeout"));
    assert!(executor.last_progress_probe.is_none());
}

#[tokio::test]
async fn device_operation_helper_times_out() {
    let error = ScriptExecutor::await_device_result_with_timeout(
        "unit.deviceTimeout",
        "设备操作",
        10,
        async {
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            Ok::<(), String>(())
        },
    )
    .await
    .unwrap_err();

    assert!(error.to_string().contains("设备操作超时"));
}

#[tokio::test]
async fn vision_inference_helper_times_out() {
    let error = ScriptExecutor::run_ocr_service_with_timeout(
        "unit.visionTimeout",
        "OCR",
        10,
        Arc::new(Mutex::new(OcrService::new())),
        |_service| {
            std::thread::sleep(std::time::Duration::from_millis(30));
            Ok::<(), String>(())
        },
    )
    .await
    .unwrap_err();

    assert!(error.to_string().contains("OCR超时"));
}

#[tokio::test]
async fn if_condition_path_triggers_timeout_detector() {
    let _guard = acquire_runtime_session_test_guard().await;
    install_runtime_policy_for_test(TimeoutAction::SkipCurrentTask).await;
    let mut executor = build_executor();
    executor.last_progress_probe = Some(super::ProgressProbe {
        page_fingerprint: None,
        evidence_signature: "flow.if".to_string(),
        task_id: None,
        step_id: None,
        stagnant_since: Instant::now() - std::time::Duration::from_secs(10),
        notified: false,
    });

    let flow = executor
        .execute_flow_control_step(&FlowControl::If {
            con: ConditionNode::RawExpr {
                expr: "true".to_string(),
            },
            then: Vec::new(),
            else_steps: None,
        })
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Return));
    clear_runtime_session().await;
}

#[tokio::test]
async fn data_set_var_path_triggers_timeout_detector() {
    let _guard = acquire_runtime_session_test_guard().await;
    install_runtime_policy_for_test(TimeoutAction::SkipCurrentTask).await;
    let mut executor = build_executor();
    executor.last_progress_probe = Some(super::ProgressProbe {
        page_fingerprint: None,
        evidence_signature: "data.setVar".to_string(),
        task_id: None,
        step_id: None,
        stagnant_since: Instant::now() - std::time::Duration::from_secs(10),
        notified: false,
    });

    let flow = executor
        .execute_data_handling_step(
            &crate::domain::scripts::nodes::data_handing::DataHanding::SetVar {
                name: "runtime.timeoutProbe".to_string(),
                val: None,
                expr: Some("1 + 1".to_string()),
            },
        )
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Return));
    clear_runtime_session().await;
}

#[tokio::test]
async fn vision_search_path_triggers_timeout_detector() {
    let _guard = acquire_runtime_session_test_guard().await;
    install_runtime_policy_for_test(TimeoutAction::SkipCurrentTask).await;
    let mut executor = build_executor();
    let snapshot = VisionSnapshot::new(
        vec![build_ocr_result("开始", 0, 0, 24, 12)],
        Vec::new(),
        None,
        8,
    )
    .unwrap();
    let page_fingerprint = super::ScriptExecutor::build_page_fingerprint(&snapshot);
    executor.last_progress_probe = Some(super::ProgressProbe {
        page_fingerprint: Some(page_fingerprint),
        evidence_signature: "vision.search".to_string(),
        task_id: None,
        step_id: None,
        stagnant_since: Instant::now() - std::time::Duration::from_secs(10),
        notified: false,
    });
    {
        let mut ctx = executor.runtime_ctx.write().await;
        ctx.observation.last_snapshot = Some(snapshot);
    }

    let flow = executor
        .execute_vision_step(&crate::domain::scripts::nodes::vision_node::VisionNode::VisionSearch {
            rule: SearchRule::Txt {
                pattern: "开始".to_string(),
            },
            out_var: "runtime.visionHits".to_string(),
            then_steps: Vec::new(),
        })
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Return));
    clear_runtime_session().await;
}

#[tokio::test]
async fn action_prepare_path_triggers_timeout_detector() {
    let _guard = acquire_runtime_session_test_guard().await;
    install_runtime_policy_for_test(TimeoutAction::SkipCurrentTask).await;
    let mut executor = build_executor();
    executor.last_progress_probe = Some(super::ProgressProbe {
        page_fingerprint: None,
        evidence_signature: "action.prepare".to_string(),
        task_id: None,
        step_id: None,
        stagnant_since: Instant::now() - std::time::Duration::from_secs(10),
        notified: false,
    });

    let flow = executor
        .execute_action_step(
            None,
            0,
            &Action::LaunchApp {
                pkg_name: String::new(),
                activity_name: String::new(),
            },
        )
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Return));
    clear_runtime_session().await;
}

#[tokio::test]
async fn task_control_path_triggers_timeout_detector() {
    let _guard = acquire_runtime_session_test_guard().await;
    install_runtime_policy_for_test(TimeoutAction::SkipCurrentTask).await;
    let mut executor = build_executor();
    executor.last_progress_probe = Some(super::ProgressProbe {
        page_fingerprint: None,
        evidence_signature: "taskControl.setState".to_string(),
        task_id: None,
        step_id: None,
        stagnant_since: Instant::now() - std::time::Duration::from_secs(10),
        notified: false,
    });

    let flow = executor
        .execute_task_control_step(&TaskControl::SetState {
            target: StateTarget::Task { id: UuidV7(700) },
            targets: Vec::new(),
            status: StateStatus::Skip { value: true },
        })
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Return));
    clear_runtime_session().await;
}

#[tokio::test]
async fn task_control_set_state_applies_multiple_targets() {
    let mut executor = build_executor();
    let task_a = UuidV7(701);
    let task_b = UuidV7(702);
    let policy_a = UuidV7(801);
    let policy_b = UuidV7(802);

    let flow = executor
        .execute_task_control_step(&TaskControl::SetState {
            target: StateTarget::Task { id: task_a },
            targets: vec![
                StateTarget::Task { id: task_a },
                StateTarget::Task { id: task_b },
                StateTarget::Policy { id: policy_a },
                StateTarget::Policy { id: policy_b },
            ],
            status: StateStatus::Skip { value: true },
        })
        .await
        .unwrap();

    assert!(matches!(flow, super::ControlFlow::Next));
    let ctx = executor.runtime_ctx.read().await;
    assert!(ctx
        .execution
        .task_states
        .get(&task_a)
        .is_some_and(|state| state.skip_flag));
    assert!(ctx
        .execution
        .task_states
        .get(&task_b)
        .is_some_and(|state| state.skip_flag));
    assert!(ctx
        .execution
        .policy_states
        .get(&policy_a)
        .is_some_and(|state| state.skip_flag));
    assert!(ctx
        .execution
        .policy_states
        .get(&policy_b)
        .is_some_and(|state| state.skip_flag));
}

#[tokio::test]
async fn current_task_condition_matches_target_list() {
    let mut executor = build_executor();
    let task_id = UuidV7(710);
    {
        let mut ctx = executor.runtime_ctx.write().await;
        ctx.execution.current_task = Some(build_task_with_variables(task_id, json!({})));
    }

    let matched = executor
        .evaluate_condition(&ConditionNode::CurrentTaskIn {
            targets: vec![UuidV7(709), task_id],
        })
        .await
        .unwrap();
    let unmatched = executor
        .evaluate_condition(&ConditionNode::CurrentTaskIn {
            targets: vec![UuidV7(711)],
        })
        .await
        .unwrap();

    assert!(matched);
    assert!(!unmatched);
}

#[test]
fn color_compare_matches_background_ring_color() {
    let mut image = RgbaImage::from_pixel(64, 32, Rgba([250, 250, 250, 255]));
    let bbox = BoundingBox::new(12, 8, 28, 20);
    fill_rect(&mut image, &bbox, [210, 30, 30, 255]);
    let item = build_ocr_result("开始", bbox.x1, bbox.y1, bbox.x2, bbox.y2);

    assert!(ScriptExecutor::ocr_item_matches_color(
        &image,
        &item,
        false,
        ScriptExecutor::rgb_to_oklab(&ColorRgb {
            r: 250,
            g: 250,
            b: 250,
        }),
        &ColorCompareMethod::OklabDistance { threshold: 0.03 },
    ));
}

#[test]
fn color_compare_matches_font_color_against_top_three_clusters() {
    let mut image = RgbaImage::from_pixel(96, 48, Rgba([245, 245, 245, 255]));
    let bbox = BoundingBox::new(18, 10, 48, 30);
    let left = BoundingBox::new(18, 10, 27, 30);
    let middle = BoundingBox::new(28, 10, 37, 30);
    let right = BoundingBox::new(38, 10, 48, 30);
    fill_rect(&mut image, &left, [220, 40, 40, 255]);
    fill_rect(&mut image, &middle, [40, 180, 70, 255]);
    fill_rect(&mut image, &right, [40, 90, 220, 255]);
    let item = build_ocr_result("开始", bbox.x1, bbox.y1, bbox.x2, bbox.y2);

    assert!(ScriptExecutor::ocr_item_matches_color(
        &image,
        &item,
        true,
        ScriptExecutor::rgb_to_oklab(&ColorRgb {
            r: 40,
            g: 90,
            b: 220,
        }),
        &ColorCompareMethod::OklabDistance { threshold: 0.04 },
    ));
}
