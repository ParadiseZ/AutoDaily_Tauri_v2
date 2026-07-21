use crate::config::{TaskScenarioConfig, TestSuiteConfig, VisionInputConfig};
use crate::image_source::{collect_image_paths, load_image};
use ad_kernel::ids::TaskId;
use child_runner::testkit::{TestTaskRunRequest, run_task_test};
use domain_script::{ScriptTaskProfile, Step};
use image::{Rgba, RgbaImage};
use runner_protocol::message::ScriptBundleSnapshot;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteReport {
    pub name: String,
    pub passed: bool,
    pub coverage: CoverageReport,
    pub scenarios: Vec<ScenarioReport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageReport {
    pub passed: bool,
    pub discovered: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioReport {
    pub name: String,
    pub task_id: Option<TaskId>,
    pub task_name: Option<String>,
    pub image: Option<PathBuf>,
    pub passed: bool,
    pub failures: Vec<String>,
    pub result: Option<Value>,
}

pub async fn run_config_path(path: &Path) -> Result<SuiteReport, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("读取测试配置失败[{}]: {error}", path.display()))?;
    let config: TestSuiteConfig = serde_json::from_str(&content)
        .map_err(|error| format!("解析测试配置失败[{}]: {error}", path.display()))?;
    let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
    run_config(config, config_dir).await
}

pub async fn run_config(config: TestSuiteConfig, config_dir: &Path) -> Result<SuiteReport, String> {
    if config.schema_version != 1 {
        return Err(format!("不支持的测试配置版本: {}", config.schema_version));
    }
    let bundle_path = resolve_path(config_dir, &config.bundle);
    let bundle: ScriptBundleSnapshot = read_json(&bundle_path, "完整脚本 bundle")?;
    let tasks: Vec<ScriptTaskProfile> = serde_json::from_str(&bundle.tasks_json)
        .map_err(|error| format!("解析 bundle.tasks_json 失败: {error}"))?;
    let coverage = build_coverage_report(&tasks, &config.required_capabilities)?;

    let mut reports = Vec::new();
    for scenario in &config.scenarios {
        let task = resolve_task(&tasks, scenario)?;
        let image_cases = scenario_images(config_dir, scenario)?;
        for image in image_cases {
            reports.push(run_scenario(config_dir, &bundle, task, scenario, image).await);
        }
    }
    let passed = coverage.passed && reports.iter().all(|report| report.passed);
    Ok(SuiteReport {
        name: config.name,
        passed,
        coverage,
        scenarios: reports,
    })
}

fn build_coverage_report(
    tasks: &[ScriptTaskProfile],
    required: &[String],
) -> Result<CoverageReport, String> {
    let mut discovered = BTreeSet::new();
    for task in tasks {
        collect_step_capabilities(&task.task.steps, &mut discovered)?;
    }
    let missing = required
        .iter()
        .filter(|capability| !discovered.contains(capability.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    Ok(CoverageReport {
        passed: missing.is_empty(),
        discovered: discovered.into_iter().collect(),
        missing,
    })
}

fn collect_step_capabilities(
    steps: &[Step],
    capabilities: &mut BTreeSet<String>,
) -> Result<(), String> {
    let value = serde_json::to_value(steps)
        .map_err(|error| format!("序列化步骤用于能力扫描失败: {error}"))?;
    walk_step_values(&value, capabilities);
    Ok(())
}

fn walk_step_values(value: &Value, capabilities: &mut BTreeSet<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                walk_step_values(value, capabilities);
            }
        }
        Value::Object(object) => {
            if let Some(op) = object.get("op").and_then(Value::as_str) {
                let capability = match op {
                    "sequence" => Some("sequence".to_string()),
                    "action" => tagged_capability(object.get("a"), "ac", "action"),
                    "dataHanding" => tagged_capability(object.get("a"), "type", "dataHanding"),
                    "flowControl" => tagged_capability(object.get("a"), "type", "flowControl"),
                    "taskControl" => tagged_capability(object.get("a"), "type", "taskControl"),
                    "vision" => tagged_capability(object.get("a"), "type", "vision"),
                    _ => Some(format!("unknown.{op}")),
                };
                if let Some(capability) = capability {
                    capabilities.insert(capability);
                }
            }
            for child in object.values() {
                walk_step_values(child, capabilities);
            }
        }
        _ => {}
    }
}

fn tagged_capability(value: Option<&Value>, tag: &str, prefix: &str) -> Option<String> {
    value?
        .get(tag)?
        .as_str()
        .map(|kind| format!("{prefix}.{kind}"))
}

fn resolve_task<'a>(
    tasks: &'a [ScriptTaskProfile],
    scenario: &TaskScenarioConfig,
) -> Result<&'a ScriptTaskProfile, String> {
    match (scenario.task_id, scenario.task_name.as_deref()) {
        (Some(task_id), _) => tasks
            .iter()
            .find(|task| task.id == task_id)
            .ok_or_else(|| format!("场景[{}]找不到任务 id={task_id}", scenario.name)),
        (None, Some(task_name)) => {
            let matches = tasks
                .iter()
                .filter(|task| task.name == task_name)
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [task] => Ok(*task),
                [] => Err(format!("场景[{}]找不到任务[{task_name}]", scenario.name)),
                _ => Err(format!(
                    "场景[{}]存在多个同名任务[{}]，请改用 taskId",
                    scenario.name, task_name
                )),
            }
        }
        (None, None) => Err(format!(
            "场景[{}]必须配置 taskId 或 taskName",
            scenario.name
        )),
    }
}

