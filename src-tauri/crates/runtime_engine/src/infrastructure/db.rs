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

const SCRIPT_TIME_TEMPLATE_VALUES_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS script_time_template_values (
            id TEXT PRIMARY KEY,
            device_id TEXT,
            script_id TEXT NOT NULL,
            time_template_id TEXT NOT NULL,
            account_id TEXT,
            values_json JSON NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE,
            FOREIGN KEY (time_template_id) REFERENCES time_templates(id) ON DELETE CASCADE
        )";

const SCRIPT_TIME_TEMPLATE_VALUES_SCOPE_INDEX_SQL: &str = "CREATE UNIQUE INDEX IF NOT EXISTS idx_script_time_template_values_scope
        ON script_time_template_values (
            ifnull(device_id, ''),
            script_id,
            time_template_id,
            ifnull(account_id, '')
        )";

const RECOVERY_CHECKPOINT_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS device_runtime_checkpoints (
            execution_id TEXT PRIMARY KEY,
            source_session_id TEXT NOT NULL,
            device_id TEXT NOT NULL UNIQUE,
            run_target_json JSON NOT NULL,
            assignment_id TEXT,
            script_id TEXT NOT NULL,
            time_template_id TEXT,
            account_id TEXT,
            task_id TEXT,
            step_id TEXT,
            resume_mode JSON NOT NULL,
            definition_fingerprint TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE,
            FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )";

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
            row_type TEXT NOT NULL DEFAULT 'task',
            trigger_mode TEXT NOT NULL DEFAULT 'rootOnly',
            record_schedule BOOLEAN NOT NULL DEFAULT 1,
            section_id TEXT,
            indent_level INTEGER NOT NULL DEFAULT 0,
            default_task_cycle JSON NOT NULL DEFAULT '\"everyRun\"',
            exec_max INTEGER NOT NULL DEFAULT 0,
            show_enabled_toggle BOOLEAN NOT NULL DEFAULT 1,
            default_enabled BOOLEAN NOT NULL DEFAULT 1,
            task_tone TEXT NOT NULL DEFAULT 'normal',
            is_hidden BOOLEAN NOT NULL DEFAULT 0,
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
            execution_id TEXT,
            assignment_id TEXT,
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
    ensure_device_script_schedule_columns(pool).await?;

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

    // 12. 脚本时间模板变量值表
    sqlx::query(SCRIPT_TIME_TEMPLATE_VALUES_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    ensure_script_time_template_values_schema(pool).await?;

    // 13. 设备恢复检查点
    sqlx::query(RECOVERY_CHECKPOINT_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    ensure_recovery_checkpoint_schema(pool).await?;

    Ok(())
}

async fn ensure_device_script_schedule_columns(pool: &Pool<Sqlite>) -> Result<(), String> {
    let rows = sqlx::query("PRAGMA table_info(device_script_schedules)")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let column_names: Vec<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect();

    if !column_names.iter().any(|column| column == "execution_id") {
        sqlx::query("ALTER TABLE device_script_schedules ADD COLUMN execution_id TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !column_names.iter().any(|column| column == "assignment_id") {
        sqlx::query("ALTER TABLE device_script_schedules ADD COLUMN assignment_id TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

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

    if !ensure_column("row_type") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN row_type TEXT NOT NULL DEFAULT 'task'")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("trigger_mode") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN trigger_mode TEXT NOT NULL DEFAULT 'rootOnly'")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("record_schedule") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN record_schedule BOOLEAN NOT NULL DEFAULT 1")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("section_id") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN section_id TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("indent_level") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN indent_level INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("default_task_cycle") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN default_task_cycle JSON NOT NULL DEFAULT '\"everyRun\"'")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("exec_max") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN exec_max INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("show_enabled_toggle") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN show_enabled_toggle BOOLEAN NOT NULL DEFAULT 1")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("default_enabled") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN default_enabled BOOLEAN NOT NULL DEFAULT 1")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !ensure_column("task_tone") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN task_tone TEXT NOT NULL DEFAULT 'normal'")
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

    if ensure_column("task_type") || ensure_column("nodes") || ensure_column("edges") {
        rebuild_script_tasks_table(pool).await?;
    }

    Ok(())
}

async fn ensure_script_time_template_values_schema(pool: &Pool<Sqlite>) -> Result<(), String> {
    let rows = sqlx::query("PRAGMA table_info(script_time_template_values)")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let column_names: Vec<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect();

    let has_device_id = column_names.iter().any(|column| column == "device_id");
    let has_account_id = column_names.iter().any(|column| column == "account_id");

    if !has_device_id || !has_account_id {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        sqlx::query("ALTER TABLE script_time_template_values RENAME TO script_time_template_values_legacy")
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(SCRIPT_TIME_TEMPLATE_VALUES_TABLE_SQL)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(SCRIPT_TIME_TEMPLATE_VALUES_SCOPE_INDEX_SQL)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO script_time_template_values (
                id,
                device_id,
                script_id,
                time_template_id,
                account_id,
                values_json,
                created_at,
                updated_at
            )
            SELECT
                id,
                NULL,
                script_id,
                time_template_id,
                NULL,
                values_json,
                created_at,
                updated_at
            FROM script_time_template_values_legacy",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("DROP TABLE script_time_template_values_legacy")
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    sqlx::query(SCRIPT_TIME_TEMPLATE_VALUES_SCOPE_INDEX_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn ensure_recovery_checkpoint_schema(pool: &Pool<Sqlite>) -> Result<(), String> {
    let rows = sqlx::query("PRAGMA table_info(device_runtime_checkpoints)")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let column_names: Vec<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect();

    let has_device_unique = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1) FROM pragma_index_list('device_runtime_checkpoints') WHERE name = 'idx_device_runtime_checkpoints_device_id'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?
        > 0;

    if !column_names.iter().any(|column| column == "device_id")
        || !column_names.iter().any(|column| column == "resume_mode")
    {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        sqlx::query("ALTER TABLE device_runtime_checkpoints RENAME TO device_runtime_checkpoints_legacy")
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(RECOVERY_CHECKPOINT_TABLE_SQL)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO device_runtime_checkpoints (
                execution_id,
                source_session_id,
                device_id,
                run_target_json,
                assignment_id,
                script_id,
                time_template_id,
                account_id,
                task_id,
                step_id,
                resume_mode,
                definition_fingerprint,
                updated_at
            )
            SELECT
                execution_id,
                source_session_id,
                device_id,
                run_target_json,
                assignment_id,
                script_id,
                time_template_id,
                account_id,
                task_id,
                step_id,
                COALESCE(resume_mode, '\"fromTaskStart\"'),
                definition_fingerprint,
                updated_at
            FROM device_runtime_checkpoints_legacy",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("DROP TABLE device_runtime_checkpoints_legacy")
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
    }

    if !has_device_unique {
        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_device_runtime_checkpoints_device_id
             ON device_runtime_checkpoints (device_id)",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
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
            row_type TEXT NOT NULL DEFAULT 'task',
            trigger_mode TEXT NOT NULL DEFAULT 'rootOnly',
            record_schedule BOOLEAN NOT NULL DEFAULT 1,
            section_id TEXT,
            indent_level INTEGER NOT NULL DEFAULT 0,
            default_task_cycle JSON NOT NULL DEFAULT '\"everyRun\"',
            exec_max INTEGER NOT NULL DEFAULT 0,
            show_enabled_toggle BOOLEAN NOT NULL DEFAULT 1,
            default_enabled BOOLEAN NOT NULL DEFAULT 1,
            task_tone TEXT NOT NULL DEFAULT 'normal',
            is_hidden BOOLEAN NOT NULL DEFAULT 0,
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
        "INSERT INTO script_tasks_v2 (id, script_id, `name`, row_type, trigger_mode, record_schedule, section_id, indent_level, default_task_cycle, exec_max, show_enabled_toggle, default_enabled, task_tone, is_hidden, `data`, created_at, updated_at, deleted_at, is_deleted, `index`)
         SELECT
            id,
            script_id,
            `name`,
            COALESCE(NULLIF(row_type, ''), 'task'),
            CASE
                WHEN trigger_mode IN ('rootOnly', 'linkOnly', 'rootAndLink') THEN trigger_mode
                WHEN task_type = 'child' THEN 'linkOnly'
                ELSE 'rootOnly'
            END,
            COALESCE(record_schedule, 1),
            section_id,
            COALESCE(indent_level, 0),
            COALESCE(default_task_cycle, '\"everyRun\"'),
            COALESCE(exec_max, 0),
            COALESCE(show_enabled_toggle, 1),
            COALESCE(default_enabled, 1),
            COALESCE(NULLIF(task_tone, ''), 'normal'),
            COALESCE(is_hidden, 0),
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
