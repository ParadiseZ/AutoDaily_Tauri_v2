use crate::api::api_response::ApiResponse;
use crate::api::backend_cmd::{trans_api_res, app_error_message};
use crate::api::backend_dto::{BackendApiRes, ScriptModelFileDto};
use crate::app::app_error::AppResult;
use crate::constant::sys_conf_path::{APP_STORE, SCRIPTS_CONFIG_KEY};
use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::domain::scripts::script_info::{RuntimeType, ScriptTable};
use crate::infrastructure::http_client::HttpClient;
use crate::infrastructure::store_local::config_store::get_or_init_config;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle};
use tauri_plugin_store::StoreExt;
use vision_core::infrastructure::vision::base_model::{BaseModel, ModelSource};
use vision_core::infrastructure::vision::det::DetectorType;
use vision_core::infrastructure::vision::rec::RecognizerType;

#[derive(Debug, Clone)]
pub(super) struct LocalModelUpload {
    pub(super) type_name: &'static str,
    pub(super) file_name: &'static str,
    pub(super) local_path: PathBuf,
    pub(super) size_bytes: u64,
    pub(super) sha256: String,
}

#[derive(Clone, Copy)]
pub(super) struct ModelTypeSpec {
    pub(super) type_name: &'static str,
    pub(super) file_name: &'static str,
}

const IMG_DET_MODEL: ModelTypeSpec = ModelTypeSpec {
    type_name: "img_det_model",
    file_name: "img_det_model.onnx",
};
const TXT_DET_MODEL: ModelTypeSpec = ModelTypeSpec {
    type_name: "txt_det_model",
    file_name: "txt_det_model.onnx",
};
const TXT_REC_MODEL: ModelTypeSpec = ModelTypeSpec {
    type_name: "txt_rec_model",
    file_name: "txt_rec_model.onnx",
};

pub(super) fn runtime_type_param(runtime_type: &RuntimeType) -> Result<String, String> {
    serde_json::to_value(runtime_type)
        .map_err(|error| format!("序列化 runtime_type 失败: {}", error))?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "runtime_type 序列化结果不是字符串".to_string())
}

pub(crate) fn local_scripts_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .store(APP_STORE)
        .map(|store| get_or_init_config::<ScriptsConfig>(store, SCRIPTS_CONFIG_KEY).dir)
        .unwrap_or_else(|_| ScriptsConfig::default().dir)
}

pub(super) fn build_model_file_payload(
    script_id: &str,
    version_num: u64,
    runtime_type: &str,
    uploads: &[LocalModelUpload],
) -> Vec<ScriptModelFileDto> {
    uploads
        .iter()
        .map(|item| ScriptModelFileDto {
            script_id: Some(script_id.to_string()),
            version_num: Some(version_num),
            runtime_type: runtime_type.to_string(),
            r#type: item.type_name.to_string(),
            file_name: item.file_name.to_string(),
            download_path: format!(
                "/api/scripts/download/model/{}/{}/{}?runtime_type={}",
                script_id, version_num, item.type_name, runtime_type
            ),
            size_bytes: Some(item.size_bytes),
            hash_algorithm: Some("SHA-256".to_string()),
            hash_value: Some(item.sha256.clone()),
            etag: None,
        })
        .collect()
}

pub(super) fn collect_model_uploads(
    script: &ScriptTable,
    scripts_root: &Path,
) -> Result<Vec<LocalModelUpload>, String> {
    let mut uploads = Vec::new();

    if let Some(spec) = detector_upload(script.data.img_det_model.as_ref(), scripts_root, IMG_DET_MODEL)? {
        uploads.push(spec);
    }
    if let Some(spec) = detector_upload(script.data.txt_det_model.as_ref(), scripts_root, TXT_DET_MODEL)? {
        uploads.push(spec);
    }
    if let Some(spec) = recognizer_upload(script.data.txt_rec_model.as_ref(), scripts_root, TXT_REC_MODEL)? {
        uploads.push(spec);
    }

    Ok(uploads)
}

pub(super) fn rewrite_script_model_paths_for_published(script: &mut ScriptTable, script_id: &str) {
    rewrite_detector_model_path(&mut script.data.img_det_model, script_id, IMG_DET_MODEL.file_name);
    rewrite_detector_model_path(&mut script.data.txt_det_model, script_id, TXT_DET_MODEL.file_name);
    rewrite_recognizer_model_path(&mut script.data.txt_rec_model, script_id, TXT_REC_MODEL.file_name);
}

pub(super) fn normalize_download_endpoint(download_path: &str) -> Result<String, String> {
    let trimmed = download_path.trim();
    if trimmed.is_empty() {
        return Err("模型下载地址为空".to_string());
    }
    if let Some(stripped) = trimmed.strip_prefix("/api") {
        return Ok(stripped.to_string());
    }
    if trimmed.starts_with('/') {
        return Ok(trimmed.to_string());
    }
    Ok(format!("/{}", trimmed))
}

pub(super) fn normalize_model_type(value: &str) -> Result<ModelTypeSpec, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "img_det_model" | "img-det-model" | "imgdetmodel" | "det" => Ok(IMG_DET_MODEL),
        "txt_det_model" | "txt-det-model" | "txtdetmodel" | "txt-det" => Ok(TXT_DET_MODEL),
        "txt_rec_model" | "txt-rec-model" | "txtrecmodel" | "rec" => Ok(TXT_REC_MODEL),
        other => Err(format!("不支持的模型类型: {}", other)),
    }
}

