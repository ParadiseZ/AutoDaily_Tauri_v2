use crate::bootstrap::get_pool;
use ad_kernel::ids::ScriptId;
use domain_script::{ScriptType, ensure_editable};

pub async fn ensure_stored_script_editable(script_id: ScriptId) -> Result<(), String> {
    if let Some(script_type) = load_script_type(script_id).await? {
        ensure_editable(&script_type).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub async fn ensure_existing_script_editable(script_id: ScriptId) -> Result<(), String> {
    let script_type = load_script_type(script_id)
        .await?
        .ok_or_else(|| "脚本不存在".to_string())?;
    ensure_editable(&script_type).map_err(|error| error.to_string())
}

async fn load_script_type(script_id: ScriptId) -> Result<Option<ScriptType>, String> {
    let value = sqlx::query_scalar::<_, Option<String>>(
        "SELECT json_extract(data, '$.scriptType') FROM scripts WHERE id = ?",
    )
    .bind(script_id.to_string())
    .fetch_optional(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    value
        .flatten()
        .map(|value| match value.as_str() {
            "dev" | "Dev" => Ok(ScriptType::Dev),
            "published" | "Published" => Ok(ScriptType::Published),
            _ => Err(format!("未知脚本类型: {value}")),
        })
        .transpose()
}
