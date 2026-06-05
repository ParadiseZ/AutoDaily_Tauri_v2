use super::schema::SCHEMA_MIGRATIONS_TABLE_SQL;
use sqlx::{Pool, Sqlite};

pub(crate) async fn run_schema_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(SCHEMA_MIGRATIONS_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