fn detector_upload(
    model: Option<&DetectorType>,
    scripts_root: &Path,
    target: ModelTypeSpec,
) -> Result<Option<LocalModelUpload>, String> {
    let Some(model) = model else {
        return Ok(None);
    };
    build_local_model_upload(detector_base_model(model), scripts_root, target)
}

fn recognizer_upload(
    model: Option<&RecognizerType>,
    scripts_root: &Path,
    target: ModelTypeSpec,
) -> Result<Option<LocalModelUpload>, String> {
    let Some(model) = model else {
        return Ok(None);
    };
    build_local_model_upload(recognizer_base_model(model), scripts_root, target)
}

fn build_local_model_upload(
    base_model: &BaseModel,
    scripts_root: &Path,
    target: ModelTypeSpec,
) -> Result<Option<LocalModelUpload>, String> {
    if base_model.model_source != ModelSource::Custom {
        return Ok(None);
    }
    if base_model.model_path.as_os_str().is_empty() {
        return Err(format!("模型 {} 缺少本地路径", target.file_name));
    }
    let local_path = resolve_local_model_path(base_model.model_path.as_path(), scripts_root);
    let metadata = std::fs::metadata(&local_path)
        .map_err(|error| format!("读取模型文件 {} 失败: {}", local_path.display(), error))?;
    if !metadata.is_file() {
        return Err(format!("模型路径不是文件: {}", local_path.display()));
    }
    Ok(Some(LocalModelUpload {
        type_name: target.type_name,
        file_name: target.file_name,
        local_path: local_path.clone(),
        size_bytes: metadata.len(),
        sha256: sha256_file_hex(&local_path)?,
    }))
}

fn resolve_local_model_path(path: &Path, scripts_root: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        scripts_root.join(path)
    }
}

fn sha256_file_hex(path: &Path) -> Result<String, String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)
        .map_err(|error| format!("打开模型文件 {} 失败: {}", path.display(), error))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("读取模型文件 {} 失败: {}", path.display(), error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn rewrite_detector_model_path(model: &mut Option<DetectorType>, script_id: &str, file_name: &str) {
    let Some(model) = model else {
        return;
    };
    let base_model = detector_base_model_mut(model);
    if base_model.model_source == ModelSource::Custom {
        base_model.model_path = PathBuf::from(script_id).join(file_name);
    }
}

fn rewrite_recognizer_model_path(
    model: &mut Option<RecognizerType>,
    script_id: &str,
    file_name: &str,
) {
    let Some(model) = model else {
        return;
    };
    let base_model = recognizer_base_model_mut(model);
    if base_model.model_source == ModelSource::Custom {
        base_model.model_path = PathBuf::from(script_id).join(file_name);
    }
}

fn detector_base_model(model: &DetectorType) -> &BaseModel {
    match model {
        DetectorType::Yolo11(det) | DetectorType::Yolo26(det) => &det.base_model,
        DetectorType::PaddleDbNet(det) => &det.base_model,
    }
}

fn detector_base_model_mut(model: &mut DetectorType) -> &mut BaseModel {
    match model {
        DetectorType::Yolo11(det) | DetectorType::Yolo26(det) => &mut det.base_model,
        DetectorType::PaddleDbNet(det) => &mut det.base_model,
    }
}

fn recognizer_base_model(model: &RecognizerType) -> &BaseModel {
    match model {
        RecognizerType::PaddleCrnn(rec) => &rec.base_model,
    }
}

fn recognizer_base_model_mut(model: &mut RecognizerType) -> &mut BaseModel {
    match model {
        RecognizerType::PaddleCrnn(rec) => &mut rec.base_model,
    }
}

#[command]
pub async fn backend_upload_model(
    app_handle: AppHandle,
    script_id: String,
    runtime_type: String,
    model_type: String,
    local_file_path: String,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let normalized_type = match normalize_model_type(model_type.as_str()) {
        Ok(value) => value,
        Err(error) => return ApiResponse::error(Some(error)),
    };
    let url = format!(
        "/scripts/upload/model/{}/{}?runtime_type={}",
        script_id, normalized_type.type_name, runtime_type
    );

    let path = std::path::Path::new(&local_file_path);
    if !path.exists() {
        return ApiResponse::error(Some(format!("File {} does not exist locally", local_file_path)));
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(normalized_type.file_name);

    let res: AppResult<BackendApiRes<String>> = client
        .upload_file(&url, path, "file", file_name)
        .await;

    trans_api_res(res)
}

#[command]
pub async fn backend_download_model(
    app_handle: AppHandle,
    script_id: String,
    version_num: u64,
    runtime_type: String,
    model_type: String,
    save_dir: String,
    expected_sha256: Option<String>,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let normalized_type = match normalize_model_type(model_type.as_str()) {
        Ok(value) => value,
        Err(error) => return ApiResponse::error(Some(error)),
    };
    let url = format!(
        "/scripts/download/model/{}/{}/{}?runtime_type={}",
        script_id, version_num, normalized_type.type_name, runtime_type
    );

    let dir_path = std::path::Path::new(&save_dir);
    if !dir_path.exists() {
        if let Err(error) = std::fs::create_dir_all(dir_path) {
            return ApiResponse::error(Some(format!("Failed to create save directory: {}", error)));
        }
    }

    let target_path = dir_path.join(normalized_type.file_name);

    match client
        .download_file_with_resume(&url, &target_path, expected_sha256.as_deref())
        .await
    {
        Ok(_) => ApiResponse::success(
            Some(target_path.to_string_lossy().to_string()),
            Some("Model downloaded successfully".to_string()),
        ),
        Err(error) => ApiResponse::error(Some(app_error_message(error.into()))),
    }
}
