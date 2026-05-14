use crate::api::backend_dto::{AuthRes, BackendApiRes, RefreshTokenReq};
use crate::app::app_error::{AppError, AppResult};
use crate::constant::sys_conf_path::{APP_STORE, AUTH_SESSION_KEY};
use crate::infrastructure::logging::log_trait::Log;
use futures_util::StreamExt;
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio_util::io::ReaderStream;

pub fn get_machine_code() -> String {
    static MACHINE_CODE: OnceLock<String> = OnceLock::new();
    MACHINE_CODE
        .get_or_init(|| {
            IdBuilder::new(Encryption::MD5)
                .add_component(HWIDComponent::SystemID)
                .add_component(HWIDComponent::MacAddress)
                .build("auto_daily")
                .unwrap_or_else(|_| "unknown_machine_val".to_string())
        })
        .clone()
}

const DEFAULT_BACKEND_SERVER_URL: &str = "http://localhost:8080";
const DEFAULT_REMOTE_SERVER_CONFIG_URL: &str =
    "https://raw.githubusercontent.com/ParadiseZ/AutoDailyTauriRelease/main/latest.json";
const BACKEND_BASE_URL_CACHE_KEY: &str = "backend_base_url_cache";
static BACKEND_BASE_URL: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ServerConfigPlugin {
    #[serde(alias = "remote_config_url")]
    remote_config_url: Option<String>,
    #[serde(alias = "default_server_url")]
    default_server_url: Option<String>,
    #[serde(alias = "default_api_base_url")]
    default_api_base_url: Option<String>,
    #[serde(alias = "server_url")]
    server_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RemoteServerConfig {
    #[serde(alias = "server_url")]
    server_url: Option<String>,
    #[serde(alias = "api_base_url")]
    api_base_url: Option<String>,
}

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    app_handle: AppHandle,
}

#[derive(Debug, Clone)]
pub struct FileTransferProgress {
    pub transferred_bytes: u64,
    pub total_bytes: Option<u64>,
}