fn scenario_images(
    config_dir: &Path,
    scenario: &TaskScenarioConfig,
) -> Result<Vec<Option<PathBuf>>, String> {
    match &scenario.vision {
        VisionInputConfig::Real { images, .. } => Ok(collect_image_paths(config_dir, images)?
            .into_iter()
            .map(Some)
            .collect()),
        _ => Ok(vec![None]),
    }
}

async fn run_scenario(
    config_dir: &Path,
    bundle: &ScriptBundleSnapshot,
    task: &ScriptTaskProfile,
    scenario: &TaskScenarioConfig,
    image_path: Option<PathBuf>,
) -> ScenarioReport {
    let expected = image_path
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .and_then(|name| scenario.expected_by_image.get(name))
        .unwrap_or(&scenario.expected);
    match build_request(
        config_dir,
        bundle.clone(),
        task.id,
        scenario,
        image_path.as_deref(),
    ) {
        Ok(request) => match run_task_test(request).await {
            Ok(result) => {
                let mut failures = Vec::new();
                assert_json_subset("$", expected, &result, &mut failures);
                ScenarioReport {
                    name: scenario.name.clone(),
                    task_id: Some(task.id),
                    task_name: Some(task.name.clone()),
                    image: image_path,
                    passed: failures.is_empty(),
                    failures,
                    result: Some(result),
                }
            }
            Err(error) => failed_scenario(scenario, task, image_path, error),
        },
        Err(error) => failed_scenario(scenario, task, image_path, error),
    }
}

fn build_request(
    config_dir: &Path,
    bundle: ScriptBundleSnapshot,
    task_id: TaskId,
    scenario: &TaskScenarioConfig,
    real_image_path: Option<&Path>,
) -> Result<TestTaskRunRequest, String> {
    let mut screenshots = match scenario.screenshots.as_ref() {
        Some(source) => collect_image_paths(config_dir, source)?
            .iter()
            .map(|path| load_image(path))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let (capture_frames, detect_frames, ocr_frames, use_real_vision) = match &scenario.vision {
        VisionInputConfig::None => (Vec::new(), Vec::new(), Vec::new(), false),
        VisionInputConfig::Injected {
            capture_frames,
            detect_frames,
            ocr_frames,
        } => (
            capture_frames.clone(),
            detect_frames.clone(),
            ocr_frames.clone(),
            false,
        ),
        VisionInputConfig::Real {
            captures_per_case, ..
        } => {
            let path = real_image_path.ok_or_else(|| "真实视觉任务缺少图片".to_string())?;
            let image = load_image(path)?;
            screenshots = (0..(*captures_per_case).max(1))
                .map(|_| image.clone())
                .collect();
            (Vec::new(), Vec::new(), Vec::new(), true)
        }
    };
    if screenshots.is_empty() {
        let [width, height] = scenario.screen_size;
        screenshots.push(RgbaImage::from_pixel(
            width.max(1),
            height.max(1),
            Rgba([0, 0, 0, 255]),
        ));
    }
    let template_values_json = scenario
        .template_values
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| format!("序列化 templateValues 失败: {error}"))?;
    Ok(TestTaskRunRequest {
        bundle,
        task_id,
        screenshots,
        capture_vision_frames: capture_frames,
        detect_vision_frames: detect_frames,
        ocr_vision_frames: ocr_frames,
        use_real_vision,
        template_values_json,
    })
}

fn assert_json_subset(path: &str, expected: &Value, actual: &Value, failures: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for (key, expected_value) in expected {
                let child_path = format!("{path}.{key}");
                match actual.get(key) {
                    Some(actual_value) => {
                        assert_json_subset(&child_path, expected_value, actual_value, failures)
                    }
                    None => failures.push(format!("缺少结果字段: {child_path}")),
                }
            }
        }
        (Value::Array(expected), Value::Array(actual)) if expected == actual => {}
        (expected, actual) if expected == actual => {}
        (expected, actual) => failures.push(format!(
            "结果不一致[{path}]: expected={expected}, actual={actual}"
        )),
    }
}

fn resolve_path(config_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("读取{label}失败[{}]: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("解析{label}失败[{}]: {error}", path.display()))
}

fn failed_scenario(
    scenario: &TaskScenarioConfig,
    task: &ScriptTaskProfile,
    image: Option<PathBuf>,
    error: String,
) -> ScenarioReport {
    ScenarioReport {
        name: scenario.name.clone(),
        task_id: Some(task.id),
        task_name: Some(task.name.clone()),
        image,
        passed: false,
        failures: vec![error],
        result: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{assert_json_subset, collect_step_capabilities};
    use domain_script::{Action, Step, StepKind};
    use serde_json::json;
    use std::collections::BTreeSet;

    fn step(kind: StepKind) -> Step {
        Step {
            id: None,
            source_id: None,
            target_id: None,
            label: None,
            skip_flag: false,
            kind,
        }
    }

    #[test]
    fn expected_json_is_a_recursive_subset_of_the_runtime_result() {
        let expected = json!({ "execution": { "outcome": "completed" } });
        let actual = json!({
            "execution": { "outcome": "completed", "error": null },
            "operations": []
        });
        let mut failures = Vec::new();
        assert_json_subset("$", &expected, &actual, &mut failures);
        assert!(failures.is_empty());
    }

    #[test]
    fn capability_scan_finds_steps_nested_in_sequences() {
        let steps = vec![step(StepKind::Sequence {
            steps: vec![step(StepKind::Action {
                exec_max: 1,
                a: Action::Back,
            })],
        })];
        let mut capabilities = BTreeSet::new();

        collect_step_capabilities(&steps, &mut capabilities).unwrap();

        assert!(capabilities.contains("sequence"));
        assert!(capabilities.contains("action.back"));
    }
}
