use crate::infrastructure::path_resolve::model_path::PathUtil;
use serde::Serialize;
use sqlx::types::Json;
use sqlx::Row;
use sqlx::{sqlite::SqliteConnectOptions, FromRow, Pool, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
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
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .pragma("foreign_keys", "ON"); // 开启外键支持

    let pool = SqlitePool::connect_with(connect_options)
        .await
        .map_err(|e| e.to_string())?;

    POOL.set(pool).map_err(|_| "Failed to set DB pool".to_string())?;
    Ok(())
}

/// 主进程初始化数据库 (通过 AppHandle)
pub async fn init_db(app_handle: &AppHandle) -> Result<(), String> {
    let db_path = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    init_db_with_path(&db_path).await?;
    init_tables(POOL.get().unwrap()).await?;
    Ok(())
}

/// 初始化所有表结构
pub async fn init_tables(pool: &Pool<Sqlite>) -> Result<(), String> {
    // 1. 设备配置表 (ID + JSON 内容)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            `data` JSON NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 2. 脚本列表 (ID + JSON 内容)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scripts (
            id TEXT PRIMARY KEY,
            `data` JSON NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 3. 策略 (Policies)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS policies (
            id TEXT PRIMARY KEY,
            script_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            `data` JSON NOT NULL,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 4. 策略组 (Policy Groups)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS policy_groups (
            id TEXT PRIMARY KEY,
            script_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            `data` JSON NOT NULL,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 5. 策略集合 (Policy Sets)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS policy_sets (
            id TEXT PRIMARY KEY,
            script_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            `data` JSON NOT NULL,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 6. 关联表: 组与策略 (Many-to-Many)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS group_policies (
            group_id TEXT NOT NULL,
            policy_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            PRIMARY KEY (group_id, policy_id),
            FOREIGN KEY (group_id) REFERENCES policy_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (policy_id) REFERENCES policies(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 7. 关联表: 集合与组 (Many-to-Many)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS set_groups (
            set_id TEXT NOT NULL,
            group_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            PRIMARY KEY (set_id, group_id),
            FOREIGN KEY (set_id) REFERENCES policy_sets(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES policy_groups(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 8. 脚本任务逻辑表
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS script_tasks (
            id TEXT PRIMARY KEY,
            script_id TEXT NOT NULL,
            `name` TEXT NOT NULL,
            is_hidden BOOLEAN NOT NULL DEFAULT 0,
            task_type TEXT NOT NULL DEFAULT 'main',
            `data` JSON NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            deleted_at TEXT,
            is_deleted BOOLEAN NOT NULL DEFAULT 0,
            `index` INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    ensure_script_tasks_columns(pool).await?;

    // 9. 设备脚本分配表（队列定义）
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS device_script_assignments (
            id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL,
            script_id TEXT NOT NULL,
            time_template_id TEXT,
            account_data JSON NOT NULL DEFAULT '{}',
            `index` INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 10. 设备脚本调度记录表（append-only）
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS device_script_schedules (
            id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL,
            script_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            task_cycle TEXT NOT NULL DEFAULT 'everyRun',
            status TEXT NOT NULL DEFAULT 'success',
            started_at TEXT NOT NULL,
            completed_at TEXT,
            message TEXT,
            FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 11. 时间模板表
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS time_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            start_time TEXT,
            end_time TEXT
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

async fn ensure_script_tasks_columns(pool: &Pool<Sqlite>) -> Result<(), String> {
    let rows = sqlx::query("PRAGMA table_info(script_tasks)")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let column_names: Vec<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect();

    let ensure_column = |name: &str| column_names.iter().any(|column| column == name);

    if !ensure_column("task_type") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN task_type TEXT NOT NULL DEFAULT 'main'")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("created_at") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN created_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("UPDATE script_tasks SET created_at = COALESCE(created_at, CURRENT_TIMESTAMP)")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("updated_at") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN updated_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("UPDATE script_tasks SET updated_at = COALESCE(updated_at, CURRENT_TIMESTAMP)")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("deleted_at") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN deleted_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("is_deleted") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("index") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN `index` INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if ensure_column("nodes") || ensure_column("edges") {
        rebuild_script_tasks_table(pool).await?;
    }

    Ok(())
}

async fn rebuild_script_tasks_table(pool: &Pool<Sqlite>) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DROP TABLE IF EXISTS script_tasks_v2")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(
        "CREATE TABLE script_tasks_v2 (
            id TEXT PRIMARY KEY,
            script_id TEXT NOT NULL,
            `name` TEXT NOT NULL,
            is_hidden BOOLEAN NOT NULL DEFAULT 0,
            task_type TEXT NOT NULL DEFAULT 'main',
            `data` JSON NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            deleted_at TEXT,
            is_deleted BOOLEAN NOT NULL DEFAULT 0,
            `index` INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO script_tasks_v2 (id, script_id, `name`, is_hidden, task_type, `data`, created_at, updated_at, deleted_at, is_deleted, `index`)
         SELECT
            id,
            script_id,
            `name`,
            COALESCE(is_hidden, 0),
            COALESCE(task_type, 'main'),
            `data`,
            COALESCE(created_at, CURRENT_TIMESTAMP),
            COALESCE(updated_at, CURRENT_TIMESTAMP),
            deleted_at,
            COALESCE(is_deleted, 0),
            COALESCE(`index`, 0)
         FROM script_tasks",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("DROP TABLE script_tasks")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("ALTER TABLE script_tasks_v2 RENAME TO script_tasks")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取全局连接池
pub fn get_pool() -> &'static SqlitePool {
    POOL.get().expect("Database pool not initialized")
}

/// 数据库操作仓库
pub struct DbRepo;

impl DbRepo {
    /// 这里的泛型 T 是你的 Data 部分 (例如 DeviceConfig)
    /// 返回 (ID, Data) 的元组列表
    pub async fn get_all<T>(table: &str) -> Result<Vec<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin + Send + Sync,
    {
        let pool = get_pool();
        let query = format!("SELECT id, `data` FROM {}", table);
        let rows: Vec<T> = sqlx::query_as(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    /// 根据 ID 获取单个记录
    pub async fn get_by_id<T>(table: &str, id: &str) -> Result<Option<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin  + Send + Sync,
    {
        let pool = get_pool();
        let query = format!("SELECT id,`data` FROM {} WHERE id = ?", table);
        let row:Option<T> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(row)
    }

    /// 插入或更新 ID + Data 模式的数据
    /// 你不需要手动转换 JSON，sqlx 会处理
    pub async fn upsert_id_data<T>(table: &str, id: &str, data: &Json<T>) -> Result<(), String>
    where
        T: Serialize + Send + Sync,
    {
        let pool = get_pool();
        let query = format!(
            "INSERT INTO {} (id, `data`) VALUES (?, ?)
             ON CONFLICT(id) DO UPDATE SET `data` = excluded.`data`",
            table
        );
        
        sqlx::query(&query)
            .bind(id)
            .bind(data) // 这里就是自动转换！
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