impl HttpClient {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            client: Client::new(),
            app_handle,
        }
    }

    async fn backend_base_url(&self) -> String {
        BACKEND_BASE_URL
            .get_or_init(|| async { self.resolve_backend_base_url().await })
            .await
            .clone()
    }

    async fn resolve_backend_base_url(&self) -> String {
        let plugin_config = self.server_config_plugin();

        if let Some(remote_url) = self.fetch_remote_backend_base_url(&plugin_config).await {
            self.set_cached_backend_base_url(&remote_url);
            return remote_url;
        }

        if let Some(cached_url) = self.cached_backend_base_url() {
            return cached_url;
        }

        self.default_backend_base_url(&plugin_config)
    }

    fn server_config_plugin(&self) -> ServerConfigPlugin {
        self.app_handle
            .config()
            .plugins
            .0
            .get("server_config")
            .and_then(|value| serde_json::from_value::<ServerConfigPlugin>(value.clone()).ok())
            .unwrap_or_default()
    }

    async fn fetch_remote_backend_base_url(
        &self,
        plugin_config: &ServerConfigPlugin,
    ) -> Option<String> {
        let config_url = plugin_config
            .remote_config_url
            .as_deref()
            .unwrap_or(DEFAULT_REMOTE_SERVER_CONFIG_URL)
            .trim();

        if config_url.is_empty() {
            return None;
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        let response = client
            .get(config_url)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let config = response.json::<RemoteServerConfig>().await.ok()?;

        config
            .api_base_url
            .as_deref()
            .and_then(normalize_api_base_url)
            .or_else(|| {
                config
                    .server_url
                    .as_deref()
                    .and_then(normalize_server_url_to_api_base)
            })
    }

    fn cached_backend_base_url(&self) -> Option<String> {
        let store = self.app_handle.store(APP_STORE).ok()?;
        store
            .get(BACKEND_BASE_URL_CACHE_KEY)
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .as_deref()
            .and_then(normalize_api_base_url)
    }

    fn set_cached_backend_base_url(&self, base_url: &str) {
        if let Ok(store) = self.app_handle.store(APP_STORE) {
            store.set(
                BACKEND_BASE_URL_CACHE_KEY,
                serde_json::Value::String(base_url.to_string()),
            );
            let _ = store.save();
        }
    }

    fn default_backend_base_url(&self, plugin_config: &ServerConfigPlugin) -> String {
        plugin_config
            .default_api_base_url
            .as_deref()
            .and_then(normalize_api_base_url)
            .or_else(|| {
                plugin_config
                    .default_server_url
                    .as_deref()
                    .or(plugin_config.server_url.as_deref())
                    .and_then(normalize_server_url_to_api_base)
            })
            .unwrap_or_else(|| {
                normalize_server_url_to_api_base(DEFAULT_BACKEND_SERVER_URL)
                    .unwrap_or_else(|| "https://api.example.com/api".to_string())
            })
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

        let base_url = self.backend_base_url().await;
        let url = format!("{}/auth/refresh", base_url);
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
            Log::error(&format!(
                "刷新登录态失败, status: {}, text: {}",
                status, text
            ));
            return Ok(false);
        }

        let api_res = serde_json::from_str::<BackendApiRes<AuthRes>>(&text).map_err(|e| {
            AppError::HttpErr {
                detail: "解析刷新登录态响应失败".to_string(),
                e: e.to_string(),
            }
        })?;

        if api_res.code != 200 {
            let _ = self.clear_auth_session();
            Log::error(&format!(
                "刷新登录态失败, code: {}, message: {}",
                api_res.code, api_res.message
            ));
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
        let base_url = self.backend_base_url().await;
        let url = format!("{}{}", base_url, endpoint);
        let request = self.client.get(&url);
        self.execute(request).await
    }

    pub async fn post<T: DeserializeOwned, B: runtime_common::core::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> AppResult<T> {
        let base_url = self.backend_base_url().await;
        let url = format!("{}{}", base_url, endpoint);
        let request = self.client.post(&url).json(body);
        self.execute(request).await
    }

    pub async fn post_api_res<T: DeserializeOwned, B: runtime_common::core::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> AppResult<BackendApiRes<T>> {
        let base_url = self.backend_base_url().await;
        let url = format!("{}{}", base_url, endpoint);
        let request = self.client.post(&url).json(body);
        let response = self.send_with_retry(request).await?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        serde_json::from_str(&text).map_err(|e| AppError::HttpErr {
            detail: format!("解析接口响应失败: {}", status),
            e: if text.is_empty() {
                e.to_string()
            } else {
                format!("body: {}, parse_error: {}", text, e)
            },
        })
    }

    pub async fn download_file(
        &self,
        endpoint: &str,
        target_path: &std::path::Path,
    ) -> AppResult<()> {
        use std::io::Write;
        let base_url = self.backend_base_url().await;
        let url = format!("{}{}", base_url, endpoint);
        let request = self.client.get(&url);
        let response = self
            .send_with_retry(request)
            .await
            .map_err(|e| AppError::HttpErr {
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

    pub async fn download_file_with_resume(
        &self,
        endpoint: &str,
        target_path: &std::path::Path,
        expected_sha256: Option<&str>,
    ) -> AppResult<()> {
        self.download_file_with_resume_progress(endpoint, target_path, expected_sha256, |_| {})
            .await
    }

    pub async fn download_file_with_resume_progress<F>(
        &self,
        endpoint: &str,
        target_path: &std::path::Path,
        expected_sha256: Option<&str>,
        mut on_progress: F,
    ) -> AppResult<()>
    where
        F: FnMut(FileTransferProgress),
    {
        use reqwest::header::{ETAG, IF_RANGE, RANGE};
        use std::fs::{self, OpenOptions};
        use std::io::Write;

        if let Some(expected_sha256) = expected_sha256 {
            if target_path.exists() && sha256_hex(target_path)? == expected_sha256 {
                return Ok(());
            }
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::HttpErr {
                detail: format!("创建目录 {} 失败", parent.display()),
                e: e.to_string(),
            })?;
        }

        let part_path = part_path_for(target_path);
        let etag_path = etag_path_for(&part_path);
        let mut resume_from = if part_path.exists() {
            part_path.metadata().map(|meta| meta.len()).unwrap_or(0)
        } else {
            0
        };

        let resume_etag = if resume_from > 0 {
            match fs::read_to_string(&etag_path) {
                Ok(value) if !value.trim().is_empty() => Some(value),
                _ => {
                    let _ = fs::remove_file(&part_path);
                    let _ = fs::remove_file(&etag_path);
                    resume_from = 0;
                    None
                }
            }
        } else {
            None
        };

        let base_url = self.backend_base_url().await;
        let url = format!("{}{}", base_url, endpoint);
        let mut request = self.client.get(&url);
        if resume_from > 0 {
            request = request.header(RANGE, format!("bytes={resume_from}-"));
            if let Some(etag) = resume_etag.as_deref() {
                request = request.header(IF_RANGE, etag);
            }
        }

        let response = self
            .send_with_retry(request)
            .await
            .map_err(|e| AppError::HttpErr {
                detail: "请求下载文件失败".to_string(),
                e: e.to_string(),
            })?;

        let status = response.status();
        if !status.is_success() {
            if status == StatusCode::UNAUTHORIZED {
                let _ = self.clear_auth_session();
            }
            return Err(AppError::HttpErr {
                detail: format!("文件下载返回了失败状态码: {}", status),
                e: "".to_string(),
            });
        }

        let response_etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let append = status == StatusCode::PARTIAL_CONTENT && resume_from > 0;

        if !append {
            let _ = fs::remove_file(&part_path);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(&part_path)
            .map_err(|e| AppError::HttpErr {
                detail: format!("创建临时文件 {} 失败", part_path.display()),
                e: e.to_string(),
            })?;

        let total_bytes = resolve_response_total_bytes(response.headers(), append, resume_from);
        let mut transferred_bytes = if append { resume_from } else { 0 };
        on_progress(FileTransferProgress {
            transferred_bytes,
            total_bytes,
        });

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::HttpErr {
                detail: "读取下载文件流失败".to_string(),
                e: e.to_string(),
            })?;
            file.write_all(&chunk).map_err(|e| AppError::HttpErr {
                detail: format!("写入临时文件 {} 失败", part_path.display()),
                e: e.to_string(),
            })?;
            transferred_bytes += chunk.len() as u64;
            on_progress(FileTransferProgress {
                transferred_bytes,
                total_bytes,
            });
        }

        if let Some(etag) = response_etag.as_deref() {
            fs::write(&etag_path, etag).map_err(|e| AppError::HttpErr {
                detail: format!("写入 ETag 文件 {} 失败", etag_path.display()),
                e: e.to_string(),
            })?;
        }

        if let Some(expected_sha256) = expected_sha256 {
            let actual_sha256 = sha256_hex(&part_path)?;
            if actual_sha256 != expected_sha256 {
                let _ = fs::remove_file(&part_path);
                let _ = fs::remove_file(&etag_path);
                return Err(AppError::HttpErr {
                    detail: "模型文件 SHA-256 校验失败".to_string(),
                    e: format!(
                        "expected {}, got {} for {}",
                        expected_sha256,
                        actual_sha256,
                        part_path.display()
                    ),
                });
            }
        }

        if target_path.exists() {
            fs::remove_file(target_path).map_err(|e| AppError::HttpErr {
                detail: format!("删除旧文件 {} 失败", target_path.display()),
                e: e.to_string(),
            })?;
        }
        fs::rename(&part_path, target_path).map_err(|e| AppError::HttpErr {
            detail: format!(
                "将临时文件 {} 改名为 {} 失败",
                part_path.display(),
                target_path.display()
            ),
            e: e.to_string(),
        })?;
        let _ = fs::remove_file(&etag_path);

        Ok(())
    }

    pub async fn upload_file<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        file_path: &std::path::Path,
        file_part_name: &str,
        file_name: &str,
    ) -> AppResult<T> {
        self.upload_file_with_progress(endpoint, file_path, file_part_name, file_name, |_| {})
            .await
    }

    pub async fn upload_file_with_progress<T: DeserializeOwned, F>(
        &self,
        endpoint: &str,
        file_path: &std::path::Path,
        file_part_name: &str,
        file_name: &str,
        mut on_progress: F,
    ) -> AppResult<T>
    where
        F: FnMut(FileTransferProgress) + Send + 'static,
    {
        let base_url = self.backend_base_url().await;
        let url = format!("{}{}", base_url, endpoint);
        let file = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| AppError::HttpErr {
                detail: format!("读取本地文件 {} 失败", file_path.display()),
                e: e.to_string(),
            })?;
        let total_bytes = file
            .metadata()
            .await
            .map_err(|e| AppError::HttpErr {
                detail: format!("读取本地文件 {} 元信息失败", file_path.display()),
                e: e.to_string(),
            })?
            .len();

        on_progress(FileTransferProgress {
            transferred_bytes: 0,
            total_bytes: Some(total_bytes),
        });

        let mut transferred_bytes = 0_u64;
        let stream = ReaderStream::new(file).map(move |item| {
            item.map(|bytes| {
                transferred_bytes += bytes.len() as u64;
                on_progress(FileTransferProgress {
                    transferred_bytes,
                    total_bytes: Some(total_bytes),
                });
                bytes
            })
        });

        let body = reqwest::Body::wrap_stream(stream);
        let part = reqwest::multipart::Part::stream_with_length(body, total_bytes)
            .file_name(file_name.to_string());

        let form = reqwest::multipart::Form::new().part(file_part_name.to_string(), part);
        let request = self.client.post(&url).multipart(form);

        self.execute(request).await
    }
}

