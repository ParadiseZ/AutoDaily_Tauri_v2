use super::{app_error_message, format_backend_message, trans_api_res};
use crate::api::api_response::ApiResponse;
use crate::api::backend_dto::*;
use crate::api::infrastructure::script_version_preflight::{
    build_download_preflight, extract_cloud_summary_version, find_replaceable_local_published_script,
    format_version_label, version_num_to_i64, ScriptVersionPreflight,
};
use crate::app::app_error::AppResult;
use crate::constant::sys_conf_path::{APP_STORE, SCRIPTS_CONFIG_KEY};
use crate::constant::table_name::SCRIPT_TABLE;
use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::domain::scripts::script_info::{RuntimeType, ScriptTable, ScriptType};
use crate::infrastructure::core::UserId;
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::http_client::HttpClient;
use crate::infrastructure::store_local::config_store::get_or_init_config;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle};
use tauri_plugin_store::StoreExt;
use vision_core::infrastructure::vision::base_model::{BaseModel, ModelSource};
use vision_core::infrastructure::vision::det::DetectorType;
use vision_core::infrastructure::vision::rec::RecognizerType;

#[derive(Debug)]
struct UploadAuthor {
    id: UserId,
    username: String,
}

#[derive(Debug, Clone)]
struct LocalModelUpload {
    type_name: &'static str,
    file_name: &'static str,
    local_path: PathBuf,
    size_bytes: u64,
    sha256: String,
}

