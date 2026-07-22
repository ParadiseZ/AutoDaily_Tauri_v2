mod auth;
pub(crate) mod dto;
pub(crate) mod profile_cache;
mod script;
mod support;

pub(crate) use auth::{
    backend_get_auth_session, backend_get_cached_profile, backend_get_profile, backend_login,
    backend_logout, backend_redeem_sponsor_code, backend_register, backend_reset_password,
    backend_send_verification_code, backend_update_username,
};
pub(crate) use script::local_scripts_dir;
pub(crate) use script::{
    backend_download_model, backend_download_script, backend_get_script_change_logs,
    backend_get_script_cloud_summary, backend_preflight_download_script,
    backend_preflight_upload_script, backend_search_scripts, backend_upload_model,
    backend_upload_script,
};
pub(crate) use support::{backend_create_feedback, backend_create_script_report};

use crate::api::response::ApiResponse;
use crate::app::app_error::{AppError, AppResult};
use crate::infra::http_client::HttpClient;
use crate::infra::logging::log_trait::Log;
use dto::{AuthRes, BackendApiRes};
use serde::Serialize;

fn trans_api_res<T: Serialize>(api_res: AppResult<BackendApiRes<T>>) -> ApiResponse<T> {
    match api_res {
        Ok(api_res) => {
            if api_res.code == 200 {
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::failed_with_details(
                    None,
                    Some(format_backend_message(
                        &api_res.message,
                        api_res.details.as_ref(),
                    )),
                    api_res.details,
                )
            }
        }
        Err(error) => ApiResponse::error(Some(app_error_message(error))),
    }
}

fn trans_api_res_token(
    client: HttpClient,
    api_res: AppResult<BackendApiRes<AuthRes>>,
) -> ApiResponse<AuthRes> {
    match api_res {
        Ok(api_res) => {
            if api_res.code == 200 {
                if let Some(auth_data) = &api_res.data {
                    let _ = client.set_auth_session(auth_data);
                }
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::failed_with_details(
                    None,
                    Some(format_backend_message(
                        &api_res.message,
                        api_res.details.as_ref(),
                    )),
                    api_res.details,
                )
            }
        }
        Err(error) => ApiResponse::error(Some(app_error_message(error))),
    }
}

fn app_error_message(error: AppError) -> String {
    Log::error(&format!("服务器接口调用失败: {}", error));
    match error {
        AppError::HttpErr { detail, e } => friendly_http_error_message(&detail, &e),
        other => other.to_string(),
    }
}

fn friendly_http_error_message(detail: &str, raw_error: &str) -> String {
    let detail_lower = detail.trim().to_ascii_lowercase();
    let raw_lower = raw_error.trim().to_ascii_lowercase();

    if raw_lower.contains("401") || raw_lower.contains("unauthorized") {
        return "登录状态已失效，请重新登录后重试。".to_string();
    }

    if raw_lower.contains("403") || raw_lower.contains("forbidden") {
        return "当前账号没有权限执行此操作。".to_string();
    }

    if detail_lower.contains("请求发送失败")
        || raw_lower.contains("error sending request for url")
        || raw_lower.contains("connection refused")
        || raw_lower.contains("dns error")
        || raw_lower.contains("failed to connect")
        || raw_lower.contains("tcp connect error")
        || raw_lower.contains("timed out")
        || raw_lower.contains("timeout")
    {
        return "连接服务器失败，请检查服务是否启动或网络是否可用后重试。".to_string();
    }

    if detail_lower.contains("接口返回错误状态码")
        || detail_lower.contains("文件下载返回了失败状态码")
    {
        if raw_lower.contains("404") {
            return "请求的服务器接口暂不可用，请稍后重试。".to_string();
        }

        if raw_lower.contains("5") && raw_lower.contains("internal server error") {
            return "服务器暂时不可用，请稍后重试。".to_string();
        }

        return "服务器处理请求失败，请稍后重试。".to_string();
    }

    if detail_lower.contains("解析响应 json 失败") || detail_lower.contains("解析接口响应失败")
    {
        return "服务器返回的数据异常，请稍后重试。".to_string();
    }

    if detail_lower.contains("请求下载文件失败") {
        return "下载失败，请检查网络后重试。".to_string();
    }

    "服务器暂时不可用，请稍后重试。".to_string()
}

fn format_backend_message(message: &str, details: Option<&serde_json::Value>) -> String {
    let detail_lines = extract_validation_issue_lines(details);
    if detail_lines.is_empty() {
        return message.to_string();
    }

    format!("{}\n{}", message, detail_lines.join("\n"))
}

fn extract_validation_issue_lines(details: Option<&serde_json::Value>) -> Vec<String> {
    let Some(details) = details else {
        return Vec::new();
    };
    let Some(issues) = details.get("issues").and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    issues
        .iter()
        .filter_map(|issue| {
            let message = issue.get("message").and_then(|value| value.as_str())?;
            let path = issue
                .get("path")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if path.is_empty() {
                Some(format!("- {}", message))
            } else {
                Some(format!("- {}: {}", path, message))
            }
        })
        .collect()
}
