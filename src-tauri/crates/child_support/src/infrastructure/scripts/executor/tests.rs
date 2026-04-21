use super::ScriptExecutor;
use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
use crate::domain::devices::device_schedule::TaskCycle;
use crate::domain::scripts::nodes::data_handing::{ColorCompareMethod, ColorRgb};
use crate::domain::scripts::nodes::flow_control::PolicySetResultCompareOp;
use crate::domain::scripts::script_task::{
    ScriptTask, ScriptTaskTable, TaskRowType, TaskTone, TaskTriggerMode,
};
use crate::domain::scripts::script_variable::{
    ScriptVariableDef, ScriptVariableNamespace, ScriptVariableSourceType, ScriptVariableValueType,
};
use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult};
use crate::infrastructure::context::runtime_context::RuntimeContext;
use crate::infrastructure::core::{TaskId, UuidV7};
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeExecutionPolicy, RuntimeVisionTextCachePolicy, TimeoutAction,
};
use crate::infrastructure::vision::ocr_service::OcrService;
use image::{Rgba, RgbaImage};
use rhai::serde::to_dynamic;
use serde_json::{json, Value};
use sqlx::types::Json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

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

fn build_executor() -> ScriptExecutor {
    let runtime_ctx = Arc::new(RwLock::new(RuntimeContext::new(
        UuidV7(1),
        RunTarget::FullScript {
            script_id: UuidV7(1),
        },
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
