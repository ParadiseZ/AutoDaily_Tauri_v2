use super::schema::SCHEMA_MIGRATIONS_TABLE_SQL;
use sqlx::{Pool, Sqlite};

const DEVICE_LOG_CONFIG_DEFAULTS_MIGRATION_VERSION: &str =
    "2026-06-10_device_log_config_defaults";

async fn has_migration(pool: &Pool<Sqlite>, version: &str) -> Result<bool, String> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = ?",
    )
    .bind(version)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

async fn apply_migration(
    pool: &Pool<Sqlite>,
    version: &str,
    name: &str,
    statements: &[&str],
) -> Result<(), String> {
    if has_migration(pool, version).await? {
        return Ok(());
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for statement in statements {
        sqlx::query(statement)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    sqlx::query(
        "INSERT INTO schema_migrations (version, name) VALUES (?, ?)",
    )
    .bind(version)
    .bind(name)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn run_schema_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(SCHEMA_MIGRATIONS_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    apply_migration(
        pool,
        DEVICE_LOG_CONFIG_DEFAULTS_MIGRATION_VERSION,
        "backfill device logLevel/logToFile defaults",
        &[
            "UPDATE devices
                SET data = json_set(data, '$.logLevel', 'Off')
              WHERE COALESCE(json_type(data, '$.logLevel'), 'null') = 'null'",
            "UPDATE devices
                SET data = json_set(data, '$.logToFile', json('true'))
              WHERE COALESCE(json_type(data, '$.logToFile'), 'null') = 'null'",
        ],
    )
    .await?;

    Ok(())
}
