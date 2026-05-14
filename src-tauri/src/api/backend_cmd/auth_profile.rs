use super::{app_error_message, format_backend_message, trans_api_res, trans_api_res_token};
use crate::api::api_response::ApiResponse;
use crate::api::backend_dto::*;
use crate::api::infrastructure::profile_cache::{
    clear_cached_user_profile, load_cached_profile_for_current_session, persist_cached_user_profile,
};
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
    trans_api_res(res)
}

#[command]
pub async fn backend_register(app_handle: AppHandle, req: RegisterReq) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/auth/register", &req).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_reset_password(
    app_handle: AppHandle,
    req: ResetPasswordReq,
) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/auth/reset-password", &req).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_login(app_handle: AppHandle, req: LoginReq) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/auth/login", &req).await;
    trans_api_res_token(client, res)
}

#[command]
pub async fn backend_get_auth_session(app_handle: AppHandle) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    ApiResponse::success(client.get_auth_session(), None)
}

#[command]
pub async fn backend_logout(app_handle: AppHandle) -> ApiResponse<()> {
    let client = HttpClient::new(app_handle.clone());
    let _: AppResult<BackendApiRes<String>> = client.post("/auth/logout", &()).await;
    let _ = client.clear_auth_session();
    clear_cached_user_profile(&app_handle);
    ApiResponse::success(None, Some("登出成功".to_string()))
}

#[command]
pub async fn backend_get_cached_profile(app_handle: AppHandle) -> ApiResponse<serde_json::Value> {
    ApiResponse::success(load_cached_profile_for_current_session(&app_handle), None)
}

#[command]
pub async fn backend_get_profile(app_handle: AppHandle) -> ApiResponse<serde_json::Value> {
    let client = HttpClient::new(app_handle.clone());
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get("/user/profile").await;
    let cached_username = client.get_auth_session().map(|session| session.username);

    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                if let (Some(username), Some(profile)) = (cached_username.as_deref(), api_res.data.as_ref()) {
                    persist_cached_user_profile(&app_handle, username, profile);
                }
                return ApiResponse::success(api_res.data, Some(api_res.message));
            }

            if cached_username.is_some() && (api_res.code == 401 || api_res.code == 403) {
                clear_cached_user_profile(&app_handle);
            }

            ApiResponse::failed_with_details(
                None,
                Some(format_backend_message(&api_res.message, api_res.details.as_ref())),
                api_res.details,
            )
        }
        Err(error) => ApiResponse::error(Some(app_error_message(error))),
    }
}

#[command]
pub async fn backend_redeem_sponsor_code(
    app_handle: AppHandle,
    req: SponsorRedeemReq,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<String>> = client.post("/sponsor/redeem", &req).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_update_username(
    app_handle: AppHandle,
    req: UpdateUsernameReq,
) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/user/username", &req).await;
    trans_api_res_token(client, res)
}