#[command]
pub async fn backend_search_scripts(
    app_handle: AppHandle,
    mut req: ScriptSearchReq,
) -> ApiResponse<PageRes<serde_json::Value>> {
    let client = HttpClient::new(app_handle);
    req.client = Some(current_client_capability());
    let res: AppResult<BackendApiRes<PageRes<serde_json::Value>>> =
        client.post("/scripts/search", &req).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_get_script_change_logs(
    app_handle: AppHandle,
    script_id: String,
    from_version: Option<i64>,
) -> ApiResponse<Vec<serde_json::Value>> {
    let client = HttpClient::new(app_handle);
    let url = match from_version {
        Some(version) => format!("/scripts/{}/change-log?fromVersion={}", script_id, version),
        None => format!("/scripts/{}/change-log", script_id),
    };
    let res: AppResult<BackendApiRes<Vec<serde_json::Value>>> = client.get(&url).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_get_script_cloud_summary(
    app_handle: AppHandle,
    script_id: String,
) -> ApiResponse<serde_json::Value> {
    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/{}/summary", script_id);
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get(&url).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_preflight_download_script(
    script_id: String,
    ver_name: Option<String>,
    ver_num: Option<i64>,
) -> ApiResponse<ScriptVersionPreflight> {
    match find_replaceable_local_published_script(&script_id).await {
        Ok(existing_local_script) => ApiResponse::success(
            Some(build_download_preflight(
                existing_local_script.as_ref(),
                ver_name.as_deref(),
                ver_num,
            )),
            None,
        ),
        Err(error) => ApiResponse::error(Some(error)),
    }
}

#[command]
pub async fn backend_preflight_upload_script(
    app_handle: AppHandle,
    script_id: String,
) -> ApiResponse<ScriptVersionPreflight> {
    let pool = crate::infrastructure::db::get_pool();
    let local_script = match sqlx::query_as::<_, ScriptTable>("SELECT id, `data` FROM scripts WHERE id = ?")
        .bind(&script_id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(script)) => script,
        Ok(None) => return ApiResponse::error(Some("脚本不存在".to_string())),
        Err(error) => return ApiResponse::error(Some(format!("读取本地脚本失败: {}", error))),
    };

    if local_script.data.script_type != ScriptType::Dev {
        return ApiResponse::error(Some("只有本地脚本 (Dev) 才能被上传".to_string()));
    }

    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/{}/summary", script_id);
    let response: AppResult<BackendApiRes<serde_json::Value>> = client.get(&url).await;
    let local_ver_num = version_num_to_i64(Some(local_script.data.ver_num));
    let local_version_label =
        format_version_label(Some(local_script.data.ver_name.as_str()), local_ver_num);

    let api_res = match response {
        Ok(value) => value,
        Err(error) => return ApiResponse::error(Some(app_error_message(error))),
    };

    if api_res.code != 200 {
        return ApiResponse::failed_with_details(
            None,
            Some(format_backend_message(&api_res.message, api_res.details.as_ref())),
            api_res.details,
        );
    }

    let Some(summary) = api_res.data else {
        return ApiResponse::success(
            Some(ScriptVersionPreflight {
                status: "cloudMissing".to_string(),
                message: format!(
                    "云端还没有该脚本，上传 {} 时将创建新的云端脚本版本。",
                    local_version_label
                ),
                local_script_id: Some(local_script.id.to_string()),
                local_version_label: Some(local_version_label),
                remote_version_label: None,
                local_ver_num,
                remote_ver_num: None,
            }),
            None,
        );
    };

    let (remote_ver_name, remote_ver_num) = extract_cloud_summary_version(&summary);
    let remote_version_label = format_version_label(remote_ver_name.as_deref(), remote_ver_num);

    let preflight = match (local_ver_num, remote_ver_num) {
        (Some(local_ver_num), Some(remote_ver_num)) if local_ver_num > remote_ver_num => {
            ScriptVersionPreflight {
                status: "upgradeAvailable".to_string(),
                message: format!(
                    "云端当前为 {}，本地为 {}。继续后会将云端脚本更新到本地版本。",
                    remote_version_label, local_version_label
                ),
                local_script_id: Some(local_script.id.to_string()),
                local_version_label: Some(local_version_label),
                remote_version_label: Some(remote_version_label),
                local_ver_num: Some(local_ver_num),
                remote_ver_num: Some(remote_ver_num),
            }
        }
        (Some(local_ver_num), Some(remote_ver_num)) if local_ver_num < remote_ver_num => {
            ScriptVersionPreflight {
                status: "downgradeBlocked".to_string(),
                message: format!(
                    "云端当前为 {}，本地仅为 {}。不允许用较旧的本地版本覆盖云端脚本。",
                    remote_version_label, local_version_label
                ),
                local_script_id: Some(local_script.id.to_string()),
                local_version_label: Some(local_version_label),
                remote_version_label: Some(remote_version_label),
                local_ver_num: Some(local_ver_num),
                remote_ver_num: Some(remote_ver_num),
            }
        }
        _ => ScriptVersionPreflight {
            status: "sameVersion".to_string(),
            message: format!(
                "云端已存在相同版本 {}。继续后会覆盖云端当前内容。",
                remote_version_label
            ),
            local_script_id: Some(local_script.id.to_string()),
            local_version_label: Some(local_version_label),
            remote_version_label: Some(remote_version_label),
            local_ver_num,
            remote_ver_num,
        },
    };

    ApiResponse::success(Some(preflight), None)
}

#[command]
pub async fn backend_download_script(
    app_handle: AppHandle,
    script_id: String,
    runtime_type: String,
    replace_local_script_id: Option<String>,
) -> ApiResponse<String> {
    use crate::infrastructure::core::{PolicyGroupId, PolicyId, PolicySetId, ScriptId, TaskId};
    use crate::infrastructure::db::get_pool;

    let client = HttpClient::new(app_handle.clone());
    let url = format!("/scripts/download/{}?runtime_type={}", script_id, runtime_type);
    let req = ScriptDownloadReq {
        client: current_client_capability(),
    };
    let res: AppResult<BackendApiRes<ScriptUploadRequest>> = client.post(&url, &req).await;

    let mut download_data = match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                if let Some(data) = api_res.data {
                    data
                } else {
                    return ApiResponse::error(Some("返回数据为空".to_string()));
                }
            } else {
                return ApiResponse::error(Some(api_res.message));
            }
        }
        Err(error) => return ApiResponse::error(Some(error.to_string())),
    };

    if let Some(error) = validate_script_compatibility(&download_data.script.data) {
        return ApiResponse::error(Some(error));
    }

    let pool = get_pool();
    let replacement_target = match replace_local_script_id.as_deref() {
        Some(local_script_id) if !local_script_id.trim().is_empty() => {
            match sqlx::query_as::<_, ScriptTable>("SELECT id, `data` FROM scripts WHERE id = ?")
                .bind(local_script_id)
                .fetch_optional(pool)
                .await
            {
                Ok(Some(local_script)) => {
                    if local_script.data.script_type != ScriptType::Published {
                        return ApiResponse::error(Some("只能覆盖本地云端脚本副本".to_string()));
                    }
                    if local_script.data.cloud_id.as_ref().map(|value| value.to_string())
                        != Some(script_id.clone())
                    {
                        return ApiResponse::error(Some("本地脚本与当前云端脚本不匹配，无法覆盖".to_string()));
                    }
                    Some(local_script)
                }
                Ok(None) => {
                    return ApiResponse::error(Some("要覆盖的本地脚本不存在".to_string()));
                }
                Err(error) => {
                    return ApiResponse::error(Some(format!("读取本地脚本失败: {}", error)));
                }
            }
        }
        _ => None,
    };

    let local_script_id = replacement_target
        .as_ref()
        .map(|script| script.id.clone())
        .unwrap_or_else(ScriptId::new_v7);
    let existing_local_ver_num = replacement_target.as_ref().map(|script| script.data.ver_num);

    if let (Some(existing_ver_num), Some(remote_ver_num)) =
        (existing_local_ver_num, Some(download_data.script.data.ver_num))
    {
        if remote_ver_num < existing_ver_num {
            return ApiResponse::error(Some(format!(
                "本地已有 {}，云端当前仅为 {}。不允许用较旧的云端版本覆盖本地副本。",
                format_version_label(
                    replacement_target
                        .as_ref()
                        .map(|script| script.data.ver_name.as_str()),
                    version_num_to_i64(Some(existing_ver_num)),
                ),
                format_version_label(
                    Some(download_data.script.data.ver_name.as_str()),
                    version_num_to_i64(Some(remote_ver_num)),
                ),
            )));
        }
    }

    let old_script_id = download_data.script.id.clone();
    let mut policy_map: std::collections::HashMap<PolicyId, PolicyId> = std::collections::HashMap::new();
    let mut group_map: std::collections::HashMap<PolicyGroupId, PolicyGroupId> =
        std::collections::HashMap::new();
    let mut set_map: std::collections::HashMap<PolicySetId, PolicySetId> = std::collections::HashMap::new();

    download_data.script.id = local_script_id.clone();
    download_data.script.data.cloud_id = Some(old_script_id);
    download_data.script.data.script_type = ScriptType::Published;
    rewrite_script_model_paths_for_published(&mut download_data.script, &local_script_id.to_string());

    for policy in download_data.policies.iter_mut() {
        let new_pid = PolicyId::new_v7();
        policy_map.insert(policy.id.clone(), new_pid.clone());
        policy.id = new_pid;
        policy.script_id = local_script_id.clone();
    }

    for group in download_data.policy_groups.iter_mut() {
        let new_gid = PolicyGroupId::new_v7();
        group_map.insert(group.id.clone(), new_gid.clone());
        group.id = new_gid;
        group.script_id = local_script_id.clone();
    }

    for set in download_data.policy_sets.iter_mut() {
        let new_sid = PolicySetId::new_v7();
        set_map.insert(set.id.clone(), new_sid.clone());
        set.id = new_sid;
        set.script_id = local_script_id.clone();
    }

    for task in download_data.tasks.iter_mut() {
        let new_tid = TaskId::new_v7();
        task.id = new_tid;
        task.script_id = local_script_id.clone();
    }

    for gp in download_data.group_policies.iter_mut() {
        if let Some(new_gid) = group_map.get(&gp.group_id) {
            gp.group_id = new_gid.clone();
        }
        if let Some(new_pid) = policy_map.get(&gp.policy_id) {
            gp.policy_id = new_pid.clone();
        }
    }

    for sg in download_data.set_groups.iter_mut() {
        if let Some(new_sid) = set_map.get(&sg.set_id) {
            sg.set_id = new_sid.clone();
        }
        if let Some(new_gid) = group_map.get(&sg.group_id) {
            sg.group_id = new_gid.clone();
        }
    }

    if let Err(error) = download_script_models(
        &client,
        &local_script_id.to_string(),
        &download_data.model_files,
        &local_scripts_dir(&app_handle),
    )
    .await
    {
        return ApiResponse::error(Some(error));
    }

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(error) => return ApiResponse::error(Some(format!("开启事务失败: {}", error))),
    };

    if let Some(existing_script) = replacement_target.as_ref() {
        if let Err(error) = delete_local_script_graph(&mut tx, &existing_script.id).await {
            return ApiResponse::error(Some(error));
        }
    }

    if let Err(error) = crate::api::domain::script_batch_insert::batch_insert_script_related(
        &mut tx,
        &download_data.script,
        &download_data.policies,
        &download_data.policy_groups,
        &download_data.policy_sets,
        &download_data.group_policies,
        &download_data.set_groups,
        &download_data.tasks,
    )
    .await
    {
        return ApiResponse::error(Some(error));
    }

    if let Err(error) = tx.commit().await {
        return ApiResponse::error(Some(format!("提交事务失败: {}", error)));
    }

    ApiResponse::success(
        Some(local_script_id.to_string()),
        Some(match existing_local_ver_num {
            Some(existing_ver_num) if existing_ver_num == download_data.script.data.ver_num => {
                "相同版本已覆盖到本地库".to_string()
            }
            Some(_) => "云端脚本已更新到本地库".to_string(),
            None => "脚本下载并写入成功".to_string(),
        }),
    )
}

