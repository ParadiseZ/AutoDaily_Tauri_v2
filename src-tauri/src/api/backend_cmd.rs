use crate::api::api_response::ApiResponse;
use crate::api::backend_dto::*;
use crate::app::app_error::AppResult;
use crate::constant::table_name::SCRIPT_TABLE;
use crate::infrastructure::http_client::HttpClient;
use crate::infrastructure::db::DbRepo;
use tauri::{command, AppHandle};
use crate::infrastructure::core::Serialize;
use crate::infrastructure::core::UserId;

#[derive(Debug)]
struct UploadAuthor {
    id: UserId,
    username: String,
}

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
pub async fn backend_reset_password(
    app_handle: AppHandle,
    req: ResetPasswordReq,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<String>> = client.post("/auth/reset-password", &req).await;
    trans_api_res(res)
}

#[command]
pub async fn backend_login(
    app_handle: AppHandle,
    req: LoginReq,
) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/auth/login", &req).await;
    trans_api_res_token(client,res)
}

#[command]
pub async fn backend_get_auth_session(app_handle: AppHandle) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    ApiResponse::success(client.get_auth_session(), None)
}

#[command]
pub async fn backend_logout(app_handle: AppHandle) -> ApiResponse<()> {
    let client = HttpClient::new(app_handle);
    let _ = client.clear_auth_session();
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
pub async fn backend_update_username(
    app_handle: AppHandle,
    req: UpdateUsernameReq,
) -> ApiResponse<AuthRes> {
    let client = HttpClient::new(app_handle);
    let res: AppResult<BackendApiRes<AuthRes>> = client.post("/user/username", &req).await;
    trans_api_res_token(client,res)
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
pub async fn backend_download_script(app_handle: AppHandle, script_id: String, current_user_id: Option<String>) -> ApiResponse<String> {
    use crate::infrastructure::db::get_pool;
    use crate::domain::scripts::script_info::ScriptType;
    use crate::infrastructure::core::{PolicyGroupId, PolicyId, PolicySetId, ScriptId, TaskId, UserId};
    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/download/{}", script_id);
    // 1. Fetch from backend
    let res: AppResult<BackendApiRes<ScriptUploadRequest>> = client.get(&url).await;
    
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
        Err(e) => return ApiResponse::error(Some(e.to_string())),
    };

    // 2. ID Regeneration mapping
    let old_script_id = download_data.script.id.clone();
    let new_script_id = ScriptId::new_v7();
    
    // Create maps to track old -> new IDs for foreign keys
    let mut policy_map: std::collections::HashMap<PolicyId, PolicyId> = std::collections::HashMap::new();
    let mut group_map: std::collections::HashMap<PolicyGroupId, PolicyGroupId> = std::collections::HashMap::new();
    let mut set_map: std::collections::HashMap<PolicySetId, PolicySetId> = std::collections::HashMap::new();
    
    // Update Script
    download_data.script.id = new_script_id.clone();
    download_data.script.data.cloud_id = Some(old_script_id);
    download_data.script.data.script_type = ScriptType::Published;
    // Replace user_id if importing (logged in local user)
    if let Some(uid_str) = current_user_id {
        if let Ok(uuid) = uuid::Uuid::parse_str(&uid_str) {
            download_data.script.data.user_id = UserId::from(uuid);
        }
    }

    // Regenerate and Update Policies
    for policy in download_data.policies.iter_mut() {
        let new_pid = PolicyId::new_v7();
        policy_map.insert(policy.id.clone(), new_pid.clone());
        policy.id = new_pid;
        policy.script_id = new_script_id.clone();
    }

    // Regenerate and Update Groups
    for group in download_data.policy_groups.iter_mut() {
        let new_gid = PolicyGroupId::new_v7();
        group_map.insert(group.id.clone(), new_gid.clone());
        group.id = new_gid;
        group.script_id = new_script_id.clone();
    }

    // Regenerate and Update Sets
    for set in download_data.policy_sets.iter_mut() {
        let new_sid = PolicySetId::new_v7();
        set_map.insert(set.id.clone(), new_sid.clone());
        set.id = new_sid;
        set.script_id = new_script_id.clone();
    }

    // Regenerate and Update Tasks
    for task in download_data.tasks.iter_mut() {
        let new_tid = TaskId::new_v7();
        task.id = new_tid;
        task.script_id = new_script_id.clone();
    }

    // Re-map relation tables
    for gp in download_data.group_policies.iter_mut() {
        if let Some(new_gid) = group_map.get(&gp.group_id) { gp.group_id = new_gid.clone(); }
        if let Some(new_pid) = policy_map.get(&gp.policy_id) { gp.policy_id = new_pid.clone(); }
    }

    for sg in download_data.set_groups.iter_mut() {
        if let Some(new_sid) = set_map.get(&sg.set_id) { sg.set_id = new_sid.clone(); }
        if let Some(new_gid) = group_map.get(&sg.group_id) { sg.group_id = new_gid.clone(); }
    }

    // 3. Save to local SQLite in Transaction
    let pool = get_pool();
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ApiResponse::error(Some(format!("开启事务失败: {}", e))),
    };

    // Batch insert all related data
    if let Err(e) = crate::api::domain::script_batch_insert::batch_insert_script_related(
        &mut tx,
        &download_data.script,
        &download_data.policies,
        &download_data.policy_groups,
        &download_data.policy_sets,
        &download_data.group_policies,
        &download_data.set_groups,
        &download_data.tasks,
    ).await {
        return ApiResponse::error(Some(e));
    }

    if let Err(e) = tx.commit().await {
        return ApiResponse::error(Some(format!("提交事务失败: {}", e)));
    }

    ApiResponse::success(Some(new_script_id.to_string()), Some("脚本下载并写入成功".to_string()))
}

