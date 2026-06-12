// 调度管理 API — 供前端调用
use crate::api::infrastructure::process_api::{
    emit_assignment_schedule_changed, load_assigned_device_ids_by_time_template,
    load_assignment_schedules_by_device, load_runtime_queue_for_current_window,
    notify_auto_dispatch_reschedule,
    sync_active_planner_schedule_order_indices, sync_active_planner_schedules_from_queue,
};
use crate::constant::table_name::{
    ASSIGNMENT_SCHEDULE_TABLE, ASSIGNMENT_TABLE, DEVICE_TABLE, SCHEDULE_TABLE, SCRIPT_TABLE,
    SCRIPT_TIME_TEMPLATE_VALUES_TABLE, TIME_TEMPLATE_TABLE,
};
use crate::domain::devices::device_conf::{DevicePlatform, DeviceTable};
use crate::domain::devices::device_schedule::{AssignmentSchedule, DeviceScriptAssignment};
use crate::domain::schedule::script_time_template_values::ScriptTimeTemplateValuesDto;
use crate::domain::schedule::time_template::TimeTemplate;
use crate::domain::scripts::script_info::{ScriptPlatform, ScriptTable};
use crate::infrastructure::core::{AccountId, AssignmentId, DeviceId, ScriptId, TemplateId};
use crate::infrastructure::db::{get_pool, DbRepo};
use crate::infrastructure::logging::log_trait::Log;
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

async fn validate_assignment_time_template(
    assignment: &DeviceScriptAssignment,
) -> Result<(), String> {
    let existing_assignment = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT 1 FROM {} WHERE id = ? LIMIT 1",
        ASSIGNMENT_TABLE
    ))
    .bind(assignment.id.to_string())
    .fetch_optional(get_pool())
    .await
    .map_err(|e| e.to_string())?
    .is_some();

    let Some(time_template_id) = assignment.time_template_id else {
        if !existing_assignment {
            return Err("追加队列任务必须选择真实时间模板。".to_string());
        }
        return Ok(());
    };

    let exists = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT 1 FROM {} WHERE id = ? LIMIT 1",
        TIME_TEMPLATE_TABLE
    ))
    .bind(time_template_id.to_string())
    .fetch_optional(get_pool())
    .await
    .map_err(|e| e.to_string())?
    .is_some();

    if exists {
        Ok(())
    } else {
        Err("所选时间模板不存在或已失效，请重新选择。".to_string())
    }
}

fn assignment_scope_label(assignment: &DeviceScriptAssignment) -> String {
    format!(
        "assignment={} device={} script={} template={}",
        assignment.id,
        assignment.device_id,
        assignment.script_id,
        assignment
            .time_template_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    )
}

async fn assignment_exists(assignment_id: AssignmentId) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>(&format!(
        "SELECT 1 FROM {} WHERE id = ? LIMIT 1",
        ASSIGNMENT_TABLE
    ))
    .bind(assignment_id.to_string())
    .fetch_optional(get_pool())
    .await
    .map_err(|e| e.to_string())
    .map(|value| value.is_some())
}

async fn next_assignment_index(device_id: DeviceId) -> Result<u32, String> {
    let next = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COALESCE(MAX(`index`), -1) + 1 FROM {} WHERE device_id = ?",
        ASSIGNMENT_TABLE
    ))
    .bind(device_id.to_string())
    .fetch_one(get_pool())
    .await
    .map_err(|e| e.to_string())?;
    Ok(next.max(0) as u32)
}

