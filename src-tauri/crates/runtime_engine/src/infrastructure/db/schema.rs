use sqlx::{Pool, Sqlite};

pub(crate) const SCRIPT_TIME_TEMPLATE_VALUES_TABLE_SQL: &str =
    "CREATE TABLE IF NOT EXISTS script_time_template_values (
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

pub(crate) const SCRIPT_TIME_TEMPLATE_VALUES_SCOPE_INDEX_SQL: &str =
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_script_time_template_values_scope
        ON script_time_template_values (
            ifnull(device_id, ''),
            script_id,
            time_template_id,
            ifnull(account_id, '')
        )";

pub(crate) const SCHEMA_MIGRATIONS_TABLE_SQL: &str =
    "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )";

pub(crate) const SCRIPT_TRANSFER_RECORDS_TABLE_SQL: &str =
    "CREATE TABLE IF NOT EXISTS script_transfer_records (
            id TEXT PRIMARY KEY,
            direction TEXT NOT NULL,
            local_script_id TEXT,
            cloud_script_id TEXT,
            script_name TEXT,
            status TEXT NOT NULL DEFAULT 'running',
            model_file_count INTEGER NOT NULL DEFAULT 0,
            completed_model_file_count INTEGER NOT NULL DEFAULT 0,
            latest_file_name TEXT,
            bytes_transferred INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            latest_message TEXT,
            error_message TEXT,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (local_script_id) REFERENCES scripts(id) ON DELETE CASCADE
        )";

pub(crate) const SCRIPT_TRANSFER_RECORDS_SCOPE_INDEX_SQL: &str =
    "CREATE INDEX IF NOT EXISTS idx_script_transfer_records_scope
        ON script_transfer_records (
            direction,
            ifnull(local_script_id, ''),
            ifnull(cloud_script_id, ''),
            updated_at DESC
        )";

pub(crate) const DEVICE_SCRIPT_SCHEDULES_DEDUP_INDEX_SQL: &str =
    "CREATE INDEX IF NOT EXISTS idx_device_script_schedules_dedup_lookup
        ON device_script_schedules (
            assignment_id,
            dedup_scope_hash,
            task_id,
            status,
            completed_at DESC,
            started_at DESC
        )";

pub(crate) fn script_tasks_table_sql(table_name: &str) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {table_name} (
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
        )"
    )
}

pub(crate) async fn create_base_tables(pool: &Pool<Sqlite>) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            `data` JSON NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scripts (
            id TEXT PRIMARY KEY,
            `data` JSON NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

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

    sqlx::query(&script_tasks_table_sql("script_tasks"))
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS device_script_schedules (
            id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL,
            execution_id TEXT,
            assignment_id TEXT,
            script_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            dedup_scope_hash TEXT NOT NULL DEFAULT '',
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
    sqlx::query(DEVICE_SCRIPT_SCHEDULES_DEDUP_INDEX_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

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

    sqlx::query(SCRIPT_TIME_TEMPLATE_VALUES_TABLE_SQL)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
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