fn resolve_response_total_bytes(
    headers: &reqwest::header::HeaderMap,
    append: bool,
    resume_from: u64,
) -> Option<u64> {
    use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE};

    if let Some(content_range) = headers.get(CONTENT_RANGE).and_then(|value| value.to_str().ok()) {
        if let Some((_, total_part)) = content_range.rsplit_once('/') {
            if let Ok(total) = total_part.parse::<u64>() {
                return Some(total);
            }
        }
    }

    headers
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|content_length| {
            if append {
                resume_from + content_length
            } else {
                content_length
            }
        })
}

fn part_path_for(target_path: &std::path::Path) -> std::path::PathBuf {
    let mut file_name = target_path
        .file_name()
        .map(|value| value.to_os_string())
        .unwrap_or_else(|| "download".into());
    file_name.push(".part");
    target_path.with_file_name(file_name)
}

fn etag_path_for(part_path: &std::path::Path) -> std::path::PathBuf {
    let mut file_name = part_path
        .file_name()
        .map(|value| value.to_os_string())
        .unwrap_or_else(|| "download.part".into());
    file_name.push(".etag");
    part_path.with_file_name(file_name)
}

fn sha256_hex(path: &std::path::Path) -> AppResult<String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| AppError::HttpErr {
        detail: format!("打开文件 {} 失败", path.display()),
        e: e.to_string(),
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|e| AppError::HttpErr {
            detail: format!("读取文件 {} 失败", path.display()),
            e: e.to_string(),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn normalize_api_base_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if !is_valid_http_url(trimmed) {
        return None;
    }
    Some(trimmed.to_string())
}

fn normalize_server_url_to_api_base(raw: &str) -> Option<String> {
    let server_url = raw.trim().trim_end_matches('/');
    if !is_valid_http_url(server_url) {
        return None;
    }
    if server_url.ends_with("/api") {
        Some(server_url.to_string())
    } else {
        Some(format!("{}/api", server_url))
    }
}

fn is_valid_http_url(raw: &str) -> bool {
    reqwest::Url::parse(raw)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .unwrap_or(false)
}

