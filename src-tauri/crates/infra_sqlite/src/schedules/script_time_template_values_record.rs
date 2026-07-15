use ad_kernel::ids::{AccountId, DeviceId, ScriptId, ScriptTemplateValueId, TemplateId};
use domain_schedule::TemplateValueProfile;
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(FromRow)]
struct TemplateValueRow {
    id: String,
    device_id: Option<String>,
    script_id: String,
    time_template_id: String,
    account_id: Option<AccountId>,
    values_json: Json<serde_json::Value>,
    created_at: String,
    updated_at: String,
}

fn id(value: String) -> Result<ad_kernel::ids::UuidV7, String> {
    Uuid::parse_str(&value)
        .map(Into::into)
        .map_err(|error| error.to_string())
}

impl TryFrom<TemplateValueRow> for TemplateValueProfile {
    type Error = String;
    fn try_from(row: TemplateValueRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id(row.id)?,
            device_id: row.device_id.map(id).transpose()?,
            script_id: id(row.script_id)?,
            time_template_id: id(row.time_template_id)?,
            account_id: row.account_id,
            values: row.values_json.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub async fn find_template_value(
    script_id: ScriptId,
    time_template_id: TemplateId,
    device_id: Option<DeviceId>,
    account_id: Option<AccountId>,
) -> Result<Option<TemplateValueProfile>, String> {
    sqlx::query_as::<_, TemplateValueRow>("SELECT id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at FROM script_time_template_values WHERE script_id = ?1 AND time_template_id = ?2 AND ((device_id = ?3 AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4)) OR (device_id = ?3 AND account_id IS NULL) OR (device_id IS NULL AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4)) OR (device_id IS NULL AND account_id IS NULL)) ORDER BY CASE WHEN device_id = ?3 AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4) THEN 0 WHEN device_id = ?3 AND account_id IS NULL THEN 1 WHEN device_id IS NULL AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4) THEN 2 ELSE 3 END LIMIT 1")
        .bind(script_id.to_string()).bind(time_template_id.to_string()).bind(device_id.map(|id| id.to_string())).bind(account_id)
        .fetch_optional(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?
        .map(TryInto::try_into).transpose()
}

pub async fn find_template_value_exact(
    device_id: Option<DeviceId>,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<Option<TemplateValueProfile>, String> {
    sqlx::query_as::<_, TemplateValueRow>("SELECT id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at FROM script_time_template_values WHERE ((device_id IS NULL AND ?1 IS NULL) OR device_id = ?1) AND script_id = ?2 AND time_template_id = ?3 AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4) LIMIT 1")
        .bind(device_id.map(|id| id.to_string())).bind(script_id.to_string()).bind(time_template_id.to_string()).bind(account_id)
        .fetch_optional(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?
        .map(TryInto::try_into).transpose()
}

pub async fn save_template_value(profile: &TemplateValueProfile) -> Result<(), String> {
    sqlx::query("INSERT INTO script_time_template_values (id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET device_id = excluded.device_id, script_id = excluded.script_id, time_template_id = excluded.time_template_id, account_id = excluded.account_id, values_json = excluded.values_json, updated_at = excluded.updated_at")
        .bind(profile.id.to_string()).bind(profile.device_id.map(|id| id.to_string())).bind(profile.script_id.to_string()).bind(profile.time_template_id.to_string()).bind(&profile.account_id).bind(Json(&profile.values)).bind(&profile.created_at).bind(&profile.updated_at)
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn delete_template_value(
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM script_time_template_values WHERE device_id = ? AND script_id = ? AND time_template_id = ? AND ((account_id IS NULL AND ?4 IS NULL) OR account_id = ?4)")
        .bind(device_id.to_string()).bind(script_id.to_string()).bind(time_template_id.to_string()).bind(account_id)
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn upsert_template_value(
    device_id: DeviceId,
    script_id: ScriptId,
    time_template_id: TemplateId,
    account_id: Option<AccountId>,
    values: serde_json::Value,
    now: String,
) -> Result<(), String> {
    let existing = find_template_value_exact(
        Some(device_id),
        script_id,
        time_template_id,
        account_id.clone(),
    )
    .await?;
    let profile = TemplateValueProfile {
        id: existing
            .as_ref()
            .map(|value| value.id)
            .unwrap_or_else(ScriptTemplateValueId::new_v7),
        device_id: Some(device_id),
        script_id,
        time_template_id,
        account_id,
        values,
        created_at: existing
            .as_ref()
            .map(|value| value.created_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now,
    };
    save_template_value(&profile).await
}
