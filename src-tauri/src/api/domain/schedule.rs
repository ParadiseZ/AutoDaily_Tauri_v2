// 调度管理 API — 供前端调用
use crate::constant::table_name::{ASSIGNMENT_TABLE, SCHEDULE_TABLE, TIME_TEMPLATE_TABLE};
use crate::domain::devices::device_schedule::DeviceScriptAssignment;
use crate::domain::schedule::time_template::TimeTemplate;
use crate::infrastructure::core::{DeviceId, ScheduleId, ScriptId};
use crate::infrastructure::db::get_pool;
use tauri::command;

// ========== 脚本分配（队列定义）==========

/// 获取指定设备的所有脚本分配（按 index 排序）
#[command]
pub async fn get_assignments_by_device_cmd(device_id: DeviceId) -> Result<Vec<DeviceScriptAssignment>, String> {
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
pub async fn save_assignment_cmd(assignment: DeviceScriptAssignment) -> Result<(), String> {
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
    Ok(())
}

/// 删除脚本分配
#[command]
pub async fn delete_assignment_cmd(assignment_id: ScheduleId) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query(&format!("DELETE FROM {} WHERE id = ?", ASSIGNMENT_TABLE))
        .bind(assignment_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 批量更新排序顺序
#[command]
pub async fn reorder_assignments_cmd(device_id: DeviceId, assignment_ids: Vec<ScheduleId>) -> Result<(), String> {
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
    Ok(())
}

// ========== 调度记录 ==========

/// 获取指定设备的调度记录
#[command]
pub async fn get_schedules_by_device_cmd(device_id: DeviceId) -> Result<Vec<crate::domain::devices::device_schedule::DeviceScriptSchedule>, String> {
    let pool = get_pool();
    let query = format!(
        "SELECT id, device_id, script_id, task_id, task_cycle, status, started_at, completed_at, message FROM {} WHERE device_id = ? ORDER BY started_at DESC",
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
    sqlx::query(&format!("DELETE FROM {} WHERE device_id = ?", SCHEDULE_TABLE))
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
    sqlx::query(&format!("DELETE FROM {} WHERE script_id = ?", SCHEDULE_TABLE))
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
    let query = format!("SELECT id, name, start_time, end_time FROM {}", TIME_TEMPLATE_TABLE);
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
pub async fn delete_time_template_cmd(template_id: String) -> Result<(), String> {
    let pool = get_pool();
    sqlx::query(&format!("DELETE FROM {} WHERE id = ?", TIME_TEMPLATE_TABLE))
        .bind(&template_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
