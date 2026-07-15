use ad_kernel::ids::TemplateId;
use domain_schedule::TimeTemplateProfile;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
struct TimeTemplateRow {
    id: String,
    name: String,
    start_time: Option<String>,
    end_time: Option<String>,
}

impl TryFrom<TimeTemplateRow> for TimeTemplateProfile {
    type Error = String;
    fn try_from(row: TimeTemplateRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TemplateId::from(Uuid::parse_str(&row.id).map_err(|error| error.to_string())?),
            name: row.name,
            start_time: row.start_time,
            end_time: row.end_time,
        })
    }
}

pub async fn list_time_templates() -> Result<Vec<TimeTemplateProfile>, String> {
    sqlx::query_as::<_, TimeTemplateRow>(
        "SELECT id, name, start_time, end_time FROM time_templates ORDER BY name",
    )
    .fetch_all(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?
    .into_iter()
    .map(TryInto::try_into)
    .collect()
}

pub async fn get_time_template(id: TemplateId) -> Result<Option<TimeTemplateProfile>, String> {
    sqlx::query_as::<_, TimeTemplateRow>(
        "SELECT id, name, start_time, end_time FROM time_templates WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?
    .map(TryInto::try_into)
    .transpose()
}

pub async fn save_time_template(template: &TimeTemplateProfile) -> Result<(), String> {
    sqlx::query("INSERT INTO time_templates (id, name, start_time, end_time) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET name = excluded.name, start_time = excluded.start_time, end_time = excluded.end_time")
        .bind(template.id.to_string()).bind(&template.name).bind(&template.start_time).bind(&template.end_time)
        .execute(crate::bootstrap::get_pool()).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn delete_time_template(template_id: TemplateId) -> Result<(), String> {
    sqlx::query("DELETE FROM time_templates WHERE id = ?")
        .bind(template_id.to_string())
        .execute(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
