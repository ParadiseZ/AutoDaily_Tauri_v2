// 脚本调度器
// 管理子进程中的脚本执行队列，按顺序执行脚本

use crate::infrastructure::context::runtime_context::get_runtime_ctx;
use crate::constant::table_name::SCRIPT_TABLE;
use crate::domain::scripts::script_info::ScriptTable;
use crate::infrastructure::core::ScriptId;
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::ipc::message::{RunTarget, RuntimeQueueItem, RuntimeSessionSnapshot};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::scripts::executor::ScriptExecutor;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// 脚本调度器
pub struct ScriptScheduler {
    /// 待执行的脚本队列
    queue: Arc<RwLock<VecDeque<RuntimeQueueItem>>>,
    /// 当前正在执行的脚本
    current_script: Arc<RwLock<Option<ScriptId>>>,
    /// 取消令牌
    cancel_token: CancellationToken,
}

/// 全局调度器
static SCHEDULER: std::sync::OnceLock<Arc<ScriptScheduler>> = std::sync::OnceLock::new();

pub fn init_scheduler(cancel_token: CancellationToken) -> Arc<ScriptScheduler> {
    let scheduler = Arc::new(ScriptScheduler {
        queue: Arc::new(RwLock::new(VecDeque::new())),
        current_script: Arc::new(RwLock::new(None)),
        cancel_token,
    });
    let _ = SCHEDULER.set(scheduler.clone());
    scheduler
}

pub fn get_scheduler() -> Option<Arc<ScriptScheduler>> {
    SCHEDULER.get().cloned()
}

impl ScriptScheduler {
    /// 用完整 session 替换当前队列
    pub async fn load_session(&self, session: RuntimeSessionSnapshot) {
        let mut queue = self.queue.write().await;
        queue.clear();
        queue.extend(session.queue);
        Log::info(&format!(
            "[ scheduler ] 已加载 session[{}]，队列长度: {}",
            session.session_id,
            queue.len()
        ));
    }

    /// 获取当前正在执行的脚本
    pub async fn current_script(&self) -> Option<ScriptId> {
        *self.current_script.read().await
    }

    /// 获取队列长度
    pub async fn queue_len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// 调度循环 — 在 Running 状态下被 main_child 调用
    /// 从队列取出脚本执行，执行完后取下一个
    /// 返回 true 表示还有任务可执行，false 表示队列为空
    pub async fn tick(&self) -> bool {
        // 检查取消
        if self.cancel_token.is_cancelled() {
            return false;
        }

        // 从队列取出下一个脚本
        let queue_item = {
            let mut queue = self.queue.write().await;
            queue.pop_front()
        };

        let queue_item = match queue_item {
            Some(item) => item,
            None => return false, // 队列为空
        };
        let script_id = queue_item.script_id;

        // 标记当前脚本
        *self.current_script.write().await = Some(script_id);
        Log::info(&format!("[ scheduler ] 开始执行脚本: {}", script_id));

        // 执行脚本
        let result = self.execute_script(queue_item).await;

        // 清除当前脚本
        *self.current_script.write().await = None;

        match result {
            Ok(()) => {
                Log::info(&format!("[ scheduler ] 脚本[{}]执行完成", script_id));
            }
            Err(e) => {
                Log::error(&format!("[ scheduler ] 脚本[{}]执行失败: {}", script_id, e));
            }
        }

        // 还有更多脚本？
        self.queue.read().await.len() > 0
    }

