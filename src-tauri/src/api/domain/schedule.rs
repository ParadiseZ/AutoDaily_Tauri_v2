// 调度管理 API — 供前端调用
use crate::api::infrastructure::runtime_sync::{
    load_assigned_device_ids_by_time_template, sync_device_session_if_online,
    sync_device_sessions_if_online,
};
use crate::constant::table_name::{
    ASSIGNMENT_TABLE, DEVICE_TABLE, RECOVERY_CHECKPOINT_TABLE, SCHEDULE_TABLE, SCRIPT_TABLE,
    SCRIPT_TIME_TEMPLATE_VALUES_TABLE, TIME_TEMPLATE_TABLE,
};
use crate::domain::devices::device_conf::{DevicePlatform, DeviceTable};
use crate::domain::devices::device_schedule::DeviceScriptAssignment;
use crate::domain::schedule::recovery_checkpoint::RecoveryCheckpointRow;
use crate::domain::schedule::script_time_template_values::ScriptTimeTemplateValuesDto;
use crate::domain::schedule::time_template::TimeTemplate;
use crate::domain::scripts::script_info::{ScriptPlatform, ScriptTable};
use crate::infrastructure::core::{AccountId, DeviceId, ScheduleId, ScriptId, TemplateId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::ipc::message::ResumeCheckpoint;
use tauri::command;

fn device_platform_label(platform: &DevicePlatform) -> &'static str {
    match platform {
        DevicePlatform::Android => "android",
        DevicePlatform::Desktop => "desktop",
    }
}

fn script_platform_label(platform: &ScriptPlatform) -> &'static str {
    match platform {
        ScriptPlatform::Android => "android",
        ScriptPlatform::Desktop => "desktop",
    }
}

fn platform_matches(device_platform: &DevicePlatform, script_platform: &ScriptPlatform) -> bool {
    matches!(
        (device_platform, script_platform),
        (DevicePlatform::Android, ScriptPlatform::Android)
            | (DevicePlatform::Desktop, ScriptPlatform::Desktop)
    )
}

fn is_checkpoint_expired(checkpoint: &ResumeCheckpoint) -> bool {
    let Ok(updated_at) = chrono::DateTime::parse_from_rfc3339(&checkpoint.updated_at) else {
        return true;
    };

    chrono::Utc::now() - updated_at.with_timezone(&chrono::Utc) > chrono::Duration::days(1)
}

