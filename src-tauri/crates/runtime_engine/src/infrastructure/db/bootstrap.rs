use super::{migrations, schema, POOL};
use crate::infrastructure::path_resolve::model_path::PathUtil;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use tauri::{AppHandle, Manager};

/// 子进程初始化数据库
pub async fn init_db_with_path(db_dir: &PathBuf) -> Result<(), String> {
    PathUtil::sure_parent_exists(db_dir).map_err(|e| e.to_string())?;
    let db_path = db_dir.join("autodaily.db");

    let connect_options =
        SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
            .map_err(|e| e.to_string())?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .pragma("foreign_keys", "ON");

    let pool = SqlitePool::connect_with(connect_options)
        .await
        .map_err(|e| e.to_string())?;

    POOL.set(pool)
        .map_err(|_| "Failed to set DB pool".to_string())?;
    Ok(())
}

/// 主进程初始化数据库 (通过 AppHandle)
pub async fn init_db(app_handle: &AppHandle) -> Result<(), String> {
    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    init_db_with_path(&db_path).await?;
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