#[command]
pub async fn backend_upload_script(
    app_handle: AppHandle,
    script_id: String,
) -> ApiResponse<String> {
    use crate::domain::scripts::policy::*;
    use crate::domain::scripts::script_task::ScriptTaskTable;
    use crate::infrastructure::db::get_pool;

    let pool = get_pool();
    let script: Option<ScriptTable> = sqlx::query_as("SELECT id, `data` FROM scripts WHERE id = ?")
        .bind(&script_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    let mut script = match script {
        Some(script) => script,
        None => return ApiResponse::error(Some("脚本不存在".to_string())),
    };

    if script.data.script_type != ScriptType::Dev {
        return ApiResponse::error(Some("只有开发中 (Dev) 的脚本才能被上传".to_string()));
    }

    let runtime_type = match runtime_type_param(&script.data.runtime_type) {
        Ok(value) => value,
        Err(error) => return ApiResponse::error(Some(error)),
    };

    let scripts_root = local_scripts_dir(&app_handle);
    let model_uploads = match collect_model_uploads(&script, &scripts_root) {
        Ok(value) => value,
        Err(error) => return ApiResponse::error(Some(error)),
    };

    let client = HttpClient::new(app_handle.clone());
    let auth_session = match client.get_auth_session() {
        Some(session) => session,
        None => return ApiResponse::error(Some("上传前请先登录".to_string())),
    };

    if auth_session.username.trim().is_empty() {
        return ApiResponse::error(Some("当前登录态缺少用户名，请重新登录".to_string()));
    }

    let summary_url = format!("/scripts/{}/summary", script_id);
    let summary_response: AppResult<BackendApiRes<serde_json::Value>> = client.get(&summary_url).await;
    let local_ver_num = version_num_to_i64(Some(script.data.ver_num));
    let local_version_label = format_version_label(Some(script.data.ver_name.as_str()), local_ver_num);
    match summary_response {
        Ok(api_res) if api_res.code == 200 => {
            if let Some(summary) = api_res.data {
                let (remote_ver_name, remote_ver_num) = extract_cloud_summary_version(&summary);
                if let (Some(local_ver_num), Some(remote_ver_num)) = (local_ver_num, remote_ver_num) {
                    if local_ver_num < remote_ver_num {
                        return ApiResponse::error(Some(format!(
                            "云端当前为 {}，本地仅为 {}。不允许用较旧的本地版本覆盖云端脚本。",
                            format_version_label(remote_ver_name.as_deref(), Some(remote_ver_num)),
                            local_version_label,
                        )));
                    }
                }
            }
        }
        Ok(api_res) => {
            return ApiResponse::failed_with_details(
                None,
                Some(format_backend_message(&api_res.message, api_res.details.as_ref())),
                api_res.details,
            );
        }
        Err(error) => return ApiResponse::error(Some(app_error_message(error))),
    }

    let script_user_name = script.data.user_name.as_deref().unwrap_or("").trim();
    if auth_session.username == "Guest" || script_user_name.is_empty() || script_user_name == "Guest" {
        let author = match fetch_upload_author(&client).await {
            Ok(author) => author,
            Err(error) => return ApiResponse::error(Some(error)),
        };

        script.data.user_id = author.id;
        script.data.user_name = Some(author.username);

        if let Err(error) = DbRepo::upsert_id_data(SCRIPT_TABLE, &script.id.to_string(), &script.data).await {
            return ApiResponse::error(Some(format!("更新本地脚本作者信息失败: {}", error)));
        }
    }

    let published_script_id = script.id.to_string();
    rewrite_script_model_paths_for_published(&mut script, &published_script_id);

    let policies: Vec<PolicyTable> = sqlx::query_as(
        "SELECT id, script_id, order_index, `data` FROM policies WHERE script_id = ?",
    )
    .bind(&script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let policy_groups: Vec<PolicyGroupTable> = sqlx::query_as(
        "SELECT id, script_id, order_index, `data` FROM policy_groups WHERE script_id = ?",
    )
    .bind(&script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let policy_sets: Vec<PolicySetTable> = sqlx::query_as(
        "SELECT id, script_id, order_index, `data` FROM policy_sets WHERE script_id = ?",
    )
    .bind(&script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let tasks: Vec<ScriptTaskTable> =
        sqlx::query_as("SELECT * FROM script_tasks WHERE script_id = ? AND is_deleted = false")
            .bind(&script_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let group_policies: Vec<GroupPolicyRelation> = sqlx::query_as(
        "SELECT gp.group_id, gp.policy_id, gp.order_index FROM group_policies gp 
         JOIN policy_groups g ON gp.group_id = g.id WHERE g.script_id = ?",
    )
    .bind(&script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let set_groups: Vec<SetGroupRelation> = sqlx::query_as(
        "SELECT sg.set_id, sg.group_id, sg.order_index FROM set_groups sg 
         JOIN policy_sets s ON sg.set_id = s.id WHERE s.script_id = ?",
    )
    .bind(&script_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let script_version = script.data.ver_num;
    let upload_req = ScriptUploadRequest {
        script,
        policies,
        tasks,
        policy_groups,
        policy_sets,
        group_policies,
        set_groups,
        model_files: build_model_file_payload(
            &script_id,
            script_version,
            runtime_type.as_str(),
            &model_uploads,
        ),
    };

    let res = client
        .post_api_res::<serde_json::Value, _>(
            &format!("/scripts/upload?runtime_type={}", runtime_type),
            &upload_req,
        )
        .await;

    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                for model in &model_uploads {
                    let upload_url = format!(
                        "/scripts/upload/model/{}/{}?runtime_type={}",
                        script_id, model.type_name, runtime_type
                    );
                    let upload_res: AppResult<BackendApiRes<String>> = client
                        .upload_file(&upload_url, &model.local_path, "file", model.file_name)
                        .await;
                    match upload_res {
                        Ok(model_api_res) if model_api_res.code == 200 => {}
                        Ok(model_api_res) => {
                            return ApiResponse::error(Some(format!(
                                "脚本已上传，但模型 {} 上传失败: {}",
                                model.file_name, model_api_res.message
                            )));
                        }
                        Err(error) => {
                            return ApiResponse::error(Some(format!(
                                "脚本已上传，但模型 {} 上传失败: {}",
                                model.file_name, error
                            )));
                        }
                    }
                }
                ApiResponse::success(None, Some(api_res.message))
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

fn validate_script_compatibility(
    script: &runtime_engine::domain::scripts::script_info::ScriptInfo,
) -> Option<String> {
    let client = current_client_capability();

    if let Some(required_schema) = script.min_runtime_schema {
        if required_schema > client.runtime_schema {
            return Some(format!(
                "该脚本需要运行时结构版本 {}，当前程序仅支持 {}，请先更新程序",
                required_schema, client.runtime_schema
            ));
        }
    }

    if let Some(required_version) = script.min_app_version.as_deref() {
        if is_version_greater(required_version, &client.app_version) {
            return Some(format!(
                "该脚本需要 AutoDaily {} 或以上版本，当前版本为 {}，请先更新程序",
                required_version, client.app_version
            ));
        }
    }

    let missing_features: Vec<&str> = script
        .required_features
        .iter()
        .map(String::as_str)
        .filter(|feature| {
            !client
                .supported_features
                .iter()
                .any(|supported| supported == feature)
        })
        .collect();

    if !missing_features.is_empty() {
        return Some(format!(
            "该脚本需要当前程序不支持的能力: {}，请先更新程序",
            missing_features
                .iter()
                .map(|feature| script_feature_label(feature))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    None
}

fn script_feature_label(feature: &str) -> &str {
    match feature {
        "onnxInference" => "ONNX推理",
        "runtime:javascript" => "JavaScript运行时",
        "runtime:rhai" => "Rhai运行时",
        "runtime:lua" => "Lua运行时",
        "llmApi" => "LLM API接口",
        "device:android" => "Android目标设备",
        "device:desktop" => "Desktop目标设备",
        _ => feature,
    }
}

fn is_version_greater(required: &str, current: &str) -> bool {
    let parse = |version: &str| {
        version
            .split(|ch: char| !ch.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .take(3)
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };

    let required_parts = parse(required);
    let current_parts = parse(current);

    for index in 0..3 {
        let required_part = *required_parts.get(index).unwrap_or(&0);
        let current_part = *current_parts.get(index).unwrap_or(&0);
        if required_part != current_part {
            return required_part > current_part;
        }
    }

    false
}

#[derive(Clone, Copy)]
struct ModelTypeSpec {
    type_name: &'static str,
    file_name: &'static str,
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

fn normalize_model_type(value: &str) -> Result<ModelTypeSpec, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "img_det_model" | "img-det-model" | "imgdetmodel" | "det" => Ok(IMG_DET_MODEL),
        "txt_det_model" | "txt-det-model" | "txtdetmodel" | "txt-det" => Ok(TXT_DET_MODEL),
        "txt_rec_model" | "txt-rec-model" | "txtrecmodel" | "rec" => Ok(TXT_REC_MODEL),
        other => Err(format!("不支持的模型类型: {}", other)),
    }
}

fn runtime_type_param(runtime_type: &RuntimeType) -> Result<String, String> {
    serde_json::to_value(runtime_type)
        .map_err(|error| format!("序列化 runtime_type 失败: {}", error))?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "runtime_type 序列化结果不是字符串".to_string())
}

fn local_scripts_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .store(APP_STORE)
        .map(|store| get_or_init_config::<ScriptsConfig>(store, SCRIPTS_CONFIG_KEY).dir)
        .unwrap_or_else(|_| ScriptsConfig::default().dir)
}

fn build_model_file_payload(
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

fn collect_model_uploads(
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

fn rewrite_script_model_paths_for_published(script: &mut ScriptTable, script_id: &str) {
    rewrite_detector_model_path(&mut script.data.img_det_model, script_id, IMG_DET_MODEL.file_name);
    rewrite_detector_model_path(&mut script.data.txt_det_model, script_id, TXT_DET_MODEL.file_name);
    rewrite_recognizer_model_path(&mut script.data.txt_rec_model, script_id, TXT_REC_MODEL.file_name);
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

async fn download_script_models(
    client: &HttpClient,
    local_script_id: &str,
    model_files: &[ScriptModelFileDto],
    scripts_root: &Path,
) -> Result<(), String> {
    if model_files.is_empty() {
        return Ok(());
    }

    let script_dir = scripts_root.join(local_script_id);
    std::fs::create_dir_all(&script_dir)
        .map_err(|error| format!("创建脚本模型目录 {} 失败: {}", script_dir.display(), error))?;

    for model in model_files {
        let target = normalize_model_type(model.r#type.as_str())?;
        let endpoint = normalize_download_endpoint(model.download_path.as_str())?;
        if let Some(hash_algorithm) = model.hash_algorithm.as_deref() {
            if !hash_algorithm.eq_ignore_ascii_case("SHA-256") {
                return Err(format!(
                    "模型 {} 使用了不支持的 hash 算法: {}",
                    model.file_name, hash_algorithm
                ));
            }
        }

        client
            .download_file_with_resume(
                endpoint.as_str(),
                &script_dir.join(target.file_name),
                model.hash_value.as_deref(),
            )
            .await
            .map_err(|error| format!("下载模型 {} 失败: {}", model.file_name, error))?;
    }

    Ok(())
}

fn normalize_download_endpoint(download_path: &str) -> Result<String, String> {
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
        Err(error) => ApiResponse::error(Some(error.to_string())),
    }
}

async fn fetch_upload_author(client: &HttpClient) -> Result<UploadAuthor, String> {
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get("/user/profile").await;
    let api_res = res.map_err(|error| error.to_string())?;

    if api_res.code != 200 {
        return Err(api_res.message);
    }

    let payload = api_res.data.ok_or_else(|| "用户资料为空".to_string())?;
    let id = payload
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "用户资料缺少 id".to_string())?;
    let username = payload
        .get("username")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "用户资料缺少 username".to_string())?;

    let uuid = uuid::Uuid::parse_str(id).map_err(|error| format!("用户 id 非法: {}", error))?;

    Ok(UploadAuthor {
        id: UserId::from(uuid),
        username: username.to_string(),
    })
}

async fn delete_local_script_graph(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    script_id: &crate::infrastructure::core::ScriptId,
) -> Result<(), String> {
    let policy_group_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM policy_groups WHERE script_id = ?")
        .bind(script_id.to_string())
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| format!("读取本地策略组失败: {}", error))?;

    for group_id in policy_group_ids {
        sqlx::query("DELETE FROM group_policies WHERE group_id = ?")
            .bind(group_id)
            .execute(&mut **tx)
            .await
            .map_err(|error| format!("删除本地策略组关联失败: {}", error))?;
    }

    let policy_set_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM policy_sets WHERE script_id = ?")
        .bind(script_id.to_string())
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| format!("读取本地策略集失败: {}", error))?;

    for set_id in policy_set_ids {
        sqlx::query("DELETE FROM set_groups WHERE set_id = ?")
            .bind(set_id)
            .execute(&mut **tx)
            .await
            .map_err(|error| format!("删除本地策略集关联失败: {}", error))?;
    }

    for query in [
        "DELETE FROM policies WHERE script_id = ?",
        "DELETE FROM script_tasks WHERE script_id = ?",
        "DELETE FROM policy_groups WHERE script_id = ?",
        "DELETE FROM policy_sets WHERE script_id = ?",
        "DELETE FROM scripts WHERE id = ?",
    ] {
        sqlx::query(query)
            .bind(script_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|error| format!("删除本地云端脚本旧副本失败: {}", error))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_type_query_values_match_server_contract() {
        assert_eq!(runtime_type_param(&RuntimeType::Rhai).unwrap(), "rhai");
        assert_eq!(
            runtime_type_param(&RuntimeType::JavaScript).unwrap(),
            "javaScript"
        );
        assert_eq!(runtime_type_param(&RuntimeType::Lua).unwrap(), "lua");
        assert_eq!(
            runtime_type_param(&RuntimeType::AIAndVision).unwrap(),
            "aIAndVision"
        );
        assert_eq!(runtime_type_param(&RuntimeType::AI).unwrap(), "aI");
    }

    #[test]
    fn download_endpoint_strips_api_prefix() {
        assert_eq!(
            normalize_download_endpoint("/api/scripts/download/model/1/2/x").unwrap(),
            "/scripts/download/model/1/2/x"
        );
        assert_eq!(
            normalize_download_endpoint("/scripts/download/model/1/2/x").unwrap(),
            "/scripts/download/model/1/2/x"
        );
    }
}
