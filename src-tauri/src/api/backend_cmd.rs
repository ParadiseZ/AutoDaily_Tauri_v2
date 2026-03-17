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
pub async fn backend_download_script(app_handle: AppHandle, script_id: String, current_user_id: Option<String>) -> ApiResponse<String> {
    use crate::infrastructure::db::get_pool;
    use crate::domain::scripts::script_info::{ScriptType};
    use crate::infrastructure::core::{ScriptId, PolicyId, PolicyGroupId, PolicySetId, TaskId, UserId};
    use std::str::FromStr;

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
        if let Ok(uid) = UserId::from_str(&uid_str) {
            download_data.script.data.user_id = uid;
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

    // Insert script
    if let Err(e) = sqlx::query("INSERT INTO scripts (id, `data`) VALUES (?, ?)")
        .bind(download_data.script.id.to_string())
        .bind(download_data.script.data)
        .execute(&mut *tx).await {
            return ApiResponse::error(Some(format!("写入 Script 表失败: {}", e)));
    }

    // Insert policies
    for policy in download_data.policies {
        if let Err(e) = sqlx::query("INSERT INTO policies (id, script_id, order_index, `data`) VALUES (?, ?, ?, ?)")
            .bind(policy.id.to_string()).bind(policy.script_id.to_string()).bind(policy.order_index).bind(policy.data)
            .execute(&mut *tx).await { return ApiResponse::error(Some(format!("写入 Policy 失败: {}", e))); }
    }

    // Insert groups
    for group in download_data.policy_groups {
        if let Err(e) = sqlx::query("INSERT INTO policy_groups (id, script_id, order_index, `data`) VALUES (?, ?, ?, ?)")
            .bind(group.id.to_string()).bind(group.script_id.to_string()).bind(group.order_index).bind(group.data)
            .execute(&mut *tx).await { return ApiResponse::error(Some(format!("写入 Group 失败: {}", e))); }
    }

    // Insert sets
    for set in download_data.policy_sets {
        if let Err(e) = sqlx::query("INSERT INTO policy_sets (id, script_id, order_index, `data`) VALUES (?, ?, ?, ?)")
            .bind(set.id.to_string()).bind(set.script_id.to_string()).bind(set.order_index).bind(set.data)
            .execute(&mut *tx).await { return ApiResponse::error(Some(format!("写入 Set 失败: {}", e))); }
    }

    // Insert relation group_policies
    for gp in download_data.group_policies {
        if let Err(e) = sqlx::query("INSERT INTO group_policies (group_id, policy_id, order_index) VALUES (?, ?, ?)")
            .bind(gp.group_id.to_string()).bind(gp.policy_id.to_string()).bind(gp.order_index)
            .execute(&mut *tx).await { return ApiResponse::error(Some(format!("写入 group_policies 失败: {}", e))); }
    }

    // Insert relation set_groups
    for sg in download_data.set_groups {
        if let Err(e) = sqlx::query("INSERT INTO set_groups (set_id, group_id, order_index) VALUES (?, ?, ?)")
            .bind(sg.set_id.to_string()).bind(sg.group_id.to_string()).bind(sg.order_index)
            .execute(&mut *tx).await { return ApiResponse::error(Some(format!("写入 set_groups 失败: {}", e))); }
    }

    // Insert tasks
    for task in download_data.tasks {
        if let Err(e) = sqlx::query("INSERT INTO script_tasks (id, script_id, name, is_hidden, nodes, edges, `data`) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(task.id.to_string()).bind(task.script_id.to_string()).bind(task.name).bind(task.is_hidden).bind(task.nodes).bind(task.edges).bind(task.data)
            .execute(&mut *tx).await { return ApiResponse::error(Some(format!("写入 Tasks 失败: {}", e))); }
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
        
    let script = match script {
        Some(s) => s,
        None => return ApiResponse::error(Some("脚本不存在".to_string())),
    };

    // 校验 ScriptType
    if script.data.script_type != ScriptType::Dev {
        return ApiResponse::error(Some("只有开发中 (Dev) 的脚本才能被上传".to_string()));
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

    let client = HttpClient::new(app_handle.clone());
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