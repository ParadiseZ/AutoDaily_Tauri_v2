use ad_kernel::ids::{DeviceId, ScriptId, TaskId};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

fn default_schema_version() -> u32 {
    1
}

fn default_baseline_dir() -> PathBuf {
    PathBuf::from("baselines")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSuiteConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub database_dir: Option<PathBuf>,
    pub script_id: ScriptId,
    #[serde(default)]
    pub device_id: Option<DeviceId>,
    #[serde(default = "default_baseline_dir")]
    pub baseline_dir: PathBuf,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    pub scenarios: Vec<TestScenarioConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestScenarioConfig {
    pub name: String,
    #[serde(default)]
    pub task_id: Option<TaskId>,
    #[serde(default)]
    pub template_values: Option<Value>,
}
