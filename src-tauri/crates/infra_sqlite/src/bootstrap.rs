use super::{POOL, migrations, schema};
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;
use std::str::FromStr;

async fn open_pool(db_dir: &Path) -> Result<SqlitePool, String> {
    std::fs::create_dir_all(db_dir).map_err(|error| error.to_string())?;
    let db_path = db_dir.join("autodaily.db");

    let connect_options =
        SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
            .map_err(|e| e.to_string())?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .pragma("foreign_keys", "ON");

    SqlitePool::connect_with(connect_options)
        .await
        .map_err(|error| error.to_string())
}

/// 子进程初始化数据库连接。
pub async fn init_db_with_path(db_dir: &Path) -> Result<(), String> {
    let pool = open_pool(db_dir).await?;
    POOL.set(pool)
        .map_err(|_| "Failed to set DB pool".to_string())?;
    Ok(())
}

/// 主进程初始化数据库连接并执行 schema 初始化与迁移。
pub async fn init_db_and_migrate_with_path(db_dir: &Path) -> Result<(), String> {
    init_db_with_path(db_dir).await?;
    init_tables(POOL.get().unwrap()).await?;
    Ok(())
}

async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), String> {
    schema::create_base_tables(pool).await?;
    migrations::run_schema_migrations(pool).await?;
    Ok(())
}

/// 获取全局连接池
pub fn get_pool() -> &'static SqlitePool {
    POOL.get().expect("Database pool not initialized")
}

#[cfg(test)]
mod tests {
    use super::{init_tables, open_pool};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn initializes_the_base_schema_without_global_state() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("auto_daily_infra_sqlite_{unique}"));
        let pool = open_pool(&dir).await.unwrap();

        init_tables(&pool).await.unwrap();
        let table_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'scripts'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(table_count, 1);
        pool.close().await;
        // Windows may keep SQLite's WAL sidecar briefly after the pool closes.
        // Cleanup is best-effort; the unique temp path prevents test collisions.
        let _ = std::fs::remove_dir_all(dir);
    }
}
