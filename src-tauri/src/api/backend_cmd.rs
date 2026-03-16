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
    trans_api_res(res)
}

#[command]
pub async fn backend_register(
    app_handle: AppHandle,
    req: RegisterReq,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<String>> = client.post("/auth/register", &req).await;
    trans_api_res(res)
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

#[command]
pub async fn backend_get_profile(app_handle: AppHandle) -> ApiResponse<serde_json::Value> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get("/user/profile").await;
    trans_api_res(res)
}

#[command]
pub async fn backend_search_scripts(
    app_handle: AppHandle,
    req: ScriptSearchReq,
) -> ApiResponse<PageRes<serde_json::Value>> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<PageRes<serde_json::Value>>> = client.post("/scripts/search", &req).await;
    trans_api_res( res)
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
pub async fn backend_check_update(app_handle: AppHandle) -> ApiResponse<TauriUpdateRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<TauriUpdateRes> = client.get("/update/check").await;
    match res {
        Ok(update_res) => ApiResponse::success(Some(update_res), Some("Found update".to_string())),
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

#[command]
pub async fn backend_download_script(app_handle: AppHandle, script_id: String) -> ApiResponse<serde_json::Value> {
    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/download/{}", script_id);
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get(&url).await;
    trans_api_res( res)
}

fn trans_api_res<T,R>(api_res: AppResult<BackendApiRes<T>>)   -> ApiResponse<R> {
    match api_res {
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