use crate::config::{TestScenarioConfig, TestSuiteConfig};
use child_runner::testkit::{TestScriptRunRequest, run_script_test};
use domain_device::DeviceProfile;
use domain_script::{ScriptTaskProfile, Step};
use infra_sqlite::{
    get_all_devices, get_device, get_script, init_db_with_path, list_group_policy_links,
    list_policies, list_policy_groups, list_policy_sets, list_script_tasks, list_set_group_links,
};
use runner_protocol::message::ScriptBundleSnapshot;
use serde::Serialize;
use serde_json::{Map, Number, Value};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static DATABASE_DIR: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RunMode {
    Record,
    Verify,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteReport {
    pub name: String,
    pub mode: RunMode,
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
    pub task_id: Option<ad_kernel::ids::TaskId>,
    pub passed: bool,
    pub failures: Vec<String>,
    pub baseline_path: PathBuf,
    pub result: Option<Value>,
}

pub async fn run_config_path(path: &Path, mode: RunMode) -> Result<SuiteReport, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("读取测试配置失败[{}]: {error}", path.display()))?;
    let config: TestSuiteConfig = serde_json::from_str(&content)
        .map_err(|error| format!("解析测试配置失败[{}]: {error}", path.display()))?;
    let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
    run_config(config, config_dir, mode).await
}

pub async fn run_config(
    config: TestSuiteConfig,
    config_dir: &Path,
    mode: RunMode,
) -> Result<SuiteReport, String> {
    if config.schema_version != 1 {
        return Err(format!("不支持的测试配置版本: {}", config.schema_version));
    }
    let database_dir = resolve_database_dir(config_dir, config.database_dir.as_deref())?;
    let bundle = load_bundle(&database_dir, config.script_id).await?;
    let device = resolve_device(config.device_id).await?;
    let tasks: Vec<ScriptTaskProfile> = serde_json::from_str(&bundle.tasks_json)
        .map_err(|error| format!("解析运行时 tasks_json 失败: {error}"))?;
    let coverage = build_coverage_report(&tasks, &config.required_capabilities)?;
    let baseline_dir =
        resolve_path(config_dir, &config.baseline_dir).join(config.script_id.to_string());

    let mut reports = Vec::new();
    let mut baseline_paths = HashSet::new();
    for scenario in &config.scenarios {
        let baseline_path = scenario_baseline_path(&baseline_dir, scenario);
        if !baseline_paths.insert(baseline_path.clone()) {
            return Err(format!(
                "多个场景使用了同一个基准文件，请勿重复配置相同 taskId: {}",
                baseline_path.display()
            ));
        }
        reports.push(run_scenario(&bundle, &device, scenario, baseline_path, mode).await);
    }

    let passed = coverage.passed && reports.iter().all(|report| report.passed);
    Ok(SuiteReport {
        name: config.name,
        mode,
        passed,
        coverage,
        scenarios: reports,
    })
}

async fn load_bundle(
    database_dir: &Path,
    script_id: ad_kernel::ids::ScriptId,
) -> Result<ScriptBundleSnapshot, String> {
    init_database(database_dir).await?;
    let script = get_script(script_id)
        .await?
        .ok_or_else(|| format!("测试脚本[{script_id}]不存在"))?;
    let tasks = list_script_tasks(script_id).await?;
    let policies = list_policies(script_id).await?;
    let policy_groups = list_policy_groups(script_id).await?;
    let policy_sets = list_policy_sets(script_id).await?;
    let group_policies = list_group_policy_links(script_id).await?;
    let set_groups = list_set_group_links(script_id).await?;
    Ok(ScriptBundleSnapshot {
        script_id,
        script_json: serialize(&script, "script")?,
        tasks_json: serialize(&tasks, "tasks")?,
        policies_json: serialize(&policies, "policies")?,
        policy_groups_json: serialize(&policy_groups, "policy groups")?,
        policy_sets_json: serialize(&policy_sets, "policy sets")?,
        group_policies_json: serialize(&group_policies, "group policies")?,
        set_groups_json: serialize(&set_groups, "set groups")?,
    })
}

