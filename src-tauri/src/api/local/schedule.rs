// 调度管理 API — 供前端调用
use crate::api::local::execution::{
    emit_assignment_schedule_changed, load_assigned_device_ids_by_time_template,
    load_runtime_queue_for_current_window, notify_auto_dispatch_reschedule, planner_queue_items,
};
use crate::infra::logging::log_trait::Log;
use ad_kernel::ids::{AccountId, AssignmentId, DeviceId, ScriptId, TemplateId};
use domain_schedule::TimeWindow;
use domain_schedule::{
    AssignmentProfile, AssignmentScheduleProfile, ExecutionScheduleProfile, TemplateValueProfile,
    TimeTemplateProfile,
};
use infra_sqlite::{
    clear_schedules_by_device, clear_schedules_by_script, delete_assignment, delete_template_value,
    delete_time_template, find_template_value_exact, list_assignments, list_execution_schedules,
    list_time_templates, save_template_value, save_time_template,
};
use infra_sqlite::{
    compact_assignment_indices, load_assignment_schedules_by_device, reorder_assignment_indices,
    sync_active_planner_schedule_order_indices, sync_active_planner_schedules_from_queue,
};
use tauri::command;

fn assignment_scope_label(assignment: &AssignmentProfile) -> String {
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

async fn load_device_assignments(device_id: DeviceId) -> Result<Vec<AssignmentProfile>, String> {
    list_assignments(device_id).await
}

// ========== 脚本分配（队列定义）==========

/// 获取指定设备的所有脚本分配（按 index 排序）
#[command]
pub async fn get_assignments_by_device_cmd(
    device_id: DeviceId,
) -> Result<Vec<AssignmentProfile>, String> {
    // 兼容历史脏数据：读取队列前先压缩重复/断裂索引，避免前端看到 5,5,5,6,7,8 这类序号。
    compact_assignment_indices(device_id).await?;
    load_device_assignments(device_id).await
}

/// 保存（新增或更新）脚本分配
#[command]
pub async fn save_assignment_cmd(
    app_handle: tauri::AppHandle,
    assignment: AssignmentProfile,
) -> Result<(), String> {
    let scope = assignment_scope_label(&assignment);
    let device_id = assignment.device_id;
    let account_data_json =
        serde_json::to_string(&assignment.account_data).map_err(|error| error.to_string())?;
    infra_sqlite::save_assignment(
        assignment.id,
        assignment.device_id,
        assignment.script_id,
        assignment.time_template_id,
        &account_data_json,
        assignment.index,
    )
    .await
    .map_err(|error| {
        let error = error.to_string();
        Log::error(&format!(
            "[调度] 保存队列分配失败 {}，错误={}",
            scope, error
        ));
        error
    })?;
    let compacted_ids = compact_assignment_indices(device_id)
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配后压缩索引失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
    let current_window_queue = load_runtime_queue_for_current_window(device_id)
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配后重载当前时间窗口队列失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
    let planner_queue = planner_queue_items(&current_window_queue);
    let synced = sync_active_planner_schedules_from_queue(
        device_id,
        &planner_queue,
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
    let updated = sync_active_planner_schedule_order_indices(device_id, compacted_ids.as_slice())
        .await
        .map_err(|error| {
            Log::error(&format!(
                "[调度] 保存队列分配后同步索引失败 {}，错误={}",
                scope, error
            ));
            error
        })?;
    if synced > 0 || updated > 0 {
        emit_assignment_schedule_changed(&app_handle, device_id);
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
    let device_id = delete_assignment(assignment_id).await?;

    if let Some(device_id) = device_id {
        let compacted_ids = compact_assignment_indices(device_id).await?;
        let current_window_queue = load_runtime_queue_for_current_window(device_id).await?;
        let planner_queue = planner_queue_items(&current_window_queue);
        let synced = sync_active_planner_schedules_from_queue(
            device_id,
            &planner_queue,
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
    reorder_assignment_indices(&assignment_ids).await?;
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
) -> Result<Vec<ExecutionScheduleProfile>, String> {
    list_execution_schedules(device_id).await
}

/// 获取指定设备的 assignment dispatch 账本
#[command]
pub async fn get_assignment_schedules_by_device_cmd(
    device_id: DeviceId,
) -> Result<Vec<AssignmentScheduleProfile>, String> {
    load_assignment_schedules_by_device(device_id).await
}

/// 清除指定设备的所有调度记录
#[command]
pub async fn clear_schedules_cmd(device_id: DeviceId) -> Result<(), String> {
    clear_schedules_by_device(device_id).await
}

/// 清除指定脚本的所有调度记录
#[command]
pub async fn clear_schedules_by_script_cmd(script_id: ScriptId) -> Result<(), String> {
    clear_schedules_by_script(script_id).await
}

// ========== 时间模板 ==========

/// 获取所有时间模板
#[command]
pub async fn get_all_time_templates_cmd() -> Result<Vec<TimeTemplateProfile>, String> {
    list_time_templates().await
}

/// 保存（新增或更新）时间模板
#[command]
pub async fn save_time_template_cmd(
    _app_handle: tauri::AppHandle,
    template: TimeTemplateProfile,
) -> Result<(), String> {
    TimeWindow::parse(template.start_time.as_deref(), template.end_time.as_deref())
        .map_err(|error| error.to_string())?;
    save_time_template(&template).await?;
    notify_auto_dispatch_reschedule();
    Ok(())
}

/// 删除时间模板
#[command]
pub async fn delete_time_template_cmd(
    _app_handle: tauri::AppHandle,
    template_id: String,
) -> Result<(), String> {
    let _affected_device_ids = load_assigned_device_ids_by_time_template(&template_id).await?;
    let template_id =
        TemplateId::from(uuid::Uuid::parse_str(&template_id).map_err(|error| error.to_string())?);
    delete_time_template(template_id).await?;
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
) -> Result<Option<TemplateValueProfile>, String> {
    let account_id = normalize_account_id(account_id);
    find_template_value_exact(device_id, script_id, time_template_id, account_id).await
}

/// 查询某设备某脚本在某时间模板和账号下的覆盖值
#[command]
pub async fn get_script_time_template_values_cmd(
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<Option<TemplateValueProfile>, String> {
    find_script_time_template_values_exact(Some(device_id), script_id, time_template_id, account_id)
        .await
}

/// 保存（新增或更新）脚本时间模板变量值
#[command]
pub async fn save_script_time_template_values_cmd(
    _app_handle: tauri::AppHandle,
    mut record: TemplateValueProfile,
) -> Result<(), String> {
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

    if let Some(existing_record) = existing {
        record.id = existing_record.id;
        record.created_at = existing_record.created_at;
    }
    record.updated_at = now;
    save_template_value(&record).await?;

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
    let account_id = normalize_account_id(account_id);
    delete_template_value(device_id, script_id, time_template_id, account_id).await
}
