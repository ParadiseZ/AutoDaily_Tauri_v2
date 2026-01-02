use crate::infrastructure::path_resolve::model_path::PathUtil;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, Database, Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use serde_json::json;
use sqlx::sqlite::SqliteRow;
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
            `data` JSON NOT NULL
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
    async fn exec_sql_vec<T>(sql: &str) -> Result<Vec<T>, String> {
        let res : Vec<T> = sqlx::query_as(sql)
            .fetch_all(get_pool())
            .await
            .map_err(|e| e.to_string())?;
        Ok(res)
    }

    async fn exec_sql_one<T>(sql: &str,bind: &str) -> Result<Option<T>, String> {
        let res : Option<T> = sqlx::query_as(sql)
            .bind(bind)
            .fetch_optional(get_pool())
            .await
            .map_err(|e| e.to_string())?;
        Ok(res)
    }

    fn db_res_to_vec<T>(rows: Vec<T>,column : &str) -> Result<Vec<T>, String>{
        let mut results = Vec::new();
        for row in rows {
            let data: String = row.get(column);
            let item: T = serde_json::from_str(&data).map_err(|e| e.to_string())?;
            results.push(item);
        }
        Ok( results)
    }

    fn db_res_to_one<T>(row: Option<T>,column : &str) -> Result<Option<T>, String>{
        if let Some(row) = row {
            let data: String = row.get(column);
            let item: T = serde_json::from_str(&data).map_err(|e| e.to_string())?;
            Ok(Some(item))
        }else { 
            Ok(None)
        }
    }
    /// 获取表中所有数据
    pub async fn get_id_data_all<T>(table: &str) -> Result<Vec<T>, String>
    where T: DeserializeOwned
    {
        let query = format!("SELECT id,data FROM {}", table);
        Self::exec_sql_vec::<T>(&query).await
    }

    /// 根据 ID 获取单条数据
    pub async fn get_id_data_by_id<T>(table: &str, id: &str) -> Result<Option<T>, String>
    where T: DeserializeOwned
    {
        let query = format!("SELECT id,data FROM {} WHERE id = ?", table);
        Ok(
            Self::exec_sql_one::<T>(&query, id).await?
        )

    }

    /// 根据属性值获取
    pub async fn get_by_prop_val<T>(table: &str, prop: &str, value: &str) -> Result<Vec<T>, String>
    where T: DeserializeOwned
    {
        let query = format!("SELECT id,data FROM {} WHERE data->>'$.{}' = '{}';", table, prop,value);
        Self::exec_sql_vec::<T>(&query).await
    }

    /// 插入或更新数据
    pub async fn upsert_id_data<T>(table: &str, id: &str, item: &T) -> Result<(), String>
    where T: Serialize
    {
        let pool = get_pool();
        let data = serde_json::to_string(item).map_err(|e| e.to_string())?;
        let query = format!(
            "INSERT INTO {} (id, data) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET data = excluded.data",
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
        let query = format!("DELETE FROM {} WHERE id = ?", table);
        sqlx::query(&query)
            .bind(id)
            .execute(get_pool())
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
