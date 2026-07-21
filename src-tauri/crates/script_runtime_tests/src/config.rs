use ad_kernel::ids::TaskId;
use child_runner::testkit::TestVisionFrame;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn default_schema_version() -> u32 {
    1
}

fn default_screen_size() -> [u32; 2] {
    [1280, 720]
}

fn default_extensions() -> Vec<String> {
    vec!["png".to_string(), "jpg".to_string(), "jpeg".to_string()]
}

fn default_captures_per_case() -> usize {
    1
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSuiteConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub name: String,
    pub bundle: PathBuf,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    pub scenarios: Vec<TaskScenarioConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskScenarioConfig {
    pub name: String,
    #[serde(default)]
    pub task_id: Option<TaskId>,
    #[serde(default)]
    pub task_name: Option<String>,
    #[serde(default = "default_screen_size")]
    pub screen_size: [u32; 2],
    #[serde(default)]
    pub screenshots: Option<ImageSourceConfig>,
    #[serde(default)]
    pub vision: VisionInputConfig,
    #[serde(default)]
    pub template_values: Option<Value>,
    #[serde(default)]
    pub expected: Value,
    #[serde(default)]
    pub expected_by_image: BTreeMap<String, Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "mode"
)]
pub enum VisionInputConfig {
    #[default]
    None,
    Injected {
        #[serde(default)]
        capture_frames: Vec<TestVisionFrame>,
        #[serde(default)]
        detect_frames: Vec<TestVisionFrame>,
        #[serde(default)]
        ocr_frames: Vec<TestVisionFrame>,
    },
    Real {
        images: ImageSourceConfig,
        #[serde(default = "default_captures_per_case")]
        captures_per_case: usize,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSourceConfig {
    pub path: PathBuf,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,
}
