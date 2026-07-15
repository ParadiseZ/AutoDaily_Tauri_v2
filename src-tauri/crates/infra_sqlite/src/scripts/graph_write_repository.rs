use crate::get_pool;
use ad_kernel::ids::ScriptId;
/// 脚本及关联数据的批量插入工具
/// 用于统一 backend_download_script 和 clone_local_script_cmd 中的重复插入逻辑
use domain_script::{
    PolicyGroupPolicyLink, PolicyGroupProfile, PolicyProfile, PolicySetGroupLink, PolicySetProfile,
    ScriptProfile, ScriptTaskProfile,
};
use sqlx::{Sqlite, Transaction};

macro_rules! bind_chunk {
    ($query:expr, $p:expr) => {
        $query
            .bind($p.id.to_string())
            .bind($p.script_id.to_string())
            .bind($p.order_index)
            .bind(serde_json::to_value(&$p.info).map_err(|error| error.to_string())?)
    };
}

fn json_value<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|error| error.to_string())
}

fn enum_text(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(|error| error.to_string())?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "任务枚举序列化失败".to_string())
}
/// 在给定事务中批量插入脚本及其所有关联数据
/// 使用多行 INSERT 语法优化性能
pub async fn batch_insert_script_related(
    tx: &mut Transaction<'_, Sqlite>,
    script: &ScriptProfile,
    policies: &[PolicyProfile],
    policy_groups: &[PolicyGroupProfile],
    policy_sets: &[PolicySetProfile],
    group_policies: &[PolicyGroupPolicyLink],
    set_groups: &[PolicySetGroupLink],
    tasks: &[ScriptTaskProfile],
) -> Result<(), String> {
    // 1. Insert script
    sqlx::query(
        "INSERT INTO scripts (id, `data`) VALUES (?, ?)
         ON CONFLICT(id) DO UPDATE SET `data` = excluded.`data`",
    )
    .bind(script.id.to_string())
    .bind(json_value(&script.info)?)
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
                .map(|_| {
                    "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string()
                })
                .collect();
            let sql = format!(
                "INSERT INTO script_tasks (id, script_id, name, description, row_type, trigger_mode, record_schedule, section_id, indent_level, default_task_cycle, exec_max, show_enabled_toggle, default_enabled, task_tone, is_hidden, `data`, created_at, updated_at, deleted_at, is_deleted, `index`) VALUES {}",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for t in chunk {
                query = query
                    .bind(t.id.to_string())
                    .bind(t.script_id.to_string())
                    .bind(&t.name)
                    .bind(&t.description)
                    .bind(enum_text(&t.row_type)?)
                    .bind(enum_text(&t.trigger_mode)?)
                    .bind(t.record_schedule)
                    .bind(t.section_id.as_ref().map(|value| value.to_string()))
                    .bind(t.indent_level as i64)
                    .bind(json_value(&t.default_task_cycle)?)
                    .bind(t.exec_max as i64)
                    .bind(t.show_enabled_toggle)
                    .bind(t.default_enabled)
                    .bind(enum_text(&t.task_tone)?)
                    .bind(t.is_hidden)
                    .bind(json_value(&t.task)?)
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

pub async fn save_script_editor_graph(
    script: &ScriptProfile,
    policies: &[PolicyProfile],
    policy_groups: &[PolicyGroupProfile],
    policy_sets: &[PolicySetProfile],
    group_policies: &[PolicyGroupPolicyLink],
    set_groups: &[PolicySetGroupLink],
    tasks: &[ScriptTaskProfile],
) -> Result<(), String> {
    let script_id = script.id.to_string();
    let mut tx = get_pool()
        .begin()
        .await
        .map_err(|error| error.to_string())?;

    sqlx::query(
        "DELETE FROM group_policies WHERE group_id IN (SELECT id FROM policy_groups WHERE script_id = ?)",
    )
    .bind(&script_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| format!("清理 group_policies 失败: {error}"))?;

    sqlx::query(
        "DELETE FROM set_groups WHERE set_id IN (SELECT id FROM policy_sets WHERE script_id = ?)",
    )
    .bind(&script_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| format!("清理 set_groups 失败: {error}"))?;

    for table in ["script_tasks", "policies", "policy_groups", "policy_sets"] {
        sqlx::query(&format!("DELETE FROM {table} WHERE script_id = ?"))
            .bind(&script_id)
            .execute(&mut *tx)
            .await
            .map_err(|error| format!("清理 {table} 失败: {error}"))?;
    }

    batch_insert_script_related(
        &mut tx,
        script,
        policies,
        policy_groups,
        policy_sets,
        group_policies,
        set_groups,
        tasks,
    )
    .await?;
    tx.commit().await.map_err(|error| error.to_string())
}

pub async fn save_cloned_script_graph(
    replaced_script_id: Option<ScriptId>,
    script: &ScriptProfile,
    policies: &[PolicyProfile],
    policy_groups: &[PolicyGroupProfile],
    policy_sets: &[PolicySetProfile],
    group_policies: &[PolicyGroupPolicyLink],
    set_groups: &[PolicySetGroupLink],
    tasks: &[ScriptTaskProfile],
) -> Result<(), String> {
    let mut tx = get_pool()
        .begin()
        .await
        .map_err(|error| error.to_string())?;
    if let Some(script_id) = replaced_script_id {
        sqlx::query("DELETE FROM scripts WHERE id = ?")
            .bind(script_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|error| format!("删除被覆盖脚本失败: {error}"))?;
    }
    batch_insert_script_related(
        &mut tx,
        script,
        policies,
        policy_groups,
        policy_sets,
        group_policies,
        set_groups,
        tasks,
    )
    .await?;
    tx.commit().await.map_err(|error| error.to_string())
}

pub async fn delete_script_graph_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    script_id: ScriptId,
) -> Result<(), String> {
    let policy_group_ids =
        sqlx::query_scalar::<_, String>("SELECT id FROM policy_groups WHERE script_id = ?")
            .bind(script_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|error| format!("读取本地策略组失败: {error}"))?;
    for group_id in policy_group_ids {
        sqlx::query("DELETE FROM group_policies WHERE group_id = ?")
            .bind(group_id)
            .execute(&mut **tx)
            .await
            .map_err(|error| format!("删除本地策略组关联失败: {error}"))?;
    }

    let policy_set_ids =
        sqlx::query_scalar::<_, String>("SELECT id FROM policy_sets WHERE script_id = ?")
            .bind(script_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|error| format!("读取本地策略集失败: {error}"))?;
    for set_id in policy_set_ids {
        sqlx::query("DELETE FROM set_groups WHERE set_id = ?")
            .bind(set_id)
            .execute(&mut **tx)
            .await
            .map_err(|error| format!("删除本地策略集关联失败: {error}"))?;
    }

    for query in [
        "DELETE FROM policies WHERE script_id = ?",
        "DELETE FROM script_tasks WHERE script_id = ?",
        "DELETE FROM policy_groups WHERE script_id = ?",
        "DELETE FROM policy_sets WHERE script_id = ?",
        "DELETE FROM scripts WHERE id = ?",
    ] {
        sqlx::query(query)
            .bind(script_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|error| format!("删除本地云端脚本旧副本失败: {error}"))?;
    }
    Ok(())
}
