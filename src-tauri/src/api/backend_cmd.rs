use crate::api::api_response::ApiResponse;
use crate::api::backend_dto::*;
use crate::app::app_error::AppResult;
use crate::infrastructure::http_client::HttpClient;
use tauri::{command, AppHandle};

#[command]
pub async fn backend_send_verification_code(
    app_handle: AppHandle,
    email: String,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let url = format!("/auth/send-verification-code?email={}", email);
    let res: AppResult<BackendApiRes<String>> = client.post(&url, &()).await;
    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::error(Some(api_res.message))
            }
        }
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

#[command]
pub async fn backend_register(
    app_handle: AppHandle,
    req: RegisterReq,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<String>> = client.post("/auth/register", &req).await;
    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::error(Some(api_res.message))
            }
        }
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

#[command]
pub async fn backend_login(
    app_handle: AppHandle,
    req: LoginReq,
) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/auth/login", &req).await;
    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                if let Some(auth_data) = &api_res.data {
                    // Save JWT locally via Store
                    let _ = client.set_jwt_token(&auth_data.access_token);
                }
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::error(Some(api_res.message))
            }
        }
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

#[command]
pub async fn backend_logout(app_handle: AppHandle) -> ApiResponse<()> {
    let client = HttpClient::new(app_handle);
    let _ = client.clear_jwt_token();
    ApiResponse::success(None, Some("登出成功".to_string()))
}
