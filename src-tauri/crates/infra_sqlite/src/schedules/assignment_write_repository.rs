use crate::bootstrap::get_pool;
use ad_kernel::ids::{AssignmentId, DeviceId, ScriptId, TemplateId};

const ASSIGNMENT_TABLE: &str = "device_script_assignments";

pub async fn save_assignment(
    id: AssignmentId,
    device_id: DeviceId,
    script_id: ScriptId,
    template_id: Option<TemplateId>,
    account_data_json: &str,
    requested_index: u32,
) -> Result<(), String> {
    validate_platform(device_id, script_id).await?;
    let exists = assignment_exists(id).await?;
    validate_template(exists, template_id).await?;
    let index = if exists {
        requested_index
    } else {
        next_assignment_index(device_id).await?
    };

    sqlx::query(&format!(
        "INSERT INTO {ASSIGNMENT_TABLE} (id, device_id, script_id, time_template_id, account_data, `index`) VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET script_id = excluded.script_id, time_template_id = excluded.time_template_id, account_data = excluded.account_data, `index` = excluded.`index`"
    ))
    .bind(id.to_string())
    .bind(device_id.to_string())
    .bind(script_id.to_string())
    .bind(template_id.map(|id| id.to_string()))
    .bind(account_data_json)
    .bind(index)
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn assignment_exists(id: AssignmentId) -> Result<bool, String> {
    sqlx::query_scalar::<_, i64>(&format!(
        "SELECT 1 FROM {ASSIGNMENT_TABLE} WHERE id = ? LIMIT 1"
    ))
    .bind(id.to_string())
    .fetch_optional(get_pool())
    .await
    .map_err(|error| error.to_string())
    .map(|value| value.is_some())
}

async fn next_assignment_index(device_id: DeviceId) -> Result<u32, String> {
    let index = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COALESCE(MAX(`index`), -1) + 1 FROM {ASSIGNMENT_TABLE} WHERE device_id = ?"
    ))
    .bind(device_id.to_string())
    .fetch_one(get_pool())
    .await
    .map_err(|error| error.to_string())?;
    Ok(index.max(0) as u32)
}

async fn validate_template(exists: bool, template_id: Option<TemplateId>) -> Result<(), String> {
    let Some(template_id) = template_id else {
        return exists
            .then_some(())
            .ok_or_else(|| "追加队列任务必须选择真实时间模板。".to_string());
    };
    let found = sqlx::query_scalar::<_, i64>("SELECT 1 FROM time_templates WHERE id = ? LIMIT 1")
        .bind(template_id.to_string())
        .fetch_optional(get_pool())
        .await
        .map_err(|error| error.to_string())?
        .is_some();
    found
        .then_some(())
        .ok_or_else(|| "所选时间模板不存在或已失效，请重新选择。".to_string())
}

async fn validate_platform(device_id: DeviceId, script_id: ScriptId) -> Result<(), String> {
    let device = load_platform("devices", &device_id.to_string())
        .await?
        .ok_or_else(|| format!("设备不存在: {device_id}"))?;
    let script = load_platform("scripts", &script_id.to_string())
        .await?
        .ok_or_else(|| format!("脚本不存在: {script_id}"))?;
    if device == script {
        Ok(())
    } else {
        Err(format!(
            "脚本平台不匹配，设备平台={device}，脚本平台={script}"
        ))
    }
}

async fn load_platform(table: &str, id: &str) -> Result<Option<String>, String> {
    sqlx::query_scalar::<_, Option<String>>(&format!(
        "SELECT json_extract(data, '$.platform') FROM {table} WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(get_pool())
    .await
    .map_err(|error| error.to_string())
    .map(Option::flatten)
}
