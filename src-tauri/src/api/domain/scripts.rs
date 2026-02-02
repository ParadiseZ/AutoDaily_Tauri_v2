use crate::constant::table_name::{SCRIPT_TABLE, SCRIPT_TASK_TABLE};
use crate::infrastructure::core::ScriptId;
use crate::infrastructure::db::{DbRepo, get_pool};
use tauri::command;
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::ScriptTaskTable;
use sqlx::types::Json;

/// 获取所有脚本配置
#[command]
pub async fn get_all_scripts_cmd() -> Result<Vec<ScriptTable>, String> {
    DbRepo::get_all::<ScriptTable>(SCRIPT_TABLE).await
}

/// 根据 ID 获取脚本配置
#[command]
pub async fn get_script_by_id_cmd(script_id: ScriptId) -> Result<Option<ScriptTable>, String> {
    DbRepo::get_by_id(SCRIPT_TABLE, &script_id.to_string()).await
}

/// 保存（新增或更新）脚本配置
#[command]
pub async fn save_script_cmd(script: ScriptTable) -> Result<(), String> {
    DbRepo::upsert_id_data(SCRIPT_TABLE, &script.id.to_string(), &script.data).await
}

/// 删除脚本配置
#[command]
pub async fn delete_script_cmd(script_id: ScriptId) -> Result<(), String> {
    DbRepo::delete(SCRIPT_TABLE, &script_id.to_string()).await
}

/// 获取脚本关联的所有任务逻辑
#[command]
pub async fn get_script_tasks_cmd(script_id: ScriptId) -> Result<Vec<ScriptTaskTable>, String> {
    let pool = get_pool();
    let query = format!("SELECT * FROM {} WHERE script_id = ?", SCRIPT_TASK_TABLE);
    let rows: Vec<ScriptTaskTable> = sqlx::query_as(&query)
        .bind(script_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// 批量保存脚本任务逻辑
/// 这里采用简单策略：先删除该脚本的所有任务，再重新插入
#[command]
pub async fn save_script_tasks_cmd(script_id: ScriptId, tasks: Vec<ScriptTaskTable>) -> Result<(), String> {
    let pool = get_pool();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 删除旧任务
    let delete_query = format!("DELETE FROM {} WHERE script_id = ?", SCRIPT_TASK_TABLE);
    sqlx::query(&delete_query)
        .bind(script_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // 插入新任务
    for task in tasks {
        let insert_query = format!(
            "INSERT INTO {} (id, script_id, name, is_hidden, nodes, edges, `data`) VALUES (?, ?, ?, ?, ?, ?, ?)",
            SCRIPT_TASK_TABLE
        );
        sqlx::query(&insert_query)
            .bind(task.id.to_string())
            .bind(script_id.to_string())
            .bind(task.name)
            .bind(task.is_hidden)
            .bind(task.nodes)
            .bind(task.edges)
            .bind(task.data)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
