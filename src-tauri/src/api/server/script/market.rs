use super::model::{
    build_model_file_payload, collect_model_uploads, local_scripts_dir, model_hash_matches_path,
    model_hash_matches_sha256, normalize_download_endpoint, normalize_model_type,
    rewrite_script_model_paths_for_published, runtime_type_param, verification_sha256,
};
use crate::api::local::script::transfer_records::{
    ScriptTransferControl, ScriptTransferControlState, ScriptTransferProgressEvent,
    emit_script_transfer_event, finish_script_transfer_record, insert_script_transfer_record,
    now_rfc3339, register_script_transfer_control, unregister_script_transfer_control,
};
use crate::api::response::ApiResponse;
use crate::api::server::dto::*;
use crate::api::server::script::preflight::{
    ScriptVersionPreflight, build_download_preflight, extract_cloud_summary_version,
    find_replaceable_local_published_script, format_version_label,
};
use crate::api::server::{app_error_message, format_backend_message, trans_api_res};
use crate::app::app_error::AppResult;
use crate::infra::http_client::HttpClient;
use crate::infra::logging::log_trait::Log;
use ad_kernel::ids::UserId;
use domain_script::ScriptType;
use infra_sqlite::{
    CreateScriptTransferRecordInput, FinishScriptTransferRecordInput, get_script,
    list_group_policy_links, list_policies, list_policy_groups, list_policy_sets,
    list_script_tasks, list_set_group_links, save_script,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, command};

const TRANSFER_DELETED_MESSAGE: &str = "传输已删除";
#[path = "market/script_dir_swap.rs"]
mod script_dir_swap;
use script_dir_swap::{
    activate as activate_script_dir_swap, cleanup as cleanup_script_dir_swap_temp,
    prepare as prepare_script_dir_swap, rollback as rollback_script_dir_swap,
};

#[derive(Debug)]
struct UploadAuthor {
    id: UserId,
    username: String,
}

#[derive(Clone)]
struct ScriptTransferRun {
    app_handle: AppHandle,
    id: String,
    direction: &'static str,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
    script_name: Option<String>,
    model_file_count: i64,
    total_bytes: i64,
    started_at: String,
    control: Arc<ScriptTransferControl>,
}

impl ScriptTransferRun {
    fn new(
        app_handle: AppHandle,
        direction: &'static str,
        local_script_id: Option<String>,
        cloud_script_id: Option<String>,
        script_name: Option<String>,
        model_file_count: i64,
        total_bytes: i64,
    ) -> Self {
        let id = uuid::Uuid::now_v7().to_string();
        Self {
            app_handle,
            control: register_script_transfer_control(&id),
            id,
            direction,
            local_script_id,
            cloud_script_id,
            script_name,
            model_file_count,
            total_bytes,
            started_at: now_rfc3339(),
        }
    }

    async fn start(&self, latest_message: Option<String>) {
        if let Err(error) = insert_script_transfer_record(CreateScriptTransferRecordInput {
            id: self.id.clone(),
            direction: self.direction.to_string(),
            local_script_id: self.local_script_id.clone(),
            cloud_script_id: self.cloud_script_id.clone(),
            script_name: self.script_name.clone(),
            status: "running".to_string(),
            model_file_count: self.model_file_count,
            completed_model_file_count: 0,
            latest_file_name: None,
            bytes_transferred: 0,
            total_bytes: self.total_bytes,
            latest_message: latest_message.clone(),
            error_message: None,
            started_at: self.started_at.clone(),
            finished_at: None,
        })
        .await
        {
            Log::error(&format!("写入传输记录失败: {}", error));
        }

        self.emit("running", 0, None, None, 0, latest_message, None, None);
    }

    fn emit(
        &self,
        status: &str,
        completed_model_file_count: i64,
        current_file_name: Option<String>,
        latest_file_name: Option<String>,
        bytes_transferred: i64,
        latest_message: Option<String>,
        error_message: Option<String>,
        finished_at: Option<String>,
    ) {
        emit_script_transfer_event(
            &self.app_handle,
            &ScriptTransferProgressEvent {
                id: self.id.clone(),
                direction: self.direction.to_string(),
                local_script_id: self.local_script_id.clone(),
                cloud_script_id: self.cloud_script_id.clone(),
                script_name: self.script_name.clone(),
                status: status.to_string(),
                model_file_count: self.model_file_count,
                completed_model_file_count,
                current_file_name,
                latest_file_name,
                bytes_transferred,
                total_bytes: self.total_bytes,
                latest_message,
                error_message,
                started_at: self.started_at.clone(),
                finished_at,
                updated_at: now_rfc3339(),
            },
        );
    }

    fn checkpoint_blocking(
        &self,
        completed_model_file_count: i64,
        current_file_name: Option<String>,
        latest_file_name: Option<String>,
        bytes_transferred: i64,
        running_message: String,
    ) -> Result<(), String> {
        tokio::task::block_in_place(|| {
            tauri::async_runtime::block_on(self.checkpoint_async(
                completed_model_file_count,
                current_file_name,
                latest_file_name,
                bytes_transferred,
                running_message,
            ))
        })
    }

