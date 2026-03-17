use crate::app::app_error::{AppError, AppResult};
use crate::constant::sys_conf_path::{APP_STORE, JWT_TOKEN_KEY};
use crate::infrastructure::logging::log_trait::Log;
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

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

    pub fn get_jwt_token(&self) -> Option<String> {
        if let Ok(store) = self.app_handle.store(APP_STORE) {
            if let Some(token_val) = store.get(JWT_TOKEN_KEY) {
                if let Some(token) = token_val.as_str() {
                    return Some(token.to_string());
                }
            }
        }
        None
    }

    pub fn set_jwt_token(&self, token: &str) -> AppResult<()> {
        if let Ok(store) = self.app_handle.store(APP_STORE) {
            store.set(JWT_TOKEN_KEY, serde_json::json!(token));
            let _ = store.save();
            Ok(())
        } else {
            Err(AppError::HttpErr {
                detail: "无法获取 Store 实例".to_string(),
                e: "".to_string(),
            })
        }
    }

    pub fn clear_jwt_token(&self) -> AppResult<()> {
        if let Ok(store) = self.app_handle.store(APP_STORE) {
            store.delete(JWT_TOKEN_KEY);
            let _ = store.save();
            Ok(())
        } else {
            Err(AppError::HttpErr {
                detail: "无法获取 Store 实例".to_string(),
                e: "".to_string(),
            })
        }
    }

    async fn execute<T: DeserializeOwned>(&self, mut request: RequestBuilder) -> AppResult<T> {
        if let Some(token) = self.get_jwt_token() {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(|e| AppError::HttpErr {
            detail: "请求发送失败".to_string(),
            e: e.to_string(),
        })?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if !status.is_success() {
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
        let mut request = self.client.get(&url);
        if let Some(token) = self.get_jwt_token() {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(|e| AppError::HttpErr {
            detail: "请求下载文件失败".to_string(),
            e: e.to_string(),
        })?;

        if !response.status().is_success() {
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
        let mut request = self.client.post(&url);
        
        if let Some(token) = self.get_jwt_token() {
            request = request.bearer_auth(token);
        }

        let file_contents = std::fs::read(file_path).map_err(|e| AppError::HttpErr {
            detail: format!("读取本地文件 {} 失败", file_path.display()),
            e: e.to_string(),
        })?;

        let part = reqwest::multipart::Part::bytes(file_contents)
            .file_name(file_name.to_string());
        
        let form = reqwest::multipart::Form::new().part(file_part_name.to_string(), part);
        request = request.multipart(form);

        self.execute(request).await
    }
}