async fn init_database(database_dir: &Path) -> Result<(), String> {
    if let Some(initialized_dir) = DATABASE_DIR.get() {
        if initialized_dir == database_dir {
            return Ok(());
        }
        return Err(format!(
            "同一测试进程只能连接一个数据库目录；已连接[{}]，不能再连接[{}]",
            initialized_dir.display(),
            database_dir.display()
        ));
    }
    init_db_with_path(database_dir)
        .await
        .map_err(|error| format!("初始化测试脚本数据库失败: {error}"))?;
    let _ = DATABASE_DIR.set(database_dir.to_path_buf());
    Ok(())
}

async fn resolve_device(
    device_id: Option<ad_kernel::ids::DeviceId>,
) -> Result<DeviceProfile, String> {
    if let Some(device_id) = device_id {
        return get_device(device_id)
            .await?
            .ok_or_else(|| format!("测试设备[{device_id}]不存在"));
    }
    let devices = get_all_devices().await?;
    match devices.as_slice() {
        [device] => Ok(device.clone()),
        [] => Err("当前数据库没有设备，请先在 AutoDaily 中配置测试设备".to_string()),
        _ => {
            let choices = devices
                .iter()
                .map(|device| format!("{}({})", device.config.device_name, device.id))
                .collect::<Vec<_>>()
                .join("、");
            Err(format!(
                "当前数据库存在多个设备，请在配置中填写 deviceId：{choices}"
            ))
        }
    }
}

fn serialize(value: &impl Serialize, label: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("序列化 {label} 失败: {error}"))
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

async fn run_scenario(
    bundle: &ScriptBundleSnapshot,
    device: &DeviceProfile,
    scenario: &TestScenarioConfig,
    baseline_path: PathBuf,
    mode: RunMode,
) -> ScenarioReport {
    let request = match build_request(bundle.clone(), device, scenario) {
        Ok(request) => request,
        Err(error) => return failed_scenario(scenario, baseline_path, error),
    };
    let result = match run_script_test(request).await {
        Ok(result) => normalize_value(result),
        Err(error) => return failed_scenario(scenario, baseline_path, error),
    };
    if result.pointer("/execution/outcome") == Some(&Value::String("failed".to_string())) {
        let error = result
            .pointer("/execution/error")
            .cloned()
            .unwrap_or(Value::Null);
        return ScenarioReport {
            name: scenario.name.clone(),
            task_id: scenario.task_id,
            passed: false,
            failures: vec![format!("正式执行失败: {error}")],
            baseline_path,
            result: Some(result),
        };
    }

    let failures = match mode {
        RunMode::Record => match write_baseline(&baseline_path, &result) {
            Ok(()) => Vec::new(),
            Err(error) => vec![error],
        },
        RunMode::Verify => verify_baseline(&baseline_path, &result),
    };
    ScenarioReport {
        name: scenario.name.clone(),
        task_id: scenario.task_id,
        passed: failures.is_empty(),
        failures,
        baseline_path,
        result: Some(result),
    }
}

fn build_request(
    bundle: ScriptBundleSnapshot,
    device: &DeviceProfile,
    scenario: &TestScenarioConfig,
) -> Result<TestScriptRunRequest, String> {
    let template_values_json = scenario
        .template_values
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| format!("序列化 templateValues 失败: {error}"))?;
    Ok(TestScriptRunRequest {
        bundle,
        task_id: scenario.task_id,
        device_id: device.id,
        device_config: Some(device.config.clone()),
        template_values_json,
    })
}

fn scenario_baseline_path(baseline_dir: &Path, scenario: &TestScenarioConfig) -> PathBuf {
    let file_name = scenario
        .task_id
        .map(|task_id| format!("task-{task_id}.json"))
        .unwrap_or_else(|| "full-script.json".to_string());
    baseline_dir.join(file_name)
}

fn write_baseline(path: &Path, result: &Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("基准路径没有父目录: {}", path.display()))?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("创建基准目录失败[{}]: {error}", parent.display()))?;
    let content = serde_json::to_string_pretty(result)
        .map_err(|error| format!("序列化基准结果失败: {error}"))?;
    std::fs::write(path, format!("{content}\n"))
        .map_err(|error| format!("写入基准结果失败[{}]: {error}", path.display()))
}