    async fn checkpoint_async(
        &self,
        completed_model_file_count: i64,
        current_file_name: Option<String>,
        latest_file_name: Option<String>,
        bytes_transferred: i64,
        running_message: String,
    ) -> Result<(), String> {
        let mut paused_emitted = false;
        loop {
            match self.control.state() {
                ScriptTransferControlState::Running => {
                    if paused_emitted {
                        self.emit(
                            "running",
                            completed_model_file_count,
                            current_file_name.clone(),
                            latest_file_name.clone(),
                            bytes_transferred,
                            Some(running_message.clone()),
                            None,
                            None,
                        );
                    }
                    return Ok(());
                }
                ScriptTransferControlState::Paused => {
                    if !paused_emitted {
                        self.emit(
                            "paused",
                            completed_model_file_count,
                            current_file_name.clone(),
                            latest_file_name.clone(),
                            bytes_transferred,
                            Some("已暂停".to_string()),
                            None,
                            None,
                        );
                        paused_emitted = true;
                    }
                    self.control.wait_for_signal().await;
                }
                ScriptTransferControlState::DeleteRequested => {
                    return Err(TRANSFER_DELETED_MESSAGE.to_string());
                }
            }
        }
    }

    fn close(&self) {
        unregister_script_transfer_control(&self.id);
    }

    async fn finish(
        &self,
        status: &str,
        completed_model_file_count: i64,
        latest_file_name: Option<String>,
        bytes_transferred: i64,
        latest_message: Option<String>,
        error_message: Option<String>,
    ) {
        if self.control.state() == ScriptTransferControlState::DeleteRequested {
            self.close();
            return;
        }

        let finished_at = Some(now_rfc3339());
        if let Err(error) = finish_script_transfer_record(FinishScriptTransferRecordInput {
            id: self.id.clone(),
            status: status.to_string(),
            completed_model_file_count,
            latest_file_name: latest_file_name.clone(),
            bytes_transferred,
            total_bytes: self.total_bytes,
            latest_message: latest_message.clone(),
            error_message: error_message.clone(),
            finished_at: finished_at.clone(),
        })
        .await
        {
            Log::error(&format!("更新传输记录失败: {}", error));
        }

        self.emit(
            status,
            completed_model_file_count,
            None,
            latest_file_name,
            bytes_transferred,
            latest_message,
            error_message,
            finished_at,
        );

        self.close();
    }
}

fn is_transfer_deleted_error(error: &crate::app::app_error::AppError) -> bool {
    match error {
        crate::app::app_error::AppError::HttpErr { detail, e } => {
            detail.contains(TRANSFER_DELETED_MESSAGE) || e.contains(TRANSFER_DELETED_MESSAGE)
        }
        _ => false,
    }
}

async fn record_immediate_transfer_terminal_state(
    app_handle: &AppHandle,
    direction: &'static str,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
    script_name: Option<String>,
    status: &'static str,
    latest_message: String,
    error_message: Option<String>,
) {
    let record_id = uuid::Uuid::now_v7().to_string();
    let started_at = now_rfc3339();
    let finished_at = Some(now_rfc3339());

    if let Err(error) = insert_script_transfer_record(CreateScriptTransferRecordInput {
        id: record_id.clone(),
        direction: direction.to_string(),
        local_script_id: local_script_id.clone(),
        cloud_script_id: cloud_script_id.clone(),
        script_name: script_name.clone(),
        status: status.to_string(),
        model_file_count: 0,
        completed_model_file_count: 0,
        latest_file_name: None,
        bytes_transferred: 0,
        total_bytes: 0,
        latest_message: Some(latest_message.clone()),
        error_message: error_message.clone(),
        started_at: started_at.clone(),
        finished_at: finished_at.clone(),
    })
    .await
    {
        Log::error(&format!("写入即时传输记录失败: {}", error));
    }

    emit_script_transfer_event(
        app_handle,
        &ScriptTransferProgressEvent {
            id: record_id,
            direction: direction.to_string(),
            local_script_id,
            cloud_script_id,
            script_name,
            status: status.to_string(),
            model_file_count: 0,
            completed_model_file_count: 0,
            current_file_name: None,
            latest_file_name: None,
            bytes_transferred: 0,
            total_bytes: 0,
            latest_message: Some(latest_message),
            error_message,
            started_at,
            finished_at,
            updated_at: now_rfc3339(),
        },
    );
}