async fn validate_assignment_platform(
    device_id: DeviceId,
    script_id: ScriptId,
) -> Result<(), String> {
    let device = DbRepo::get_by_id::<DeviceTable>(DEVICE_TABLE, &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备不存在: {}", device_id.to_string()))?;
    let script = DbRepo::get_by_id::<ScriptTable>(SCRIPT_TABLE, &script_id.to_string())
        .await?
        .ok_or_else(|| format!("脚本不存在: {}", script_id.to_string()))?;

    if platform_matches(&device.data.0.platform, &script.data.0.platform) {
        return Ok(());
    }

    Err(format!(
        "脚本平台不匹配，设备平台={}, 脚本平台={}",
        device_platform_label(&device.data.0.platform),
        script_platform_label(&script.data.0.platform),
    ))
}

// ========== 脚本分配（队列定义）==========

/// 获取指定设备的所有脚本分配（按 index 排序）
#[command]
pub async fn get_assignments_by_device_cmd(
    device_id: DeviceId,
) -> Result<Vec<DeviceScriptAssignment>, String> {
    let pool = get_pool();
    let query = format!(
        "SELECT id, device_id, script_id, time_template_id, account_data, `index` FROM {} WHERE device_id = ? ORDER BY `index` ASC",
        ASSIGNMENT_TABLE
    );
    sqlx::query_as::<_, DeviceScriptAssignment>(&query)
        .bind(device_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

/// 保存（新增或更新）脚本分配
#[command]
pub async fn save_assignment_cmd(
    app_handle: tauri::AppHandle,
    assignment: DeviceScriptAssignment,
) -> Result<(), String> {
    validate_assignment_platform(assignment.device_id, assignment.script_id).await?;
    let pool = get_pool();
    sqlx::query(&format!(
        "INSERT INTO {} (id, device_id, script_id, time_template_id, account_data, `index`) VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET script_id = excluded.script_id, time_template_id = excluded.time_template_id, account_data = excluded.account_data, `index` = excluded.`index`",
        ASSIGNMENT_TABLE
    ))
    .bind(assignment.id.to_string())
    .bind(assignment.device_id.to_string())
    .bind(assignment.script_id.to_string())
    .bind(assignment.time_template_id.map(|t| t.to_string()))
    .bind(&assignment.account_data)
    .bind(assignment.index)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sync_device_session_if_online(&app_handle, assignment.device_id).await?;
    Ok(())
}

/// 删除脚本分配
#[command]
pub async fn delete_assignment_cmd(
    app_handle: tauri::AppHandle,
    assignment_id: ScheduleId,
) -> Result<(), String> {
    let pool = get_pool();
    let device_id = sqlx::query_scalar::<_, String>(&format!(
        "SELECT device_id FROM {} WHERE id = ?",
        ASSIGNMENT_TABLE
    ))
    .bind(assignment_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(&format!("DELETE FROM {} WHERE id = ?", ASSIGNMENT_TABLE))
        .bind(assignment_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(device_id) = device_id {
        let parsed = uuid::Uuid::parse_str(&device_id).map_err(|e| e.to_string())?;
        sync_device_session_if_online(&app_handle, DeviceId::from(parsed)).await?;
    }
    Ok(())
}

/// 批量更新排序顺序
#[command]
pub async fn reorder_assignments_cmd(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    assignment_ids: Vec<ScheduleId>,
) -> Result<(), String> {
    let pool = get_pool();
    for (idx, id) in assignment_ids.iter().enumerate() {
        sqlx::query(&format!(
            "UPDATE {} SET `index` = ? WHERE id = ? AND device_id = ?",
            ASSIGNMENT_TABLE
        ))
        .bind(idx as u32)
        .bind(id.to_string())
        .bind(device_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }
    sync_device_session_if_online(&app_handle, device_id).await?;
    Ok(())
}

// ========== 调度记录 ==========

/// 获取指定设备的调度记录
#[command]
pub async fn get_schedules_by_device_cmd(
    device_id: DeviceId,
) -> Result<Vec<crate::domain::devices::device_schedule::DeviceScriptSchedule>, String> {
    let pool = get_pool();
    let query = format!(
        "SELECT id, device_id, execution_id, assignment_id, script_id, task_id, task_cycle, status, started_at, completed_at, message FROM {} WHERE device_id = ? ORDER BY started_at DESC",
        SCHEDULE_TABLE
    );
    sqlx::query_as(&query)
        .bind(device_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

/// 清除指定设备的所有调度记录
#[command]
pub async fn clear_schedules_cmd(device_id: DeviceId) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query(&format!(
        "DELETE FROM {} WHERE device_id = ?",
        SCHEDULE_TABLE
    ))
    .bind(device_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(&format!(
        "DELETE FROM {} WHERE device_id = ?",
        RECOVERY_CHECKPOINT_TABLE
    ))
    .bind(device_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 清除指定脚本的所有调度记录
#[command]
pub async fn clear_schedules_by_script_cmd(script_id: ScriptId) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query(&format!(
        "DELETE FROM {} WHERE script_id = ?",
        SCHEDULE_TABLE
    ))
    .bind(script_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(&format!(
        "DELETE FROM {} WHERE script_id = ?",
        RECOVERY_CHECKPOINT_TABLE
    ))
    .bind(script_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取指定设备最近一次可恢复检查点
#[command]
pub async fn get_recovery_checkpoint_by_device_cmd(
    device_id: DeviceId,
) -> Result<Option<ResumeCheckpoint>, String> {
    let pool = get_pool();
    let query = format!(
        "SELECT execution_id, source_session_id, device_id, run_target_json, assignment_id, script_id, time_template_id, account_id, task_id, step_id, resume_mode, definition_fingerprint, updated_at
         FROM {}
         WHERE device_id = ?
         ORDER BY updated_at DESC
         LIMIT 1",
        RECOVERY_CHECKPOINT_TABLE
    );
    let checkpoint = sqlx::query_as::<_, RecoveryCheckpointRow>(&query)
        .bind(device_id.to_string())
        .fetch_optional(pool)
        .await
        .map(|row| row.map(RecoveryCheckpointRow::into_checkpoint))
        .map_err(|e| e.to_string())?;

    if checkpoint.as_ref().is_some_and(is_checkpoint_expired) {
        sqlx::query(&format!(
            "DELETE FROM {} WHERE device_id = ?",
            RECOVERY_CHECKPOINT_TABLE
        ))
        .bind(device_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        return Ok(None);
    }

    Ok(checkpoint)
}

// ========== 时间模板 ==========

/// 获取所有时间模板
#[command]
pub async fn get_all_time_templates_cmd() -> Result<Vec<TimeTemplate>, String> {
    let pool = get_pool();
    let query = format!(
        "SELECT id, name, start_time, end_time FROM {}",
        TIME_TEMPLATE_TABLE
    );
    sqlx::query_as::<_, TimeTemplate>(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

/// 保存（新增或更新）时间模板
#[command]
pub async fn save_time_template_cmd(template: TimeTemplate) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query(&format!(
        "INSERT INTO {} (id, name, start_time, end_time) VALUES (?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, start_time = excluded.start_time, end_time = excluded.end_time",
        TIME_TEMPLATE_TABLE
    ))
    .bind(template.id.to_string())
    .bind(&template.name)
    .bind(&template.start_time)
    .bind(&template.end_time)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 删除时间模板
#[command]
pub async fn delete_time_template_cmd(
    app_handle: tauri::AppHandle,
    template_id: String,
) -> Result<(), String> {
    let pool = get_pool();
    let affected_device_ids = load_assigned_device_ids_by_time_template(&template_id).await?;
    sqlx::query(&format!("DELETE FROM {} WHERE id = ?", TIME_TEMPLATE_TABLE))
        .bind(&template_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    sync_device_sessions_if_online(&app_handle, affected_device_ids).await
}

// ========== 脚本时间模板变量值 ==========

fn normalize_account_id(account_id: Option<AccountId>) -> Option<AccountId> {
    account_id.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

async fn find_script_time_template_values_exact(
    device_id: Option<DeviceId>,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<Option<ScriptTimeTemplateValuesDto>, String> {
    let pool = get_pool();
    let device_id = device_id.map(|value| value.to_string());
    let account_id = normalize_account_id(account_id);
    let query = format!(
        "SELECT id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at
         FROM {}
         WHERE ((device_id IS NULL AND ?1 IS NULL) OR device_id = ?1)
           AND script_id = ?2
           AND time_template_id = ?3
           AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4)
         LIMIT 1",
        SCRIPT_TIME_TEMPLATE_VALUES_TABLE
    );

    sqlx::query_as::<_, ScriptTimeTemplateValuesDto>(&query)
        .bind(device_id)
        .bind(script_id.to_string())
        .bind(time_template_id.to_string())
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())
}

/// 查询某设备某脚本在某时间模板和账号下的覆盖值
#[command]
pub async fn get_script_time_template_values_cmd(
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<Option<ScriptTimeTemplateValuesDto>, String> {
    find_script_time_template_values_exact(Some(device_id), script_id, time_template_id, account_id)
        .await
}

/// 保存（新增或更新）脚本时间模板变量值
#[command]
pub async fn save_script_time_template_values_cmd(
    app_handle: tauri::AppHandle,
    mut record: ScriptTimeTemplateValuesDto,
) -> Result<(), String> {
    let pool = get_pool();
    if record.device_id.is_none() {
        return Err("device_id 不能为空".to_string());
    }

    record.account_id = normalize_account_id(record.account_id);
    let now = chrono::Local::now().to_rfc3339();

    let existing = find_script_time_template_values_exact(
        record.device_id,
        record.script_id,
        record.time_template_id,
        record.account_id.clone(),
    )
    .await?;

    match existing {
        Some(existing_record) => {
            sqlx::query(&format!(
                "UPDATE {} SET values_json = ?, updated_at = ? WHERE id = ?",
                SCRIPT_TIME_TEMPLATE_VALUES_TABLE
            ))
            .bind(&record.values_json)
            .bind(&now)
            .bind(existing_record.id.to_string())
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
        None => {
            if record.created_at.is_empty() {
                record.created_at = now.clone();
            }
            record.updated_at = now.clone();
            sqlx::query(&format!(
                "INSERT INTO {} (id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                SCRIPT_TIME_TEMPLATE_VALUES_TABLE
            ))
            .bind(record.id.to_string())
            .bind(record.device_id.map(|value| value.to_string()))
            .bind(record.script_id.to_string())
            .bind(record.time_template_id.to_string())
            .bind(record.account_id)
            .bind(&record.values_json)
            .bind(&record.created_at)
            .bind(&record.updated_at)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }
    }

    sync_device_session_if_online(
        &app_handle,
        record
            .device_id
            .ok_or_else(|| "device_id 不能为空".to_string())?,
    )
    .await
}

/// 删除某设备某脚本在某时间模板和账号下的覆盖值
#[command]
pub async fn delete_script_time_template_values_cmd(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<(), String> {
    let pool = get_pool();
    let account_id = normalize_account_id(account_id);
    sqlx::query(&format!(
        "DELETE FROM {}
         WHERE device_id = ?
           AND script_id = ?
           AND time_template_id = ?
           AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4)",
        SCRIPT_TIME_TEMPLATE_VALUES_TABLE
    ))
    .bind(device_id.to_string())
    .bind(script_id.to_string())
    .bind(time_template_id.to_string())
    .bind(account_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sync_device_session_if_online(&app_handle, device_id).await
}
