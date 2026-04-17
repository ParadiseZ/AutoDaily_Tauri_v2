use super::ScriptExecutor;
use crate::domain::scripts::nodes::flow_control::PolicySetResultCompareOp;
use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult};
use image::{Rgba, RgbaImage};
use rhai::serde::to_dynamic;

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
