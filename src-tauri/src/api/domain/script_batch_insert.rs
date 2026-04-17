use crate::domain::scripts::policy::*;
/// 脚本及关联数据的批量插入工具
/// 用于统一 backend_download_script 和 clone_local_script_cmd 中的重复插入逻辑
use crate::domain::scripts::script_info::ScriptTable;
use crate::domain::scripts::script_task::ScriptTaskTable;
use sqlx::{Sqlite, Transaction};

macro_rules! bind_chunk {
    ($query:expr, $p:expr) => {
        $query
            .bind($p.id.to_string())
            .bind($p.script_id.to_string())
            .bind($p.order_index)
            .bind(&$p.data)
    };
}
/// 在给定事务中批量插入脚本及其所有关联数据
/// 使用多行 INSERT 语法优化性能
pub async fn batch_insert_script_related(
    tx: &mut Transaction<'_, Sqlite>,
    script: &ScriptTable,
    policies: &[PolicyTable],
    policy_groups: &[PolicyGroupTable],
    policy_sets: &[PolicySetTable],
    group_policies: &[GroupPolicyRelation],
    set_groups: &[SetGroupRelation],
    tasks: &[ScriptTaskTable],
) -> Result<(), String> {
    // 1. Insert script
    sqlx::query("INSERT INTO scripts (id, `data`) VALUES (?, ?)")
        .bind(script.id.to_string())
        .bind(&script.data)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("写入 Script 失败: {}", e))?;

    // 2. Batch insert policies (4 bind params each)
    if !policies.is_empty() {
        for chunk in policies.chunks(50) {
            let placeholders: Vec<String> =
                chunk.iter().map(|_| "(?, ?, ?, ?)".to_string()).collect();
            let sql = format!(
                "INSERT INTO policies (id, script_id, order_index, `data`) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for p in chunk {
                query = bind_chunk!(query, p);
            }
            query
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("批量写入 policies 失败: {}", e))?;
        }
    }

    // 3. Batch insert policy_groups (4 bind params each)
    if !policy_groups.is_empty() {
        for chunk in policy_groups.chunks(50) {
            let placeholders: Vec<String> =
                chunk.iter().map(|_| "(?, ?, ?, ?)".to_string()).collect();
            let sql = format!(
                "INSERT INTO policy_groups (id, script_id, order_index, `data`) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for g in chunk {
                query = bind_chunk!(query, g);
            }
            query
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("批量写入 policy_groups 失败: {}", e))?;
        }
    }

    // 4. Batch insert policy_sets (4 bind params each)
    if !policy_sets.is_empty() {
        for chunk in policy_sets.chunks(50) {
            let placeholders: Vec<String> =
                chunk.iter().map(|_| "(?, ?, ?, ?)".to_string()).collect();
            let sql = format!(
                "INSERT INTO policy_sets (id, script_id, order_index, `data`) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for s in chunk {
                query = bind_chunk!(query, s);
            }
            query
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("批量写入 policy_sets 失败: {}", e))?;
        }
    }

    // 5. Batch insert group_policies (3 bind params each)
    if !group_policies.is_empty() {
        for chunk in group_policies.chunks(50) {
            let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?, ?)".to_string()).collect();
            let sql = format!(
                "INSERT INTO group_policies (group_id, policy_id, order_index) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for gp in chunk {
                query = query
                    .bind(gp.group_id.to_string())
                    .bind(gp.policy_id.to_string())
                    .bind(gp.order_index);
            }
            query
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("批量写入 group_policies 失败: {}", e))?;
        }
    }

    // 6. Batch insert set_groups (3 bind params each)
    if !set_groups.is_empty() {
        for chunk in set_groups.chunks(50) {
            let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?, ?)".to_string()).collect();
            let sql = format!(
                "INSERT INTO set_groups (set_id, group_id, order_index) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for sg in chunk {
                query = query
                    .bind(sg.set_id.to_string())
                    .bind(sg.group_id.to_string())
                    .bind(sg.order_index);
            }
            query
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("批量写入 set_groups 失败: {}", e))?;
        }
    }

    // 7. Batch insert script_tasks
    if !tasks.is_empty() {
        for chunk in tasks.chunks(50) {
            let placeholders: Vec<String> = chunk
                .iter()
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string())
                .collect();
            let sql = format!(
                "INSERT INTO script_tasks (id, script_id, name, row_type, trigger_mode, record_schedule, section_id, indent_level, default_task_cycle, exec_max, show_enabled_toggle, default_enabled, task_tone, is_hidden, `data`, created_at, updated_at, deleted_at, is_deleted, `index`) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for t in chunk {
                query = query
                    .bind(t.id.to_string())
                    .bind(t.script_id.to_string())
                    .bind(&t.name)
                    .bind(t.row_type.clone())
                    .bind(t.trigger_mode.clone())
                    .bind(t.record_schedule)
                    .bind(t.section_id.as_ref().map(|value| value.to_string()))
                    .bind(t.indent_level as i64)
                    .bind(&t.default_task_cycle)
                    .bind(t.exec_max as i64)
                    .bind(t.show_enabled_toggle)
                    .bind(t.default_enabled)
                    .bind(t.task_tone.clone())
                    .bind(t.is_hidden)
                    .bind(&t.data)
                    .bind(t.created_at)
                    .bind(t.updated_at)
                    .bind(t.deleted_at)
                    .bind(t.is_deleted)
                    .bind(t.index as i64);
            }
            query
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("批量写入 script_tasks 失败: {}", e))?;
        }
    }

    Ok(())
}
