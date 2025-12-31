use crate::infrastructure::path_resolve::model_path::PathUtil;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use serde_json::json;
use tauri::AppHandle;
use tauri::Manager;
use tokio::sync::OnceCell;

pub static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

/// 子进程初始化数据库
pub async fn init_db_with_path(db_dir: &PathBuf) -> Result<(), String> {
    PathUtil::sure_parent_exists(db_dir).map_err(|e| e.to_string())?;
    let db_path = db_dir.join("autodaily.db");

    // 关键：开启 WAL 模式和同步模式，提升多进程性能
    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
        .map_err(|e| e.to_string())?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // 开启 WAL
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
    let pool = SqlitePool::connect_with(connect_options)
        .await
        .map_err(|e| e.to_string())?;
    POOL.set(pool).map_err(|_| "Failed to set DB pool".to_string())?;
    Ok(())
}

/// 主进程使用的初始化数据库
pub async fn init_db(app_handle: &AppHandle) -> Result<(), String> {
    let db_path = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    init_db_with_path(&db_path).await?;
    init_tables(POOL.get().unwrap()).await?;
    Ok(())
}

/// 初始化表
pub async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), String>{
    // 创建设备配置表
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS device_configs (
            id TEXT PRIMARY KEY,
            content JSON NOT NULL
        )",
    )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?

    // 创建通用配置表，用于存储其他类型的结构体

}

/// 获取数据库连接池
pub fn get_pool() -> &'static SqlitePool {
    POOL.get().expect("Database pool not initialized")
}

/// 通用的数据库操作封装
pub struct DbRepo;

impl DbRepo {
    /// 获取表中所有数据
    pub async fn get_content<T>(table: &str) -> Result<Vec<T>, String>
    where T: DeserializeOwned 
    {
        let pool = get_pool();
        let query = format!("SELECT content FROM {}", table);
        let rows = sqlx::query(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for row in rows {
            let data: String = row.get("content");
            let item: T = serde_json::from_str(&data).map_err(|e| e.to_string())?;
            results.push(item);
        }
        Ok(results)
    }

    /// 根据 ID 获取单条数据
    pub async fn get_by_id<T>(table: &str, id: &str) -> Result<Option<T>, String>
    where T: DeserializeOwned
    {
        let pool = get_pool();
        let query = format!("SELECT content FROM {} WHERE id = ?", table);
        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(row) = row {
            let data: String = row.get("content");
            let item: T = serde_json::from_str(&data).map_err(|e| e.to_string())?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// 插入或更新数据
    pub async fn upsert<T>(table: &str, id: &str, item: &T) -> Result<(), String>
    where T: Serialize
    {
        let pool = get_pool();
        let data = serde_json::to_string(item).map_err(|e| e.to_string())?;
        let query = format!(
            "INSERT INTO {} (id, content) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET content = excluded.content",
            table
        );
        sqlx::query(&query)
            .bind(id)
            .bind(data)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 删除数据
    pub async fn delete(table: &str, id: &str) -> Result<(), String> {
        let pool = get_pool();
        let query = format!("DELETE FROM {} WHERE id = ?", table);
        sqlx::query(&query)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