async fn fetch_remote_model_file_index(
    client: &HttpClient,
    script_id: &str,
    runtime_type: &str,
) -> Option<HashMap<String, ScriptModelFileDto>> {
    let url = format!(
        "/scripts/download/{}?runtime_type={}",
        script_id, runtime_type
    );
    let req = ScriptDownloadReq {
        client: current_client_capability(),
    };
    let response: AppResult<BackendApiRes<ScriptUploadRequest>> = client.post(&url, &req).await;
    let api_res = match response {
        Ok(value) => value,
        Err(error) => {
            Log::warn(&format!(
                "获取云端模型元数据失败，将回退为完整模型上传: {}",
                app_error_message(error)
            ));
            return None;
        }
    };
    if api_res.code != 200 {
        Log::warn(&format!(
            "获取云端模型元数据返回失败，将回退为完整模型上传: {}",
            api_res.message
        ));
        return None;
    }
    let Some(data) = api_res.data else {
        return None;
    };

    Some(
        data.model_files
            .into_iter()
            .map(|model| (model.r#type.to_ascii_lowercase(), model))
            .collect(),
    )
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
    ver_num: Option<u32>,
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
    let local_script_id = match uuid::Uuid::parse_str(&script_id) {
        Ok(id) => id.into(),
        Err(_) => return ApiResponse::error(Some("脚本 ID 格式无效".to_string())),
    };
    let local_script = match get_script(local_script_id).await {
        Ok(Some(script)) => script,
        Ok(None) => return ApiResponse::error(Some("脚本不存在".to_string())),
        Err(error) => return ApiResponse::error(Some(format!("读取本地脚本失败: {}", error))),
    };

    if local_script.info.script_type != ScriptType::Dev {
        return ApiResponse::error(Some("只有本地脚本 (Dev) 才能被上传".to_string()));
    }

    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/{}/summary", script_id);
    let response: AppResult<BackendApiRes<serde_json::Value>> = client.get(&url).await;
    let local_ver_num = Some(local_script.info.ver_num);
    let local_version_label =
        format_version_label(Some(local_script.info.ver_name.as_str()), local_ver_num);

    let api_res = match response {
        Ok(value) => value,
        Err(error) => return ApiResponse::error(Some(app_error_message(error))),
    };

    if api_res.code != 200 {
        return ApiResponse::failed_with_details(
            None,
            Some(format_backend_message(
                &api_res.message,
                api_res.details.as_ref(),
            )),
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
    script_name: Option<String>,
) -> ApiResponse<String> {
    use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId, ScriptId, TaskId};
    use infra_sqlite::get_pool;
    let pool = get_pool();

    let client = HttpClient::new(app_handle.clone());
    let url = format!(
        "/scripts/download/{}?runtime_type={}",
        script_id, runtime_type
    );
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
                    let message = "返回数据为空".to_string();
                    record_immediate_transfer_terminal_state(
                        &app_handle,
                        "download",
                        replace_local_script_id.clone(),
                        Some(script_id.clone()),
                        script_name.clone(),
                        "error",
                        message.clone(),
                        Some(message.clone()),
                    )
                    .await;
                    return ApiResponse::error(Some("返回数据为空".to_string()));
                }
            } else {
                record_immediate_transfer_terminal_state(
                    &app_handle,
                    "download",
                    replace_local_script_id.clone(),
                    Some(script_id.clone()),
                    script_name.clone(),
                    "error",
                    api_res.message.clone(),
                    Some(api_res.message.clone()),
                )
                .await;
                return ApiResponse::error(Some(api_res.message));
            }
        }
        Err(error) => {
            let message = app_error_message(error);
            record_immediate_transfer_terminal_state(
                &app_handle,
                "download",
                replace_local_script_id.clone(),
                Some(script_id.clone()),
                script_name.clone(),
                "error",
                message.clone(),
                Some(message.clone()),
            )
            .await;
            return ApiResponse::error(Some(message));
        }
    };

    if let Some(error) = crate::api::server::script::preflight::validate_script_compatibility(
        &download_data.script.info,
    ) {
        record_immediate_transfer_terminal_state(
            &app_handle,
            "download",
            replace_local_script_id.clone(),
            Some(script_id.clone()),
            script_name
                .clone()
                .or_else(|| Some(download_data.script.info.name.clone())),
            "error",
            error.clone(),
            Some(error.clone()),
        )
        .await;
        return ApiResponse::error(Some(error));
    }

    let replacement_target = match replace_local_script_id.as_deref() {
        Some(local_script_id) if !local_script_id.trim().is_empty() => {
            let local_script_id = match uuid::Uuid::parse_str(local_script_id) {
                Ok(id) => id.into(),
                Err(_) => {
                    return ApiResponse::error(Some("要覆盖的本地脚本 ID 格式无效".to_string()));
                }
            };
            match get_script(local_script_id).await {
                Ok(Some(local_script)) => {
                    if local_script.info.script_type != ScriptType::Published {
                        record_immediate_transfer_terminal_state(
                            &app_handle,
                            "download",
                            Some(local_script_id.to_string()),
                            Some(script_id.clone()),
                            script_name
                                .clone()
                                .or_else(|| Some(download_data.script.info.name.clone())),
                            "error",
                            "只能覆盖本地云端脚本副本".to_string(),
                            Some("只能覆盖本地云端脚本副本".to_string()),
                        )
                        .await;
                        return ApiResponse::error(Some("只能覆盖本地云端脚本副本".to_string()));
                    }
                    if local_script
                        .info
                        .cloud_id
                        .as_ref()
                        .map(|value| value.to_string())
                        != Some(script_id.clone())
                    {
                        let message = "本地脚本与当前云端脚本不匹配，无法覆盖".to_string();
                        record_immediate_transfer_terminal_state(
                            &app_handle,
                            "download",
                            Some(local_script_id.to_string()),
                            Some(script_id.clone()),
                            script_name
                                .clone()
                                .or_else(|| Some(download_data.script.info.name.clone())),
                            "error",
                            message.clone(),
                            Some(message.clone()),
                        )
                        .await;
                        return ApiResponse::error(Some(
                            "本地脚本与当前云端脚本不匹配，无法覆盖".to_string(),
                        ));
                    }
                    Some(local_script)
                }
                Ok(None) => {
                    let message = "要覆盖的本地脚本不存在".to_string();
                    record_immediate_transfer_terminal_state(
                        &app_handle,
                        "download",
                        Some(local_script_id.to_string()),
                        Some(script_id.clone()),
                        script_name
                            .clone()
                            .or_else(|| Some(download_data.script.info.name.clone())),
                        "error",
                        message.clone(),
                        Some(message.clone()),
                    )
                    .await;
                    return ApiResponse::error(Some(message));
                }
                Err(error) => {
                    let message = format!("读取本地脚本失败: {}", error);
                    record_immediate_transfer_terminal_state(
                        &app_handle,
                        "download",
                        Some(local_script_id.to_string()),
                        Some(script_id.clone()),
                        script_name
                            .clone()
                            .or_else(|| Some(download_data.script.info.name.clone())),
                        "error",
                        message.clone(),
                        Some(message.clone()),
                    )
                    .await;
                    return ApiResponse::error(Some(message));
                }
            }
        }
        _ => None,
    };

    let local_script_id = replacement_target
        .as_ref()
        .map(|script| script.id.clone())
        .unwrap_or_else(ScriptId::new_v7);
    let existing_local_ver_num = replacement_target
        .as_ref()
        .map(|script| script.info.ver_num);

    if let (Some(existing_ver_num), Some(remote_ver_num)) = (
        existing_local_ver_num,
        Some(download_data.script.info.ver_num),
    ) {
        if remote_ver_num < existing_ver_num {
            let message = format!(
                "本地已有 {}，云端当前仅为 {}。不允许用较旧的云端版本覆盖本地副本。",
                format_version_label(
                    replacement_target
                        .as_ref()
                        .map(|script| script.info.ver_name.as_str()),
                    Some(existing_ver_num),
                ),
                format_version_label(
                    Some(download_data.script.info.ver_name.as_str()),
                    Some(remote_ver_num),
                ),
            );
            record_immediate_transfer_terminal_state(
                &app_handle,
                "download",
                replacement_target
                    .as_ref()
                    .map(|script| script.id.to_string()),
                Some(script_id.clone()),
                script_name
                    .clone()
                    .or_else(|| Some(download_data.script.info.name.clone())),
                "error",
                message.clone(),
                Some(message.clone()),
            )
            .await;
            return ApiResponse::error(Some(format!(
                "本地已有 {}，云端当前仅为 {}。不允许用较旧的云端版本覆盖本地副本。",
                format_version_label(
                    replacement_target
                        .as_ref()
                        .map(|script| script.info.ver_name.as_str()),
                    Some(existing_ver_num),
                ),
                format_version_label(
                    Some(download_data.script.info.ver_name.as_str()),
                    Some(remote_ver_num),
                ),
            )));
        }
    }

    let old_script_id = download_data.script.id.clone();
    let mut policy_map: std::collections::HashMap<PolicyId, PolicyId> =
        std::collections::HashMap::new();
    let mut group_map: std::collections::HashMap<PolicyGroupId, PolicyGroupId> =
        std::collections::HashMap::new();
    let mut set_map: std::collections::HashMap<PolicySetId, PolicySetId> =
        std::collections::HashMap::new();

    download_data.script.id = local_script_id.clone();
    download_data.script.info.cloud_id = Some(old_script_id);
    download_data.script.info.script_type = ScriptType::Published;
    rewrite_script_model_paths_for_published(
        &mut download_data.script,
        &local_script_id.to_string(),
    );

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

    let transfer_total_bytes = download_data
        .model_files
        .iter()
        .map(|model| model.size_bytes.unwrap_or(0) as i64)
        .sum::<i64>();
    let transfer_model_count = download_data.model_files.len() as i64;
    let transfer_run = ScriptTransferRun::new(
        app_handle.clone(),
        "download",
        Some(local_script_id.to_string()),
        Some(script_id.clone()),
        Some(download_data.script.info.name.clone()),
        transfer_model_count,
        transfer_total_bytes,
    );
    transfer_run
        .start(Some(if transfer_model_count > 0 {
            "正在下载模型文件".to_string()
        } else {
            "正在写入脚本数据".to_string()
        }))
        .await;

    let scripts_root = local_scripts_dir(&app_handle);
    let mut dir_swap = match prepare_script_dir_swap(
        &scripts_root,
        &local_script_id.to_string(),
        transfer_run.id.as_str(),
    ) {
        Ok(state) => state,
        Err(error) => {
            transfer_run
                .finish(
                    "error",
                    0,
                    None,
                    0,
                    Some("创建脚本模型暂存目录失败".to_string()),
                    Some(error.clone()),
                )
                .await;
            return ApiResponse::error(Some(error));
        }
    };

    if let Err(error) = std::fs::create_dir_all(&dir_swap.staging_dir) {
        transfer_run
            .finish(
                "error",
                0,
                None,
                0,
                Some("创建脚本模型暂存目录失败".to_string()),
                Some(error.to_string()),
            )
            .await;
        cleanup_script_dir_swap_temp(&dir_swap);
        return ApiResponse::error(Some(format!(
            "创建脚本模型暂存目录 {} 失败: {}",
            dir_swap.staging_dir.display(),
            error
        )));
    }

    let mut completed_model_count = 0_i64;
    let mut bytes_transferred = 0_i64;
    let mut latest_file_name: Option<String> = None;

    for model in &download_data.model_files {
        let normalized_type = match normalize_model_type(model.r#type.as_str()) {
            Ok(value) => value,
            Err(error) => {
                transfer_run
                    .finish(
                        "error",
                        completed_model_count,
                        latest_file_name.clone(),
                        bytes_transferred,
                        Some("解析模型类型失败".to_string()),
                        Some(error.clone()),
                    )
                    .await;
                cleanup_script_dir_swap_temp(&dir_swap);
                return ApiResponse::error(Some(error));
            }
        };
        let endpoint = match normalize_download_endpoint(model.download_path.as_str()) {
            Ok(value) => value,
            Err(error) => {
                transfer_run
                    .finish(
                        "error",
                        completed_model_count,
                        latest_file_name.clone(),
                        bytes_transferred,
                        Some("解析模型下载地址失败".to_string()),
                        Some(error.clone()),
                    )
                    .await;
                cleanup_script_dir_swap_temp(&dir_swap);
                return ApiResponse::error(Some(error));
            }
        };
        latest_file_name = Some(model.file_name.clone());
        if let Err(message) = transfer_run
            .checkpoint_async(
                completed_model_count,
                latest_file_name.clone(),
                latest_file_name.clone(),
                bytes_transferred,
                format!("正在下载模型 {}", model.file_name),
            )
            .await
        {
            transfer_run.close();
            cleanup_script_dir_swap_temp(&dir_swap);
            return ApiResponse::error(Some(message));
        }
        transfer_run.emit(
            "running",
            completed_model_count,
            latest_file_name.clone(),
            latest_file_name.clone(),
            bytes_transferred,
            Some(format!("正在下载模型 {}", model.file_name)),
            None,
            None,
        );

        let existing_model_path = replacement_target.as_ref().map(|script| {
            scripts_root
                .join(script.id.to_string())
                .join(normalized_type.file_name)
        });
        if let Some(existing_model_path) = existing_model_path.as_ref() {
            match model_hash_matches_path(
                model.hash_algorithm.as_deref(),
                model.hash_value.as_deref(),
                existing_model_path,
            ) {
                Ok(true) => {
                    if let Err(error) = std::fs::copy(
                        existing_model_path,
                        dir_swap.staging_dir.join(normalized_type.file_name),
                    ) {
                        transfer_run
                            .finish(
                                "error",
                                completed_model_count,
                                latest_file_name.clone(),
                                bytes_transferred,
                                Some("复用本地模型失败".to_string()),
                                Some(error.to_string()),
                            )
                            .await;
                        cleanup_script_dir_swap_temp(&dir_swap);
                        return ApiResponse::error(Some(format!(
                            "复制本地模型 {} 失败: {}",
                            existing_model_path.display(),
                            error
                        )));
                    }

                    completed_model_count += 1;
                    bytes_transferred += model.size_bytes.unwrap_or(0) as i64;
                    transfer_run.emit(
                        "running",
                        completed_model_count,
                        latest_file_name.clone(),
                        latest_file_name.clone(),
                        bytes_transferred,
                        Some(format!("模型 {} 已复用，无需下载", model.file_name)),
                        None,
                        None,
                    );
                    continue;
                }
                Ok(false) => {}
                Err(error) => {
                    transfer_run
                        .finish(
                            "error",
                            completed_model_count,
                            latest_file_name.clone(),
                            bytes_transferred,
                            Some("校验本地模型失败".to_string()),
                            Some(error.clone()),
                        )
                        .await;
                    cleanup_script_dir_swap_temp(&dir_swap);
                    return ApiResponse::error(Some(error));
                }
            }
        }

        let completed_bytes_before = bytes_transferred;
        let download_result = client
            .download_file_with_resume_progress(
                endpoint.as_str(),
                &dir_swap.staging_dir.join(normalized_type.file_name),
                verification_sha256(model.hash_algorithm.as_deref(), model.hash_value.as_deref()),
                |progress| {
                    let current_file_bytes = progress.transferred_bytes as i64;
                    transfer_run.emit(
                        "running",
                        completed_model_count,
                        latest_file_name.clone(),
                        latest_file_name.clone(),
                        completed_bytes_before + current_file_bytes,
                        Some(format!("正在下载模型 {}", model.file_name)),
                        None,
                        None,
                    );
                },
                || {
                    transfer_run.checkpoint_blocking(
                        completed_model_count,
                        latest_file_name.clone(),
                        latest_file_name.clone(),
                        completed_bytes_before,
                        format!("正在下载模型 {}", model.file_name),
                    )
                },
            )
            .await;

        if let Err(error) = download_result {
            if is_transfer_deleted_error(&error) {
                transfer_run.close();
                cleanup_script_dir_swap_temp(&dir_swap);
                return ApiResponse::error(Some(TRANSFER_DELETED_MESSAGE.to_string()));
            }
            let friendly_error = app_error_message(error);
            let error_message = format!("下载模型 {} 失败，请检查网络后重试。", model.file_name);
            transfer_run
                .finish(
                    "error",
                    completed_model_count,
                    latest_file_name.clone(),
                    bytes_transferred,
                    Some(error_message.clone()),
                    Some(friendly_error),
                )
                .await;
            cleanup_script_dir_swap_temp(&dir_swap);
            return ApiResponse::error(Some(error_message));
        }

        completed_model_count += 1;
        bytes_transferred = completed_bytes_before + model.size_bytes.unwrap_or(0) as i64;
        transfer_run.emit(
            "running",
            completed_model_count,
            latest_file_name.clone(),
            latest_file_name.clone(),
            bytes_transferred,
            Some(format!("模型 {} 下载完成", model.file_name)),
            None,
            None,
        );
    }

    if transfer_run.control.state() == ScriptTransferControlState::DeleteRequested {
        transfer_run.close();
        cleanup_script_dir_swap_temp(&dir_swap);
        return ApiResponse::error(Some(TRANSFER_DELETED_MESSAGE.to_string()));
    }

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            transfer_run
                .finish(
                    "error",
                    completed_model_count,
                    latest_file_name.clone(),
                    bytes_transferred,
                    Some("开启事务失败".to_string()),
                    Some(error.to_string()),
                )
                .await;
            cleanup_script_dir_swap_temp(&dir_swap);
            return ApiResponse::error(Some(format!("开启事务失败: {}", error)));
        }
    };

    if let Some(existing_script) = replacement_target.as_ref() {
        if let Err(error) =
            infra_sqlite::delete_script_graph_in_transaction(&mut tx, existing_script.id).await
        {
            transfer_run
                .finish(
                    "error",
                    completed_model_count,
                    latest_file_name.clone(),
                    bytes_transferred,
                    Some("删除本地旧副本失败".to_string()),
                    Some(error.clone()),
                )
                .await;
            cleanup_script_dir_swap_temp(&dir_swap);
            return ApiResponse::error(Some(error));
        }
    }

    if let Err(error) = infra_sqlite::batch_insert_script_related(
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
        transfer_run
            .finish(
                "error",
                completed_model_count,
                latest_file_name.clone(),
                bytes_transferred,
                Some("写入本地脚本失败".to_string()),
                Some(error.clone()),
            )
            .await;
        cleanup_script_dir_swap_temp(&dir_swap);
        return ApiResponse::error(Some(error));
    }

    if let Err(error) = activate_script_dir_swap(&mut dir_swap) {
        transfer_run
            .finish(
                "error",
                completed_model_count,
                latest_file_name.clone(),
                bytes_transferred,
                Some("切换脚本模型目录失败".to_string()),
                Some(error.clone()),
            )
            .await;
        cleanup_script_dir_swap_temp(&dir_swap);
        return ApiResponse::error(Some(error));
    }

    if let Err(error) = tx.commit().await {
        if let Err(rollback_error) = rollback_script_dir_swap(&mut dir_swap) {
            Log::error(&format!(
                "提交失败后恢复脚本模型目录失败: {}, final_dir={}",
                rollback_error,
                dir_swap.final_dir.display()
            ));
        }
        transfer_run
            .finish(
                "error",
                completed_model_count,
                latest_file_name.clone(),
                bytes_transferred,
                Some("提交事务失败".to_string()),
                Some(error.to_string()),
            )
            .await;
        return ApiResponse::error(Some(format!("提交事务失败: {}", error)));
    }

    cleanup_script_dir_swap_temp(&dir_swap);

    let success_message = match existing_local_ver_num {
        Some(existing_ver_num) if existing_ver_num == download_data.script.info.ver_num => {
            "相同版本已覆盖到本地库".to_string()
        }
        Some(_) => "云端脚本已更新到本地库".to_string(),
        None => "脚本下载并写入成功".to_string(),
    };
    if transfer_run.control.state() == ScriptTransferControlState::DeleteRequested {
        transfer_run.close();
        return ApiResponse::error(Some(TRANSFER_DELETED_MESSAGE.to_string()));
    }
    transfer_run
        .finish(
            "success",
            completed_model_count,
            latest_file_name,
            bytes_transferred,
            Some(success_message.clone()),
            None,
        )
        .await;

    ApiResponse::success(Some(local_script_id.to_string()), Some(success_message))
}

