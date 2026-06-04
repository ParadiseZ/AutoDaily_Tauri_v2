use super::schema::SCHEMA_MIGRATIONS_TABLE_SQL;
use sqlx::{Pool, Sqlite};

pub(crate) async fn run_schema_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(SCHEMA_MIGRATIONS_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
async fn table_columns(pool: &Pool<Sqlite>, table_name: &str) -> Result<Vec<String>, String> {
    let rows = sqlx::query(&format!("PRAGMA table_info({table_name})"))
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect())
}