fn verify_baseline(path: &Path, actual: &Value) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            return vec![format!(
                "读取基准结果失败[{}]: {error}；请先运行 record",
                path.display()
            )];
        }
    };
    let expected = match serde_json::from_str::<Value>(&content) {
        Ok(value) => value,
        Err(error) => {
            return vec![format!("解析基准结果失败[{}]: {error}", path.display())];
        }
    };
    let mut differences = Vec::new();
    diff_values("$", &expected, actual, &mut differences);
    differences
}

fn normalize_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut normalized = Map::new();
            for (key, value) in object {
                if key == "score" {
                    continue;
                }
                normalized.insert(key, normalize_value(value));
            }
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(normalize_value).collect()),
        Value::Number(number) if number.is_f64() => {
            let Some(value) = number.as_f64() else {
                return Value::Number(number);
            };
            let rounded = (value * 10_000.0).round() / 10_000.0;
            Number::from_f64(rounded)
                .map(Value::Number)
                .unwrap_or(Value::Number(number))
        }
        other => other,
    }
}

fn diff_values(path: &str, expected: &Value, actual: &Value, differences: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for (key, expected_value) in expected {
                let child_path = format!("{path}.{key}");
                match actual.get(key) {
                    Some(actual_value) => {
                        diff_values(&child_path, expected_value, actual_value, differences)
                    }
                    None => differences.push(format!("缺少结果字段: {child_path}")),
                }
            }
            for key in actual.keys() {
                if !expected.contains_key(key) {
                    differences.push(format!("出现额外结果字段: {path}.{key}"));
                }
            }
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                differences.push(format!(
                    "数组长度不同[{path}]: expected={}, actual={}",
                    expected.len(),
                    actual.len()
                ));
            }
            for (index, (expected, actual)) in expected.iter().zip(actual.iter()).enumerate() {
                diff_values(&format!("{path}[{index}]"), expected, actual, differences);
            }
        }
        _ if expected == actual => {}
        _ => differences.push(format!(
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

fn resolve_database_dir(config_dir: &Path, configured: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = configured {
        return Ok(resolve_path(config_dir, path));
    }
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("com.smart.autodaily"))
        .ok_or_else(|| "未配置 databaseDir，且系统不存在 APPDATA 环境变量".to_string())
}

fn failed_scenario(
    scenario: &TestScenarioConfig,
    baseline_path: PathBuf,
    error: String,
) -> ScenarioReport {
    ScenarioReport {
        name: scenario.name.clone(),
        task_id: scenario.task_id,
        passed: false,
        failures: vec![error],
        baseline_path,
        result: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_step_capabilities, diff_values, normalize_value, verify_baseline, write_baseline,
    };
    use domain_script::{Action, Step, StepKind};
    use serde_json::json;
    use std::collections::BTreeSet;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn normalization_removes_vision_scores_and_rounds_floats() {
        let value = normalize_value(json!({
            "score": 0.998765,
            "center": 1.234567,
            "nested": [{ "score": [0.9], "text": "开始" }]
        }));

        assert_eq!(
            value,
            json!({ "center": 1.2346, "nested": [{ "text": "开始" }] })
        );
    }

    #[test]
    fn exact_diff_reports_changed_and_added_fields() {
        let expected = json!({ "variables": { "runtime.result": "matched" } });
        let actual = json!({
            "variables": {
                "runtime.result": "missed",
                "runtime.extra": true
            }
        });
        let mut differences = Vec::new();

        diff_values("$", &expected, &actual, &mut differences);

        assert!(
            differences
                .iter()
                .any(|item| item.contains("runtime.result"))
        );
        assert!(
            differences
                .iter()
                .any(|item| item.contains("runtime.extra"))
        );
    }

    #[test]
    fn recorded_baseline_verifies_until_the_result_changes() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "autodaily-script-runtime-baseline-{}-{unique}",
            std::process::id()
        ));
        let path = directory.join("task.json");
        let recorded = json!({ "execution": { "outcome": "completed" } });

        write_baseline(&path, &recorded).unwrap();
        assert!(verify_baseline(&path, &recorded).is_empty());

        let differences = verify_baseline(&path, &json!({ "execution": { "outcome": "stopped" } }));
        assert!(differences.iter().any(|item| item.contains("outcome")));

        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&directory).unwrap();
    }
}
