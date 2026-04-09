use crate::constant::table_name::RECOVERY_CHECKPOINT_TABLE;
use crate::domain::schedule::recovery_checkpoint::RecoveryCheckpointRow;
use crate::infrastructure::context::runtime_context::get_runtime_ctx;
use crate::infrastructure::db::get_pool;
use crate::infrastructure::ipc::message::{ResumeCheckpoint, ResumeMode, SessionCheckpointReason};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::session::runtime_session::{
    get_runtime_queue_item, get_script_bundle_snapshot, try_current_session_summary,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

async fn build_definition_fingerprint(
    script_id: crate::infrastructure::core::ScriptId,
) -> Result<String, String> {
    let mut hasher = DefaultHasher::new();
    let Some(bundle) = get_script_bundle_snapshot(script_id).await else {
        return Ok(script_id.to_string());
    };

    bundle.script_json.hash(&mut hasher);
    bundle.tasks_json.hash(&mut hasher);
    bundle.policies_json.hash(&mut hasher);
    bundle.policy_groups_json.hash(&mut hasher);
    bundle.policy_sets_json.hash(&mut hasher);
    bundle.group_policies_json.hash(&mut hasher);
    bundle.set_groups_json.hash(&mut hasher);

    Ok(format!("{:016x}", hasher.finish()))
}

pub async fn persist_checkpoint(checkpoint: &ResumeCheckpoint) -> Result<(), String> {
    sqlx::query(&format!(
        "INSERT INTO {} (
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
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(device_id) DO UPDATE SET
            execution_id = excluded.execution_id,
            source_session_id = excluded.source_session_id,
            run_target_json = excluded.run_target_json,
            assignment_id = excluded.assignment_id,
            script_id = excluded.script_id,
            time_template_id = excluded.time_template_id,
            account_id = excluded.account_id,
            task_id = excluded.task_id,
            step_id = excluded.step_id,
            resume_mode = excluded.resume_mode,
            definition_fingerprint = excluded.definition_fingerprint,
            updated_at = excluded.updated_at",
        RECOVERY_CHECKPOINT_TABLE
    ))
    .bind(checkpoint.execution_id.to_string())
    .bind(checkpoint.source_session_id.to_string())
    .bind(checkpoint.device_id.to_string())
    .bind(sqlx::types::Json(checkpoint.run_target.clone()))
    .bind(checkpoint.assignment_id.map(|id| id.to_string()))
    .bind(checkpoint.script_id.to_string())
    .bind(checkpoint.time_template_id.map(|id| id.to_string()))
    .bind(checkpoint.account_id.clone())
    .bind(checkpoint.task_id.map(|id| id.to_string()))
    .bind(checkpoint.step_id.map(|id| id.to_string()))
    .bind(sqlx::types::Json(checkpoint.resume_mode.clone()))
    .bind(&checkpoint.definition_fingerprint)
    .bind(&checkpoint.updated_at)
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    Ok(())
}

pub async fn prepare_and_persist_checkpoint(
    reason: SessionCheckpointReason,
) -> Result<Option<ResumeCheckpoint>, String> {
    let Some(summary) = try_current_session_summary() else {
        return Ok(None);
    };

    let (execution_id, assignment_id, script_id, task_id, step_id) = {
        let runtime_ctx = get_runtime_ctx();
        let mut ctx = runtime_ctx.write().await;
        if let Err(error) = ctx.vision_text_cache.flush_current_script() {
            Log::warn(&format!(
                "[ recovery ] checkpoint 前写回 OCR 文字缓存失败，已忽略: {}",
                error
            ));
        }

        (
            ctx.current_execution_id,
            ctx.current_assignment_id,
            ctx.script_id,
            ctx.current_task.as_ref().map(|task| task.id),
            ctx.current_step_id,
        )
    };

    let Some(execution_id) = execution_id else {
        Log::info(&format!(
            "[ recovery ] 收到 checkpoint 请求({:?})，但当前没有活动 execution，已跳过",
            reason
        ));
        return Ok(None);
    };

    let queue_item = match assignment_id {
        Some(current_assignment_id) => get_runtime_queue_item(current_assignment_id).await,
        None => None,
    };
    let checkpoint = ResumeCheckpoint {
        execution_id,
        source_session_id: summary.session_id,
        device_id: summary.device_id,
        run_target: summary.run_target,
        assignment_id,
        script_id,
        time_template_id: queue_item.as_ref().and_then(|item| item.time_template_id),
        account_id: queue_item.as_ref().and_then(|item| item.account_id.clone()),
        task_id,
        step_id,
        resume_mode: ResumeMode::FromTaskStart,
        definition_fingerprint: build_definition_fingerprint(script_id).await?,
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    persist_checkpoint(&checkpoint).await?;
    Log::info(&format!(
        "[ recovery ] 已保存 checkpoint，device={}, execution={}, reason={:?}",
        checkpoint.device_id, checkpoint.execution_id, reason
    ));
    Ok(Some(checkpoint))
}

pub async fn load_checkpoint_by_device(
    device_id: crate::infrastructure::core::DeviceId,
) -> Result<Option<ResumeCheckpoint>, String> {
    let query = format!(
        "SELECT execution_id, source_session_id, device_id, run_target_json, assignment_id, script_id, time_template_id, account_id, task_id, step_id, resume_mode, definition_fingerprint, updated_at
         FROM {}
         WHERE device_id = ?
         ORDER BY updated_at DESC
         LIMIT 1",
        RECOVERY_CHECKPOINT_TABLE
    );
    sqlx::query_as::<_, RecoveryCheckpointRow>(&query)
        .bind(device_id.to_string())
        .fetch_optional(get_pool())
        .await
        .map(|row| row.map(RecoveryCheckpointRow::into_checkpoint))
        .map_err(|error| error.to_string())
}