#[command]
pub async fn backend_upload_script(
    app_handle: AppHandle,
    script_id: String,
) -> ApiResponse<String> {
    use crate::infrastructure::db::get_pool;
    use crate::domain::scripts::script_info::{ScriptTable, ScriptType};
    use crate::domain::scripts::policy::*;
    use crate::domain::scripts::script_task::ScriptTaskTable;
    
    let pool = get_pool();
    
    // 1. 获取主脚本
    let script: Option<ScriptTable> = sqlx::query_as("SELECT id, `data` FROM scripts WHERE id = ?")
        .bind(&script_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
        
    let mut script = match script {
        Some(s) => s,
        None => return ApiResponse::error(Some("脚本不存在".to_string())),
    };

    // 校验 ScriptType
    if script.data.script_type != ScriptType::Dev {
        return ApiResponse::error(Some("只有开发中 (Dev) 的脚本才能被上传".to_string()));
    }

    let client = HttpClient::new(app_handle.clone());
    let auth_session = match client.get_auth_session() {
        Some(session) => session,
        None => return ApiResponse::error(Some("上传前请先登录".to_string())),
    };

    if auth_session.username.trim().is_empty() {
        return ApiResponse::error(Some("当前登录态缺少用户名，请重新登录".to_string()));
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

    // 2. 收集关联数据
    let policies: Vec<PolicyTable> = sqlx::query_as("SELECT id, script_id, order_index, `data` FROM policies WHERE script_id = ?")
        .bind(&script_id).fetch_all(pool).await.unwrap_or_default();
        
    let policy_groups: Vec<PolicyGroupTable> = sqlx::query_as("SELECT id, script_id, order_index, `data` FROM policy_groups WHERE script_id = ?")
        .bind(&script_id).fetch_all(pool).await.unwrap_or_default();
        
    let policy_sets: Vec<PolicySetTable> = sqlx::query_as("SELECT id, script_id, order_index, `data` FROM policy_sets WHERE script_id = ?")
        .bind(&script_id).fetch_all(pool).await.unwrap_or_default();

    let tasks: Vec<ScriptTaskTable> = sqlx::query_as("SELECT * FROM script_tasks WHERE script_id = ? AND is_deleted = false")
        .bind(&script_id).fetch_all(pool).await.unwrap_or_default();

    // 关联表 (Group <-> Policy)
    let group_policies: Vec<GroupPolicyRelation> = sqlx::query_as(
        "SELECT gp.group_id, gp.policy_id, gp.order_index FROM group_policies gp 
         JOIN policy_groups g ON gp.group_id = g.id WHERE g.script_id = ?"
    ).bind(&script_id).fetch_all(pool).await.unwrap_or_default();

    // 关联表 (Set <-> Group)
    let set_groups: Vec<SetGroupRelation> = sqlx::query_as(
        "SELECT sg.set_id, sg.group_id, sg.order_index FROM set_groups sg 
         JOIN policy_sets s ON sg.set_id = s.id WHERE s.script_id = ?"
    ).bind(&script_id).fetch_all(pool).await.unwrap_or_default();

    let upload_req = ScriptUploadRequest {
        script,
        policies,
        tasks,
        policy_groups,
        policy_sets,
        group_policies,
        set_groups,
    };

    let res: AppResult<BackendApiRes<serde_json::Value>> = client.post("/scripts/upload", &upload_req).await;
    
    match res {
        Ok(api_res) => {
            if api_res.code == 200 {
                // 如果云端返回了 cloud_id 或是我们应当更新 local db，可以在这里做
                ApiResponse::success(None, Some(api_res.message))
            } else {
                ApiResponse::error(Some(api_res.message))
            }
        }
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

#[command]
pub async fn backend_upload_model(
    app_handle: AppHandle,
    script_id: String,
    model_type: String, // "det" or "rec"
    local_file_path: String,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/upload/model/{}/{}", script_id, model_type);
    
    let path = std::path::Path::new(&local_file_path);
    if !path.exists() {
        return ApiResponse::error(Some(format!("File {} does not exist locally", local_file_path)));
    }
    
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("model.onnx");

    let res: AppResult<BackendApiRes<String>> = client.upload_file(
        &url,
        path,
        "file", // must match Spring Boot RequestParam "file"
        file_name
    ).await;

    trans_api_res(res)
}

#[command]
pub async fn backend_download_model(
    app_handle: AppHandle,
    script_id: String,
    model_type: String, // "det" or "rec"
    save_dir: String,
) -> ApiResponse<String> {
    let client = HttpClient::new(app_handle);
    let url = format!("/scripts/download/model/{}/{}", script_id, model_type);
    
    // Construct local save path
    let file_name = format!("{}_model.onnx", model_type);
    let dir_path = std::path::Path::new(&save_dir);
    if !dir_path.exists() {
        if let Err(e) = std::fs::create_dir_all(dir_path) {
            return ApiResponse::error(Some(format!("Failed to create save directory: {}", e)));
        }
    }
    
    let target_path = dir_path.join(file_name);

    match client.download_file(&url, &target_path).await {
        Ok(_) => ApiResponse::success(
            Some(target_path.to_string_lossy().to_string()),
            Some("Model downloaded successfully".to_string())
        ),
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

fn trans_api_res<T: Serialize>(api_res: AppResult<BackendApiRes<T>>) -> ApiResponse<T> {
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

fn trans_api_res_token(client: HttpClient,api_res: AppResult<BackendApiRes<AuthRes>>) -> ApiResponse<AuthRes> {
    match api_res {
        Ok(api_res) => {
            if api_res.code == 200 {
                if let Some(auth_data) = &api_res.data {
                    let _ = client.set_auth_session(auth_data);
                }
                ApiResponse::success(api_res.data, Some(api_res.message))
            } else {
                ApiResponse::error(Some(api_res.message))
            }
        }
        Err(e) => ApiResponse::error(Some(e.to_string())),
    }
}

async fn fetch_upload_author(client: &HttpClient) -> Result<UploadAuthor, String> {
    let res: AppResult<BackendApiRes<serde_json::Value>> = client.get("/user/profile").await;
    let api_res = res.map_err(|e| e.to_string())?;

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

    let uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("用户 id 非法: {}", e))?;

    Ok(UploadAuthor {
        id: UserId::from(uuid),
        username: username.to_string(),
    })
}