#[command]
pub async fn backend_upload_script(
    app_handle: AppHandle,
    script_id: String,
) -> ApiResponse<String> {
    use domain_script::{
        PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink,
        PolicySetProfile, ScriptTaskProfile,
    };
    let script_id_value = match uuid::Uuid::parse_str(&script_id) {
        Ok(id) => id.into(),
        Err(_) => return ApiResponse::error(Some("脚本 ID 格式无效".to_string())),
    };
    let mut script = match get_script(script_id_value).await {
        Ok(Some(script)) => script,
        Ok(None) => return ApiResponse::error(Some("脚本不存在".to_string())),
        Err(error) => return ApiResponse::error(Some(format!("读取本地脚本失败: {}", error))),
    };

    if script.info.script_type != ScriptType::Dev {
        return ApiResponse::error(Some("只有开发中 (Dev) 的脚本才能被上传".to_string()));
    }

    let runtime_type = match runtime_type_param(&script.info.runtime_type) {
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
    let summary_response: AppResult<BackendApiRes<serde_json::Value>> =
        client.get(&summary_url).await;
    let local_ver_num = Some(script.info.ver_num);
    let local_version_label =
        format_version_label(Some(script.info.ver_name.as_str()), local_ver_num);
    match summary_response {
        Ok(api_res) if api_res.code == 200 => {
            if let Some(summary) = api_res.data {
                let (remote_ver_name, remote_ver_num) = extract_cloud_summary_version(&summary);
                if let (Some(local_ver_num), Some(remote_ver_num)) = (local_ver_num, remote_ver_num)
                {
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
                Some(format_backend_message(
                    &api_res.message,
                    api_res.details.as_ref(),
                )),
                api_res.details,
            );
        }
        Err(error) => return ApiResponse::error(Some(app_error_message(error))),
    }

    let script_user_name = script.info.user_name.as_deref().unwrap_or("").trim();
    if auth_session.username == "Guest"
        || script_user_name.is_empty()
        || script_user_name == "Guest"
    {
        let author = match fetch_upload_author(&client).await {
            Ok(author) => author,
            Err(error) => return ApiResponse::error(Some(error)),
        };

        script.info.user_id = author.id;
        script.info.user_name = Some(author.username);

        if let Err(error) = save_script(&script).await {
            return ApiResponse::error(Some(format!("更新本地脚本作者信息失败: {}", error)));
        }
    }

    apply_current_client_capability(&mut script.info);
    if let Err(error) = save_script(&script).await {
        return ApiResponse::error(Some(format!("更新本地脚本兼容信息失败: {}", error)));
    }

    let published_script_id = script.id.to_string();
    rewrite_script_model_paths_for_published(&mut script, &published_script_id);

    let policies: Vec<PolicyProfile> = list_policies(script.id).await.unwrap_or_default();
    let policy_groups: Vec<PolicyGroupProfile> =
        list_policy_groups(script.id).await.unwrap_or_default();
    let policy_sets: Vec<PolicySetProfile> = list_policy_sets(script.id).await.unwrap_or_default();
    let tasks: Vec<ScriptTaskProfile> = list_script_tasks(script.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|task| !task.is_deleted)
        .collect();
    let group_policies: Vec<PolicyGroupPolicyLink> =
        list_group_policy_links(script.id).await.unwrap_or_default();
    let set_groups: Vec<PolicySetGroupLink> =
        list_set_group_links(script.id).await.unwrap_or_default();

    let script_version = script.info.ver_num;
    let remote_model_files =
        fetch_remote_model_file_index(&client, &script_id, &runtime_type).await;
    let transfer_total_bytes = model_uploads
        .iter()
        .map(|model| model.size_bytes as i64)
        .sum::<i64>();
    let transfer_model_count = model_uploads.len() as i64;
    let transfer_run = ScriptTransferRun::new(
        app_handle.clone(),
        "upload",
        Some(script_id.clone()),
        Some(script_id.clone()),
        Some(script.info.name.clone()),
        transfer_model_count,
        transfer_total_bytes,
    );
    transfer_run
        .start(Some(if transfer_model_count > 0 {
            "正在上传脚本数据".to_string()
        } else {
            "正在上传脚本".to_string()
        }))
        .await;

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

    if transfer_run.control.state() == ScriptTransferControlState::DeleteRequested {
        transfer_run.close();
        return ApiResponse::error(Some(TRANSFER_DELETED_MESSAGE.to_string()));
    }

    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                let mut completed_model_count = 0_i64;
                let mut bytes_transferred = 0_i64;
                let mut latest_file_name: Option<String> = None;

                for model in &model_uploads {
                    let upload_url = format!(
                        "/scripts/upload/model/{}/{}?runtime_type={}",
                        script_id, model.type_name, runtime_type
                    );
                    latest_file_name = Some(model.file_name.to_string());
                    if let Err(message) = transfer_run
                        .checkpoint_async(
                            completed_model_count,
                            latest_file_name.clone(),
                            latest_file_name.clone(),
                            bytes_transferred,
                            format!("正在上传模型 {}", model.file_name),
                        )
                        .await
                    {
                        transfer_run.close();
                        return ApiResponse::error(Some(message));
                    }
                    transfer_run.emit(
                        "running",
                        completed_model_count,
                        latest_file_name.clone(),
                        latest_file_name.clone(),
                        bytes_transferred,
                        Some(format!("正在上传模型 {}", model.file_name)),
                        None,
                        None,
                    );

                    if let Some(remote_model) = remote_model_files
                        .as_ref()
                        .and_then(|items| items.get(&model.type_name.to_ascii_lowercase()))
                    {
                        if model_hash_matches_sha256(
                            remote_model.hash_algorithm.as_deref(),
                            remote_model.hash_value.as_deref(),
                            &model.sha256,
                        ) {
                            completed_model_count += 1;
                            bytes_transferred += model.size_bytes as i64;
                            transfer_run.emit(
                                "running",
                                completed_model_count,
                                latest_file_name.clone(),
                                latest_file_name.clone(),
                                bytes_transferred,
                                Some(format!("模型 {} 已存在，无需上传", model.file_name)),
                                None,
                                None,
                            );
                            continue;
                        }
                    }

                    let completed_bytes_before = bytes_transferred;
                    let progress_emit_run = transfer_run.clone();
                    let progress_control_run = transfer_run.clone();
                    let progress_completed_model_count = completed_model_count;
                    let progress_emit_file_name = latest_file_name.clone();
                    let progress_control_file_name = latest_file_name.clone();
                    let progress_emit_model_name = model.file_name.to_string();
                    let progress_control_model_name = model.file_name.to_string();
                    let upload_res: AppResult<BackendApiRes<String>> = client
                        .upload_file_with_progress(
                            &upload_url,
                            &model.local_path,
                            "file",
                            model.file_name,
                            move |progress| {
                                progress_emit_run.emit(
                                    "running",
                                    progress_completed_model_count,
                                    progress_emit_file_name.clone(),
                                    progress_emit_file_name.clone(),
                                    completed_bytes_before + progress.transferred_bytes as i64,
                                    Some(format!("正在上传模型 {}", progress_emit_model_name)),
                                    None,
                                    None,
                                );
                            },
                            move || {
                                progress_control_run.checkpoint_blocking(
                                    progress_completed_model_count,
                                    progress_control_file_name.clone(),
                                    progress_control_file_name.clone(),
                                    completed_bytes_before,
                                    format!("正在上传模型 {}", progress_control_model_name),
                                )
                            },
                        )
                        .await;
                    match upload_res {
                        Ok(model_api_res) if model_api_res.code == 200 => {
                            completed_model_count += 1;
                            bytes_transferred = completed_bytes_before + model.size_bytes as i64;
                            transfer_run.emit(
                                "running",
                                completed_model_count,
                                latest_file_name.clone(),
                                latest_file_name.clone(),
                                bytes_transferred,
                                Some(format!("模型 {} 上传完成", model.file_name)),
                                None,
                                None,
                            );
                        }
                        Ok(model_api_res) => {
                            let error_message = format!(
                                "脚本已上传，但模型 {} 上传失败: {}",
                                model.file_name, model_api_res.message
                            );
                            transfer_run
                                .finish(
                                    "error",
                                    completed_model_count,
                                    latest_file_name.clone(),
                                    bytes_transferred,
                                    Some(error_message.clone()),
                                    Some(model_api_res.message),
                                )
                                .await;
                            return ApiResponse::error(Some(format!("{}", error_message)));
                        }
                        Err(error) => {
                            if is_transfer_deleted_error(&error) {
                                transfer_run.close();
                                return ApiResponse::error(Some(
                                    TRANSFER_DELETED_MESSAGE.to_string(),
                                ));
                            }
                            let friendly_error = app_error_message(error);
                            let error_message = format!(
                                "脚本已上传，但模型 {} 上传失败，请检查网络后重试。",
                                model.file_name
                            );
                            transfer_run
                                .finish(
                                    "error",
                                    completed_model_count,
                                    latest_file_name.clone(),
                                    bytes_transferred,
                                    Some(error_message.clone()),
                                    Some(friendly_error),
                                )
                                .await;
                            return ApiResponse::error(Some(format!("{}", error_message)));
                        }
                    }
                }
                let success_message = api_res.message;
                if transfer_run.control.state() == ScriptTransferControlState::DeleteRequested {
                    transfer_run.close();
                    return ApiResponse::error(Some(TRANSFER_DELETED_MESSAGE.to_string()));
                }
                transfer_run
                    .finish(
                        "success",
                        completed_model_count,
                        latest_file_name,
                        bytes_transferred,
                        Some(success_message.clone()),
                        None,
                    )
                    .await;
                ApiResponse::success(None, Some(success_message))
            } else {
                transfer_run
                    .finish(
                        "error",
                        0,
                        None,
                        0,
                        Some(format_backend_message(
                            &api_res.message,
                            api_res.details.as_ref(),
                        )),
                        Some(api_res.message.clone()),
                    )
                    .await;
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
        Err(error) => {
            let error_message = app_error_message(error);
            transfer_run
                .finish(
                    "error",
                    0,
                    None,
                    0,
                    Some(error_message.clone()),
                    Some(error_message.clone()),
                )
                .await;
            ApiResponse::error(Some(error_message))
        }
    }
}

async fn fetch_upload_author(client: &HttpClient) -> Result<UploadAuthor, String> {
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get("/user/profile").await;
    let api_res = res.map_err(app_error_message)?;

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
