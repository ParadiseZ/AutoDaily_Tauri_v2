use crate::app::app_error::{AppError, AppResult};
use crate::api::backend_dto::{AuthRes, BackendApiRes, RefreshTokenReq};
use crate::constant::sys_conf_path::{APP_STORE, AUTH_SESSION_KEY};
use crate::infrastructure::logging::log_trait::Log;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use std::sync::OnceLock;
use machineid_rs::{IdBuilder, Encryption, HWIDComponent};

pub fn get_machine_code() -> String {
    static MACHINE_CODE: OnceLock<String> = OnceLock::new();
    MACHINE_CODE.get_or_init(|| {
        IdBuilder::new(Encryption::MD5)
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::MacAddress)
            .build("auto_daily")
            .unwrap_or_else(|_| "unknown_machine_val".to_string())
    }).clone()
}

// 可以在正式环境使用环境变量或配置
const BACKEND_BASE_URL: &str = "http://localhost:8080/api";

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    app_handle: AppHandle,
}

impl HttpClient {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            client: Client::new(),
            app_handle,
        }
    }

    pub fn get_auth_session(&self) -> Option<AuthRes> {
        let store = self.app_handle.store(APP_STORE).ok()?;

        if let Some(session_val) = store.get(AUTH_SESSION_KEY) {
            if let Ok(session) = serde_json::from_value::<AuthRes>(session_val.clone()) {
                return Some(session);
            }
        }

        None
    }

    pub fn get_jwt_token(&self) -> Option<String> {
        self.get_auth_session().map(|session| session.access_token)
    }

    pub fn set_auth_session(&self, session: &AuthRes) -> AppResult<()> {
        if let Ok(store) = self.app_handle.store(APP_STORE) {
            store.set(
                AUTH_SESSION_KEY,
                serde_json::to_value(session).map_err(|e| AppError::HttpErr {
                    detail: "序列化认证会话失败".to_string(),
                    e: e.to_string(),
                })?,
            );
            let _ = store.save();
            Ok(())
        } else {
            Err(AppError::HttpErr {
                detail: "无法获取 Store 实例".to_string(),
                e: "".to_string(),
            })
        }
    }

    pub fn clear_auth_session(&self) -> AppResult<()> {
        if let Ok(store) = self.app_handle.store(APP_STORE) {
            store.delete(AUTH_SESSION_KEY);
            let _ = store.save();
            Ok(())
        } else {
            Err(AppError::HttpErr {
                detail: "无法获取 Store 实例".to_string(),
                e: "".to_string(),
            })
        }
    }

    fn apply_auth_headers(&self, mut request: RequestBuilder) -> RequestBuilder {
        if let Some(token) = self.get_jwt_token() {
            request = request.bearer_auth(token);
        }

        let machine_id = get_machine_code();
        request.header("Machine-Code", machine_id)
    }

    async fn send_request(&self, request: RequestBuilder) -> AppResult<Response> {
        self.apply_auth_headers(request)
            .send()
            .await
            .map_err(|e| AppError::HttpErr {
                detail: "请求发送失败".to_string(),
                e: e.to_string(),
            })
    }

    async fn refresh_auth_session(&self) -> AppResult<bool> {
        let Some(session) = self.get_auth_session() else {
            return Ok(false);
        };

        if session.refresh_token.trim().is_empty() {
            let _ = self.clear_auth_session();
            return Ok(false);
        }

        let url = format!("{}/auth/refresh", BACKEND_BASE_URL);
        let request = self
            .client
            .post(&url)
            .header("Machine-Code", get_machine_code())
            .json(&RefreshTokenReq {
                refresh_token: session.refresh_token,
            });

        let response = request.send().await.map_err(|e| AppError::HttpErr {
            detail: "刷新登录态失败".to_string(),
            e: e.to_string(),
        })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            let _ = self.clear_auth_session();
            Log::error(&format!("刷新登录态失败, status: {}, text: {}", status, text));
            return Ok(false);
        }

        let api_res = serde_json::from_str::<BackendApiRes<AuthRes>>(&text).map_err(|e| AppError::HttpErr {
            detail: "解析刷新登录态响应失败".to_string(),
            e: e.to_string(),
        })?;

        if api_res.code != 200 {
            let _ = self.clear_auth_session();
            Log::error(&format!("刷新登录态失败, code: {}, message: {}", api_res.code, api_res.message));
            return Ok(false);
        }

        if let Some(next_session) = api_res.data {
            self.set_auth_session(&next_session)?;
            return Ok(true);
        }

        let _ = self.clear_auth_session();
        Ok(false)
    }

    async fn send_with_retry(&self, request: RequestBuilder) -> AppResult<Response> {
        let retry_request = request.try_clone();
        let response = self.send_request(request).await?;

        if response.status() != StatusCode::UNAUTHORIZED {
            return Ok(response);
        }

        let can_retry = retry_request.is_some();
        if !can_retry || !self.refresh_auth_session().await? {
            let _ = self.clear_auth_session();
            return Ok(response);
        }

        self.send_request(retry_request.expect("retry request already checked"))
            .await
    }

    async fn execute<T: DeserializeOwned>(&self, request: RequestBuilder) -> AppResult<T> {
        let response = self.send_with_retry(request).await?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            if status == StatusCode::UNAUTHORIZED {
                let _ = self.clear_auth_session();
            }
            Log::error(&format!("HTTP请求失败, status: {}, text: {}", status, text));
            return Err(AppError::HttpErr {
                detail: format!("接口返回错误状态码: {}", status),
                e: text,
            });
        }

        serde_json::from_str(&text).map_err(|e| AppError::HttpErr {
            detail: "解析响应 JSON 失败".to_string(),
            e: e.to_string(),
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> AppResult<T> {
        let url = format!("{}{}", BACKEND_BASE_URL, endpoint);
        let request = self.client.get(&url);
        self.execute(request).await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, endpoint: &str, body: &B) -> AppResult<T> {
        let url = format!("{}{}", BACKEND_BASE_URL, endpoint);
        let request = self.client.post(&url).json(body);
        self.execute(request).await
    }

    pub async fn download_file(&self, endpoint: &str, target_path: &std::path::Path) -> AppResult<()> {
        use std::io::Write;
        let url = format!("{}{}", BACKEND_BASE_URL, endpoint);
        let request = self.client.get(&url);
        let response = self.send_with_retry(request).await.map_err(|e| AppError::HttpErr {
            detail: "请求下载文件失败".to_string(),
            e: e.to_string(),
        })?;

        if !response.status().is_success() {
            if response.status() == StatusCode::UNAUTHORIZED {
                let _ = self.clear_auth_session();
            }
            return Err(AppError::HttpErr {
                detail: format!("文件下载返回了失败状态码: {}", response.status()),
                e: "".to_string(),
            });
        }

        let bytes = response.bytes().await.map_err(|e| AppError::HttpErr {
            detail: "读取下载文件流失败".to_string(),
            e: e.to_string(),
        })?;

        let mut file = std::fs::File::create(target_path).map_err(|e| AppError::HttpErr {
            detail: format!("创建本地文件 {} 失败", target_path.display()),
            e: e.to_string(),
        })?;

        file.write_all(&bytes).map_err(|e| AppError::HttpErr {
            detail: format!("写入本地文件 {} 失败", target_path.display()),
            e: e.to_string(),
        })?;

        Ok(())
    }

    pub async fn upload_file<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        file_path: &std::path::Path,
        file_part_name: &str,
        file_name: &str,
    ) -> AppResult<T> {
        let url = format!("{}{}", BACKEND_BASE_URL, endpoint);
        let file_contents = std::fs::read(file_path).map_err(|e| AppError::HttpErr {
            detail: format!("读取本地文件 {} 失败", file_path.display()),
            e: e.to_string(),
        })?;

        let part = reqwest::multipart::Part::bytes(file_contents)
            .file_name(file_name.to_string());
        
        let form = reqwest::multipart::Form::new().part(file_part_name.to_string(), part);
        let request = self.client.post(&url).multipart(form);

        self.execute(request).await
    }
}
