use super::schema::{
    script_tasks_table_sql, DEVICE_SCRIPT_SCHEDULES_DEDUP_INDEX_SQL, SCHEMA_MIGRATIONS_TABLE_SQL,
    SCRIPT_TIME_TEMPLATE_VALUES_SCOPE_INDEX_SQL, SCRIPT_TIME_TEMPLATE_VALUES_TABLE_SQL,
    SCRIPT_TRANSFER_RECORDS_SCOPE_INDEX_SQL, SCRIPT_TRANSFER_RECORDS_TABLE_SQL,
};
use sqlx::{Pool, Row, Sqlite};

pub(crate) async fn run_schema_migrations(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(SCHEMA_MIGRATIONS_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if should_apply(pool, "2026043001").await? {
        ensure_script_tasks_columns(pool).await?;
        mark_applied(pool, "2026043001", "ensure_script_tasks_current_columns").await?;
    }

    if should_apply(pool, "2026043002").await? {
        ensure_device_script_schedule_columns(pool).await?;
        mark_applied(
            pool,
            "2026043002",
            "ensure_device_script_schedule_runtime_columns",
        )
        .await?;
    }

    if should_apply(pool, "2026043003").await? {
        ensure_script_time_template_values_schema(pool).await?;
        mark_applied(
            pool,
            "2026043003",
            "ensure_script_time_template_values_scope",
        )
        .await?;
    }

    if should_apply(pool, "2026051401").await? {
        ensure_script_transfer_records_schema(pool).await?;
        mark_applied(
            pool,
            "2026051401",
            "ensure_script_transfer_records_schema",
        )
        .await?;
    }

    if should_apply(pool, "2026060201").await? {
        ensure_device_script_schedule_dedup_schema(pool).await?;
        mark_applied(
            pool,
            "2026060201",
            "ensure_device_script_schedule_dedup_scope",
        )
        .await?;
    }

    Ok(())
}

async fn should_apply(pool: &Pool<Sqlite>, version: &str) -> Result<bool, String> {
    let exists: Option<(String,)> =
        sqlx::query_as("SELECT version FROM schema_migrations WHERE version = ?")
            .bind(version)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    Ok(exists.is_none())
}

async fn mark_applied(pool: &Pool<Sqlite>, version: &str, name: &str) -> Result<(), String> {
    sqlx::query("INSERT INTO schema_migrations (version, name) VALUES (?, ?)")
        .bind(version)
        .bind(name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn ensure_device_script_schedule_columns(pool: &Pool<Sqlite>) -> Result<(), String> {
    let column_names = table_columns(pool, "device_script_schedules").await?;

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

async fn ensure_device_script_schedule_dedup_schema(pool: &Pool<Sqlite>) -> Result<(), String> {
    let column_names = table_columns(pool, "device_script_schedules").await?;

    if !column_names.iter().any(|column| column == "dedup_scope_hash") {
        sqlx::query(
            "ALTER TABLE device_script_schedules ADD COLUMN dedup_scope_hash TEXT NOT NULL DEFAULT ''",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    sqlx::query(DEVICE_SCRIPT_SCHEDULES_DEDUP_INDEX_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn ensure_script_tasks_columns(pool: &Pool<Sqlite>) -> Result<(), String> {
    let column_names = table_columns(pool, "script_tasks").await?;
    let has = |name: &str| column_names.iter().any(|column| column == name);

    if !has("row_type") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN row_type TEXT NOT NULL DEFAULT 'task'")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("trigger_mode") {
        sqlx::query(
            "ALTER TABLE script_tasks ADD COLUMN trigger_mode TEXT NOT NULL DEFAULT 'rootOnly'",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if !has("record_schedule") {
        sqlx::query(
            "ALTER TABLE script_tasks ADD COLUMN record_schedule BOOLEAN NOT NULL DEFAULT 1",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if !has("section_id") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN section_id TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("indent_level") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN indent_level INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("default_task_cycle") {
        sqlx::query(
            "ALTER TABLE script_tasks ADD COLUMN default_task_cycle JSON NOT NULL DEFAULT '\"everyRun\"'",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if !has("exec_max") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN exec_max INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("show_enabled_toggle") {
        sqlx::query(
            "ALTER TABLE script_tasks ADD COLUMN show_enabled_toggle BOOLEAN NOT NULL DEFAULT 1",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if !has("default_enabled") {
        sqlx::query(
            "ALTER TABLE script_tasks ADD COLUMN default_enabled BOOLEAN NOT NULL DEFAULT 1",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if !has("task_tone") {
        sqlx::query(
            "ALTER TABLE script_tasks ADD COLUMN task_tone TEXT NOT NULL DEFAULT 'normal'",
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    if !has("created_at") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN created_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("UPDATE script_tasks SET created_at = COALESCE(created_at, CURRENT_TIMESTAMP)")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("updated_at") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN updated_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("UPDATE script_tasks SET updated_at = COALESCE(updated_at, CURRENT_TIMESTAMP)")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("deleted_at") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN deleted_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("is_deleted") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !has("index") {
        sqlx::query("ALTER TABLE script_tasks ADD COLUMN `index` INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if has("task_type") || has("nodes") || has("edges") {
        rebuild_script_tasks_table(pool).await?;
    }

    Ok(())
}

async fn ensure_script_time_template_values_schema(pool: &Pool<Sqlite>) -> Result<(), String> {
    let column_names = table_columns(pool, "script_time_template_values").await?;
    let has_device_id = column_names.iter().any(|column| column == "device_id");
    let has_account_id = column_names.iter().any(|column| column == "account_id");

    if !has_device_id || !has_account_id {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        sqlx::query(
            "ALTER TABLE script_time_template_values RENAME TO script_time_template_values_legacy",
        )
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

async fn ensure_script_transfer_records_schema(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(SCRIPT_TRANSFER_RECORDS_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query(SCRIPT_TRANSFER_RECORDS_SCOPE_INDEX_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn rebuild_script_tasks_table(pool: &Pool<Sqlite>) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DROP TABLE IF EXISTS script_tasks_v2")
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(&script_tasks_table_sql("script_tasks_v2"))
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