async fn compact_assignment_indices(device_id: DeviceId) -> Result<Vec<AssignmentId>, String> {
    let rows = sqlx::query_as::<_, DeviceScriptAssignment>(&format!(
        "SELECT id, device_id, script_id, time_template_id, account_data, `index`
         FROM {}
         WHERE device_id = ?
         ORDER BY `index` ASC, id ASC",
        ASSIGNMENT_TABLE
    ))
    .bind(device_id.to_string())
    .fetch_all(get_pool())
    .await
    .map_err(|e| e.to_string())?;

    let mut ordered_ids = Vec::with_capacity(rows.len());
    for (next_index, row) in rows.into_iter().enumerate() {
        ordered_ids.push(row.id);
        let next_index = next_index as u32;
        if row.index == next_index {
            continue;
        }
        sqlx::query(&format!(
            "UPDATE {} SET `index` = ? WHERE id = ?",
            ASSIGNMENT_TABLE
        ))
        .bind(next_index)
        .bind(row.id.to_string())
        .execute(get_pool())
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(ordered_ids)
}

// ========== 脚本分配（队列定义）==========

/// 获取指定设备的所有脚本分配（按 index 排序）
#[command]
pub async fn get_assignments_by_device_cmd(
    device_id: DeviceId,
) -> Result<Vec<DeviceScriptAssignment>, String> {
    // 兼容历史脏数据：读取队列前先压缩重复/断裂索引，避免前端看到 5,5,5,6,7,8 这类序号。
    compact_assignment_indices(device_id).await?;
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
    let scope = assignment_scope_label(&assignment);
    let is_existing = assignment_exists(assignment.id).await?;
    let next_index = if is_existing {
        assignment.index
    } else {
        next_assignment_index(assignment.device_id).await?
    };

    validate_assignment_platform(assignment.device_id, assignment.script_id)
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配时平台校验失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
    validate_assignment_time_template(&assignment)
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配时时间模板校验失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
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
    .bind(next_index)
    .execute(pool)
    .await
    .map_err(|error| {
        let error = error.to_string();
        Log::error(&format!(
            "[调度] 保存队列分配时写入数据库失败 {}，错误={}",
            scope, error
        ));
        error
    })?;
    let compacted_ids = compact_assignment_indices(assignment.device_id)
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配后压缩索引失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
    let current_window_queue = load_runtime_queue_for_current_window(assignment.device_id)
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配后重载当前时间窗口队列失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
    let synced = sync_active_planner_schedules_from_queue(
        assignment.device_id,
        current_window_queue.as_slice(),
        "队列定义变更，已同步当前批次",
    )
    .await
    .map_err(|error| {
        Log::error(&format!(
            "[调度] 保存队列分配后同步当前批次调度账本失败 {}，错误={}",
            scope, error
        ));
        error
    })?;
    let updated = sync_active_planner_schedule_order_indices(
        assignment.device_id,
        compacted_ids.as_slice(),
    )
    .await
    .map_err(|error| {
        Log::error(&format!(
            "[调度] 保存队列分配后同步索引失败 {}，错误={}",
            scope, error
        ));
        error
    })?;
    if synced > 0 || updated > 0 {
        emit_assignment_schedule_changed(&app_handle, assignment.device_id);
    }
    notify_auto_dispatch_reschedule();
    Ok(())
}

/// 删除脚本分配
#[command]
pub async fn delete_assignment_cmd(
    app_handle: tauri::AppHandle,
    assignment_id: AssignmentId,
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
        let device_id =
            DeviceId::from(uuid::Uuid::parse_str(&device_id).map_err(|e| e.to_string())?);
        let compacted_ids = compact_assignment_indices(device_id).await?;
        let current_window_queue = load_runtime_queue_for_current_window(device_id).await?;
        let synced = sync_active_planner_schedules_from_queue(
            device_id,
            current_window_queue.as_slice(),
            "队列定义变更，已同步当前批次",
        )
        .await?;
        let updated =
            sync_active_planner_schedule_order_indices(device_id, compacted_ids.as_slice()).await?;
        if synced > 0 || updated > 0 {
            emit_assignment_schedule_changed(&app_handle, device_id);
        }
        notify_auto_dispatch_reschedule();
    }
    Ok(())
}

/// 批量更新排序顺序
#[command]
pub async fn reorder_assignments_cmd(
    app_handle: tauri::AppHandle,
    device_id: DeviceId,
    assignment_ids: Vec<AssignmentId>,
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
    let updated =
        sync_active_planner_schedule_order_indices(device_id, assignment_ids.as_slice()).await?;
    if updated > 0 {
        emit_assignment_schedule_changed(&app_handle, device_id);
    }
    notify_auto_dispatch_reschedule();
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
        "SELECT id, device_id, execution_id, assignment_id, script_id, task_id, dedup_scope_hash, task_cycle, status, started_at, completed_at, message FROM {} WHERE device_id = ? ORDER BY started_at DESC",
        SCHEDULE_TABLE
    );
    sqlx::query_as(&query)
        .bind(device_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
}

/// 获取指定设备的 assignment dispatch 账本
#[command]
pub async fn get_assignment_schedules_by_device_cmd(
    device_id: DeviceId,
) -> Result<Vec<AssignmentSchedule>, String> {
    load_assignment_schedules_by_device(device_id).await
}

/// 清除指定设备的所有调度记录
#[command]
pub async fn clear_schedules_cmd(device_id: DeviceId) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query(&format!(
        "DELETE FROM {} WHERE device_id = ?",
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(device_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(&format!(
        "DELETE FROM {} WHERE device_id = ?",
        SCHEDULE_TABLE
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
        ASSIGNMENT_SCHEDULE_TABLE
    ))
    .bind(script_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(&format!(
        "DELETE FROM {} WHERE script_id = ?",
        SCHEDULE_TABLE
    ))
    .bind(script_id.to_string())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
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
pub async fn save_time_template_cmd(
    _app_handle: tauri::AppHandle,
    template: TimeTemplate,
) -> Result<(), String> {
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
    notify_auto_dispatch_reschedule();
    Ok(())
}

/// 删除时间模板
#[command]
pub async fn delete_time_template_cmd(
    _app_handle: tauri::AppHandle,
    template_id: String,
) -> Result<(), String> {
    let pool = get_pool();
    let _affected_device_ids = load_assigned_device_ids_by_time_template(&template_id).await?;
    sqlx::query(&format!("DELETE FROM {} WHERE id = ?", TIME_TEMPLATE_TABLE))
        .bind(&template_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    notify_auto_dispatch_reschedule();
    Ok(())
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
    _app_handle: tauri::AppHandle,
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

    Ok(())
}

/// 删除某设备某脚本在某时间模板和账号下的覆盖值
#[command]
pub async fn delete_script_time_template_values_cmd(
    _app_handle: tauri::AppHandle,
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
    Ok(())
}