    /// 执行单个脚本
    async fn execute_script(&self, queue_item: RuntimeQueueItem) -> Result<(), String> {
        let script_id = queue_item.script_id;
        let runtime_ctx = get_runtime_ctx();
        let script_table: ScriptTable = DbRepo::get_by_id(SCRIPT_TABLE, &script_id.to_string())
            .await?
            .ok_or_else(|| format!("脚本[{}]不存在", script_id))?;
        let script_info = script_table.data.0;
        let script_name = script_info.name.clone();

        // 更新运行时上下文的 script_id
        {
            let mut ctx = runtime_ctx.write().await;
            ctx.script_id = script_id;
            ctx.target = RunTarget::DeviceQueue;
            ctx.script_info = Some(script_info);
            ctx.current_task = None;
            ctx.last_snapshot = None;
            ctx.last_hits.clear();
            if let Err(error) = ctx.vision_text_cache.load_for_script(script_id, &script_name) {
                Log::warn(&format!(
                    "[ scheduler ] 脚本[{}]加载 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        // 创建执行器
        let _executor = ScriptExecutor::new(runtime_ctx.clone());

        // TODO: 从数据库加载脚本的任务列表（ScriptTaskTable）
        // TODO: 解析节点/边为 Step 执行序列
        // TODO: 加载脚本参数到 runtime_ctx
        // 目前使用占位逻辑：
        Log::info(&format!(
            "[ scheduler ] 脚本[{}]加载中... assignment: {} (TODO: 从数据库加载任务)",
            script_id, queue_item.assignment_id
        ));

        // 占位：执行空步骤列表（后续接入实际加载逻辑）
        // let steps = load_script_steps(script_id).await?;
        // executor.execute(&steps).await.map_err(|e| e.to_string())?;
        {
            let mut ctx = runtime_ctx.write().await;
            if let Err(error) = ctx.vision_text_cache.flush_current_script() {
                Log::warn(&format!(
                    "[ scheduler ] 脚本[{}]写回 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        Ok(())
    }

    /// 开发者调试执行 — 直接执行指定目标，不走队列
    pub async fn debug_execute(
        &self,
        target: RunTarget,
    ) -> Result<(), String> {
        let script_id = target
            .script_id()
            .ok_or_else(|| "调试运行目标必须携带 script_id".to_string())?;
        Log::info(&format!(
            "[ scheduler ] 调试执行脚本[{}] target: {:?}",
            script_id, target
        ));

        let runtime_ctx = get_runtime_ctx();
        let script_table: ScriptTable = DbRepo::get_by_id(SCRIPT_TABLE, &script_id.to_string())
            .await?
            .ok_or_else(|| format!("脚本[{}]不存在", script_id))?;
        let script_info = script_table.data.0;
        let script_name = script_info.name.clone();
        {
            let mut ctx = runtime_ctx.write().await;
            ctx.script_id = script_id;
            ctx.target = target.clone();
            ctx.script_info = Some(script_info);
            ctx.current_task = None;
            ctx.last_snapshot = None;
            ctx.last_hits.clear();
            if let Err(error) = ctx.vision_text_cache.load_for_script(script_id, &script_name) {
                Log::warn(&format!(
                    "[ scheduler ] 调试执行脚本[{}]加载 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        let _executor = ScriptExecutor::new(runtime_ctx.clone());

        // TODO: 根据 target 加载对应的步骤
        // match target {
        //     RunTarget::FullScript { script_id } => { ... }
        //     RunTarget::Task { script_id, task_id } => { ... }
        //     RunTarget::PolicyGroup { script_id, policy_group_id } => { ... }
        //     RunTarget::PolicySet { script_id, policy_set_id } => { ... }
        // }

        Log::info(&format!(
            "[ scheduler ] 调试执行脚本[{}]完成 (TODO: 实际执行逻辑)",
            script_id
        ));

        {
            let mut ctx = runtime_ctx.write().await;
            if let Err(error) = ctx.vision_text_cache.flush_current_script() {
                Log::warn(&format!(
                    "[ scheduler ] 调试执行脚本[{}]写回 OCR 文字缓存失败，已忽略: {}",
                    script_id, error
                ));
            }
        }

        Ok(())
    }

    /// 清空队列
    pub async fn clear_queue(&self) {
        self.queue.write().await.clear();
        Log::info("[ scheduler ] 队列已清空");
    }

    /// 清空当前会话
    pub async fn clear_session(&self) {
        self.clear_queue().await;
        *self.current_script.write().await = None;
        Log::info("[ scheduler ] 当前 session 已清空");
    }
}
