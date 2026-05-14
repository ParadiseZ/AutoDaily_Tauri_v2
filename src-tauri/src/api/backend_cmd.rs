mod auth_profile;
mod script_transfer;

pub use auth_profile::{
    backend_get_auth_session, backend_get_cached_profile, backend_get_profile, backend_login,
    backend_logout, backend_redeem_sponsor_code, backend_register, backend_reset_password,
    backend_send_verification_code, backend_update_username,
};
pub use script_transfer::{
    backend_download_model, backend_download_script, backend_get_script_change_logs,
    backend_get_script_cloud_summary, backend_preflight_download_script,
    backend_preflight_upload_script, backend_search_scripts, backend_upload_model,
    backend_upload_script,
};

use crate::api::api_response::ApiResponse;
use crate::api::backend_dto::{AuthRes, BackendApiRes};
use crate::app::app_error::{AppError, AppResult};
use crate::infrastructure::core::Serialize;
use crate::infrastructure::http_client::HttpClient;

fn trans_api_res<T: Serialize>(api_res: AppResult<BackendApiRes<T>>) -> ApiResponse<T> {
    match api_res {
        Ok(api_res) => {
            if api_res.code == 200 {
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::failed_with_details(
                    None,
                    Some(format_backend_message(&api_res.message, api_res.details.as_ref())),
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
                    Some(format_backend_message(&api_res.message, api_res.details.as_ref())),
                    api_res.details,
                )
            }
        }
        Err(error) => ApiResponse::error(Some(app_error_message(error))),
    }
}

fn app_error_message(error: AppError) -> String {
    match error {
        AppError::HttpErr { e, .. } if !e.trim().is_empty() => e,
        other => other.to_string(),
    }
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
            let path = issue.get("path").and_then(|value| value.as_str()).unwrap_or("");
            if path.is_empty() {
                Some(format!("- {}", message))
            } else {
                Some(format!("- {}: {}", path, message))
            }
        })
        .collect()
}
