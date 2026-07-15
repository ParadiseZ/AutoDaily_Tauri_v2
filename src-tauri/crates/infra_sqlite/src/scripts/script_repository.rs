use ad_kernel::ids::ScriptId;
use domain_script::{ScriptInfo, ScriptProfile};
use sqlx::{FromRow, types::Json};
use uuid::Uuid;

#[derive(FromRow)]
struct ScriptRow {
    id: String,
    data: Json<ScriptInfo>,
}

impl TryFrom<ScriptRow> for ScriptProfile {
    type Error = String;

    fn try_from(row: ScriptRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ScriptId::from(Uuid::parse_str(&row.id).map_err(|error| error.to_string())?),
            info: row.data.0,
        })
    }
}

pub async fn get_script(script_id: ScriptId) -> Result<Option<ScriptProfile>, String> {
    sqlx::query_as::<_, ScriptRow>("SELECT id, `data` FROM scripts WHERE id = ?")
        .bind(script_id.to_string())
        .fetch_optional(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .map(TryInto::try_into)
        .transpose()
}

pub async fn list_scripts() -> Result<Vec<ScriptProfile>, String> {
    sqlx::query_as::<_, ScriptRow>("SELECT id, `data` FROM scripts ORDER BY id")
        .fetch_all(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

pub async fn find_dev_script_by_cloud_id(
    cloud_id: ScriptId,
) -> Result<Option<ScriptProfile>, String> {
    sqlx::query_as::<_, ScriptRow>(
        "SELECT id, `data` FROM scripts
         WHERE json_extract(data, '$.scriptType') = 'Dev'
           AND json_extract(data, '$.cloudId') = ?",
    )
    .bind(cloud_id.to_string())
    .fetch_optional(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?
    .map(TryInto::try_into)
    .transpose()
}

pub async fn save_script(profile: &ScriptProfile) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO scripts (id, `data`) VALUES (?, ?)
         ON CONFLICT(id) DO UPDATE SET `data` = excluded.`data`",
    )
    .bind(profile.id.to_string())
    .bind(Json(&profile.info))
    .execute(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn delete_script(script_id: ScriptId) -> Result<(), String> {
    sqlx::query("DELETE FROM scripts WHERE id = ?")
        .bind(script_id.to_string())
        .execute(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
