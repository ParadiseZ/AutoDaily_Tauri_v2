use crate::api::response::ApiResponse;
use crate::app::app_error::AppResult;
use crate::infra::http_client::HttpClient;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, command};
use uuid::Uuid;

use super::dto::BackendApiRes;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptReportReq {
    pub category: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackReq {
    pub target_type: String,
    pub script_id: Option<String>,
    pub category: String,
    pub title: String,
    pub description: String,
    pub reproduction_steps: Option<String>,
    pub expected_behavior: Option<String>,
    pub actual_behavior: Option<String>,
    pub runtime_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendFeedbackReq<'a> {
    target_type: &'a str,
    script_id: Option<&'a str>,
    category: &'a str,
    title: &'a str,
    description: &'a str,
    reproduction_steps: Option<&'a str>,
    expected_behavior: Option<&'a str>,
    actual_behavior: Option<&'a str>,
    app_version: &'static str,
    platform: &'static str,
    os_version: &'static str,
    runtime_type: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportSubmissionResult {
    pub id: String,
    pub uploaded_screenshots: usize,
    pub failed_screenshots: usize,
}

#[command]
pub async fn backend_create_script_report(
    app_handle: AppHandle,
    script_id: String,
    req: ScriptReportReq,
    screenshot_paths: Vec<String>,
) -> ApiResponse<SupportSubmissionResult> {
    let script_id = match Uuid::parse_str(&script_id) {
        Ok(value) => value,
        Err(_) => return ApiResponse::error(Some("云端脚本 ID 无效，无法提交举报。".to_string())),
    };
    let client = HttpClient::new(app_handle);
    let endpoint = format!("/scripts/{script_id}/reports");
    submit_with_screenshots(client, &endpoint, &req, "report", screenshot_paths).await
}

#[command]
pub async fn backend_create_feedback(
    app_handle: AppHandle,
    req: FeedbackReq,
    screenshot_paths: Vec<String>,
) -> ApiResponse<SupportSubmissionResult> {
    if req.target_type == "script" {
        let Some(script_id) = req.script_id.as_deref() else {
            return ApiResponse::error(Some("脚本反馈缺少云端脚本 ID。".to_string()));
        };
        if Uuid::parse_str(script_id).is_err() {
            return ApiResponse::error(Some("云端脚本 ID 无效，无法提交反馈。".to_string()));
        }
    } else if req.target_type != "product" {
        return ApiResponse::error(Some("不支持的反馈目标。".to_string()));
    }
    let client = HttpClient::new(app_handle);
    let body = BackendFeedbackReq {
        target_type: &req.target_type,
        script_id: req.script_id.as_deref(),
        category: &req.category,
        title: &req.title,
        description: &req.description,
        reproduction_steps: req.reproduction_steps.as_deref(),
        expected_behavior: req.expected_behavior.as_deref(),
        actual_behavior: req.actual_behavior.as_deref(),
        app_version: env!("CARGO_PKG_VERSION"),
        platform: "desktop",
        os_version: std::env::consts::OS,
        runtime_type: req.runtime_type.as_deref(),
    };
    submit_with_screenshots(client, "/feedback", &body, "feedback", screenshot_paths).await
}

async fn submit_with_screenshots<B: Serialize>(
    client: HttpClient,
    endpoint: &str,
    body: &B,
    owner_type: &str,
    screenshot_paths: Vec<String>,
) -> ApiResponse<SupportSubmissionResult> {
    let response: AppResult<BackendApiRes<serde_json::Value>> =
        client.post_api_res(endpoint, body).await;
    let response = match response {
        Ok(value) if value.code == 200 => value,
        Ok(value) => {
            return ApiResponse::failed_with_details(
                None,
                Some(super::format_backend_message(
                    &value.message,
                    value.details.as_ref(),
                )),
                value.details,
            );
        }
        Err(error) => return ApiResponse::error(Some(super::app_error_message(error))),
    };
    let Some(id) = response
        .data
        .as_ref()
        .and_then(|value| value.get("id"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
    else {
        return ApiResponse::error(Some("服务器没有返回工单编号，请稍后重试。".to_string()));
    };

    let mut uploaded = 0;
    let mut failed = 0;
    for path in normalize_screenshot_paths(screenshot_paths) {
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("screenshot.png");
        let upload_endpoint = format!("/attachments/{owner_type}/{id}");
        let upload: AppResult<BackendApiRes<serde_json::Value>> = client
            .upload_file(&upload_endpoint, &path, "file", file_name)
            .await;
        match upload {
            Ok(value) if value.code == 200 => uploaded += 1,
            _ => failed += 1,
        }
    }

    ApiResponse::success(
        Some(SupportSubmissionResult {
            id,
            uploaded_screenshots: uploaded,
            failed_screenshots: failed,
        }),
        Some(response.message),
    )
}

fn normalize_screenshot_paths(paths: Vec<String>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .take(5)
        .map(PathBuf::from)
        .filter(|path| is_supported_screenshot(path) && path.is_file())
        .collect()
}

fn is_supported_screenshot(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png")
    )
}

#[cfg(test)]
mod tests {
    use super::is_supported_screenshot;
    use std::path::Path;

    #[test]
    fn screenshot_extensions_are_restricted_to_images() {
        assert!(is_supported_screenshot(Path::new("issue.PNG")));
        assert!(is_supported_screenshot(Path::new("issue.jpeg")));
        assert!(!is_supported_screenshot(Path::new("runtime.log")));
        assert!(!is_supported_screenshot(Path::new("archive.zip")));
    }
}
