use crate::domain::entities::scheduler::task_queue::{TaskQueue, ScheduledTask, TaskQueueStatus};
use chrono::{DateTime, Local, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::scripts::script_info::{ScriptId, ScriptInfo, ScriptStatus};

/// 调度器状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerStatus {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 停止
    Stopped,
    /// 错误状态
    Error(String),
}

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 任务检查间隔（秒）
    pub check_interval_seconds: u64,
    /// 任务超时时间（秒）
    pub default_task_timeout_seconds: u64,
    /// 是否启用自动重试
    pub enable_auto_retry: bool,
    /// 默认重试次数
    pub default_retry_count: u32,
    /// 是否启用空闲检测
    pub enable_idle_detection: bool,
    /// 空闲检测阈值（秒）
    pub idle_threshold_seconds: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            check_interval_seconds: 30,
            default_task_timeout_seconds: 300,
            enable_auto_retry: true,
            default_retry_count: 3,
            enable_idle_detection: true,
            idle_threshold_seconds: 300,
        }
    }
}

/// 调度器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// 调度器启动时间
    pub started_at: Option<DateTime<Local>>,
    /// 总调度任务数
    pub total_scheduled_tasks: u64,
    /// 成功执行任务数
    pub successful_tasks: u64,
    /// 失败任务数
    pub failed_tasks: u64,
    /// 平均任务执行时间（毫秒）
    pub average_execution_time_ms: u64,
    /// 当前运行时长（秒）
    pub uptime_seconds: u64,
    /// 系统负载（0-100%）
    pub system_load_percent: u8,
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self {
            started_at: None,
            total_scheduled_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            average_execution_time_ms: 0,
            uptime_seconds: 0,
            system_load_percent: 0,
        }
    }
}

/// 脚本执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionContext {
    /// 脚本ID
    pub script_id: ScriptId,
    /// 执行开始时间
    pub started_at: DateTime<Local>,
    /// 预期执行时长（毫秒）
    pub expected_duration_ms: u64,
    /// 进程ID（如果以进程方式执行）
    pub process_id: Option<String>,
    /// 设备ID
    pub device_id: Option<String>,
    /// 执行参数
    pub execution_params: HashMap<String, serde_json::Value>,
}

/// 调度器状态管理器
#[derive(Debug)]
pub struct SchedulerState {
    /// 当前状态
    status: SchedulerStatus,
    /// 配置
    config: SchedulerConfig,
    /// 任务队列
    task_queue: TaskQueue,
    /// 脚本注册表
    scripts: HashMap<ScriptId, ScriptInfo>,
    /// 正在执行的脚本上下文
    executing_scripts: HashMap<ScriptId, ScriptExecutionContext>,
    /// 统计信息
    stats: SchedulerStats,
    /// 最后检查时间
    last_check_time: DateTime<Local>,
    /// 最后活动时间（用于空闲检测）
    last_activity_time: DateTime<Local>,
}

impl SchedulerState {
    /// 创建新的调度器状态管理器
    pub fn new(config: SchedulerConfig) -> Self {
        let now = Local::now();
        Self {
            status: SchedulerStatus::Stopped,
            task_queue: TaskQueue::new(config.max_concurrent_tasks),
            config,
            scripts: HashMap::new(),
            executing_scripts: HashMap::new(),
            stats: SchedulerStats::default(),
            last_check_time: now,
            last_activity_time: now,
        }
    }

    /// 初始化调度器
    pub fn initialize(&mut self) -> AppResult<()> {
        self.status = SchedulerStatus::Initializing;
        tracing::info!("调度器初始化开始");

        // 验证配置
        if self.config.max_concurrent_tasks == 0 {
            return Err(AppError::ConfigError("最大并发任务数不能为0".to_string()));
        }

        // 清理状态
        self.executing_scripts.clear();
        self.task_queue.clear();

        self.status = SchedulerStatus::Stopped;
        tracing::info!("调度器初始化完成");
        Ok(())
    }

    /// 启动调度器
    pub fn start(&mut self) -> AppResult<()> {
        match self.status {
            SchedulerStatus::Stopped | SchedulerStatus::Paused => {
                self.status = SchedulerStatus::Running;
                self.task_queue.start();
                self.stats.started_at = Some(Local::now());
                tracing::info!("调度器已启动");
                Ok(())
            }
            SchedulerStatus::Running => {
                tracing::warn!("调度器已在运行中");
                Ok(())
            }
            _ => Err(AppError::ConfigError(format!(
                "无法从状态 {:?} 启动调度器",
                self.status
            ))),
        }
    }

    /// 暂停调度器
    pub fn pause(&mut self) -> AppResult<()> {
        match self.status {
            SchedulerStatus::Running => {
                self.status = SchedulerStatus::Paused;
                self.task_queue.pause();
                tracing::info!("调度器已暂停");
                Ok(())
            }
            _ => Err(AppError::ConfigError(format!(
                "无法从状态 {:?} 暂停调度器",
                self.status
            ))),
        }
    }

    /// 停止调度器
    pub fn stop(&mut self) -> AppResult<()> {
        self.status = SchedulerStatus::Stopped;
        self.task_queue.stop();
        
        // 记录停止前的统计信息
        if let Some(started_at) = self.stats.started_at {
            let uptime = Local::now().signed_duration_since(started_at);
            self.stats.uptime_seconds = uptime.num_seconds() as u64;
        }
        
        tracing::info!("调度器已停止");
        Ok(())
    }

    /// 注册脚本
    pub fn register_script(&mut self, script: ScriptInfo) -> AppResult<()> {
        let script_id = script.id.clone();
        
        // 检查脚本是否已存在
        if self.scripts.contains_key(&script_id) {
            return Err(AppError::ConfigError(format!(
                "脚本已存在: {}",
                script_id
            )));
        }

        tracing::info!("注册脚本: {}", script_id);
        self.scripts.insert(script_id, script);
        Ok(())
    }

    /// 取消注册脚本
    pub fn unregister_script(&mut self, script_id: &ScriptId) -> AppResult<()> {
        // 取消所有相关任务
        self.task_queue.cancel_script_tasks(script_id);
        
        // 如果脚本正在执行，标记为停止状态
        if let Some(script) = self.scripts.get_mut(script_id) {
            script.update_status(ScriptStatus::Stopped);
        }
        
        // 移除执行上下文
        self.executing_scripts.remove(script_id);
        
        // 移除脚本
        self.scripts.remove(script_id);
        
        tracing::info!("取消注册脚本: {}", script_id);
        Ok(())
    }

    /// 更新脚本状态
    pub fn update_script_status(&mut self, script_id: &ScriptId, status: ScriptStatus) -> AppResult<()> {
        if let Some(script) = self.scripts.get_mut(script_id) {
            let old_status = script.status.clone();
            script.update_status(status.clone());
            
            tracing::info!("脚本状态更新: {} {} -> {:?}", script_id, format!("{:?}", old_status), status);
            
            // 根据状态变化处理任务
            match status {
                ScriptStatus::Stopped => {
                    // 取消所有待执行任务
                    self.task_queue.cancel_script_tasks(script_id);
                    self.executing_scripts.remove(script_id);
                }
                ScriptStatus::Running => {
                    // 开始执行脚本
                    self.schedule_script_execution(script_id)?;
                }
                ScriptStatus::Error(_) => {
                    // 标记任务失败
                    self.task_queue.mark_task_completed(script_id, false);
                    self.executing_scripts.remove(script_id);
                }
                _ => {}
            }
            
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("脚本不存在: {}", script_id)))
        }
    }

    /// 获取脚本状态
    pub fn get_script_status(&self, script_id: &ScriptId) -> Option<ScriptStatus> {
        self.scripts.get(script_id).map(|script| script.status.clone())
    }

    /// 调度脚本执行
    pub fn schedule_script_execution(&mut self, script_id: &ScriptId) -> AppResult<()> {
        let script = self.scripts.get(script_id)
            .ok_or_else(|| AppError::ConfigError(format!("脚本不存在: {}", script_id)))?;

        // 检查脚本是否可以执行
        script.can_execute()?;

        let now = Local::now();
        let current_time = now.time();

        // 检查调度时间
        if !script.should_run_at_time(current_time) {
            return Err(AppError::ConfigError(format!(
                "脚本 {} 当前时间不在执行时间范围内",
                script_id
            )));
        }

        // 检查是否达到最大执行次数
        if script.has_reached_max_executions() {
            return Err(AppError::ConfigError(format!(
                "脚本 {} 已达到最大执行次数",
                script_id
            )));
        }

        // 创建调度任务
        let task = ScheduledTask::new(
            script_id.clone(),
            now,
            script.priority.clone(),
            script.config.resource_requirements.estimated_duration_seconds * 1000,
            script.config.retry_count,
        );

        self.task_queue.enqueue_task(task);
        self.stats.total_scheduled_tasks += 1;

        tracing::info!("脚本已加入执行队列: {}", script_id);
        Ok(())
    }

    /// 获取下一个待执行任务
    pub fn get_next_task(&mut self) -> Option<ScheduledTask> {
        let current_time = Local::now();
        self.last_check_time = current_time;
        self.task_queue.get_next_task(current_time)
    }

    /// 标记任务开始执行
    pub fn mark_task_started(&mut self, task: &ScheduledTask, process_id: Option<String>, device_id: Option<String>) -> AppResult<()> {
        // 更新脚本状态
        if let Some(script) = self.scripts.get_mut(&task.script_id) {
            script.update_status(ScriptStatus::Running);
        }

        // 创建执行上下文
        let context = ScriptExecutionContext {
            script_id: task.script_id.clone(),
            started_at: Local::now(),
            expected_duration_ms: task.estimated_duration_ms,
            process_id,
            device_id,
            execution_params: HashMap::new(),
        };

        self.executing_scripts.insert(task.script_id.clone(), context);
        self.task_queue.mark_task_running(task.clone());
        
        tracing::info!("任务开始执行: {}", task.script_id);
        Ok(())
    }

    /// 标记任务完成
    pub fn mark_task_completed(&mut self, script_id: &ScriptId, success: bool, execution_time_ms: u64) -> AppResult<()> {
        // 更新统计信息
        if success {
            self.stats.successful_tasks += 1;
        } else {
            self.stats.failed_tasks += 1;
        }

        // 更新平均执行时间
        let total_tasks = self.stats.successful_tasks + self.stats.failed_tasks;
        if total_tasks > 0 {
            let total_time = self.stats.average_execution_time_ms * (total_tasks - 1) + execution_time_ms;
            self.stats.average_execution_time_ms = total_time / total_tasks;
        }

        // 更新脚本状态和统计
        if let Some(script) = self.scripts.get_mut(script_id) {
            let result = if success {
                crate::domain::entities::scripts::script_info::ScriptExecutionResult::Success
            } else {
                crate::domain::entities::scripts::script_info::ScriptExecutionResult::Failed("执行失败".to_string())
            };
            
            script.record_execution(result, execution_time_ms);
            script.update_status(ScriptStatus::Stopped);
        }

        // 清理执行上下文
        self.executing_scripts.remove(script_id);
        self.task_queue.mark_task_completed(script_id, success);

        tracing::info!("任务执行完成: script_id={}, success={}, duration={}ms", 
                      script_id, success, execution_time_ms);
        Ok(())
    }

    /// 检查是否处于空闲状态
    pub fn is_idle(&self) -> bool {
        if !self.config.enable_idle_detection {
            return false;
        }

        let idle_duration = Local::now().signed_duration_since(self.last_activity_time);
        idle_duration.num_seconds() as u64 >= self.config.idle_threshold_seconds
    }

    /// 更新活动时间
    pub fn update_activity(&mut self) {
        self.last_activity_time = Local::now();
    }

    /// 获取调度器状态
    pub fn get_status(&self) -> &SchedulerStatus {
        &self.status
    }

    /// 获取配置
    pub fn get_config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &SchedulerStats {
        &self.stats
    }

    /// 获取所有脚本
    pub fn get_scripts(&self) -> &HashMap<ScriptId, ScriptInfo> {
        &self.scripts
    }

    /// 获取正在执行的脚本
    pub fn get_executing_scripts(&self) -> &HashMap<ScriptId, ScriptExecutionContext> {
        &self.executing_scripts
    }

    /// 获取任务队列状态
    pub fn get_task_queue_stats(&self) -> &crate::domain::entities::scheduler::task_queue::TaskQueueStats {
        self.task_queue.get_stats()
    }

    /// 检查脚本是否正在运行
    pub fn is_script_running(&self, script_id: &ScriptId) -> bool {
        self.executing_scripts.contains_key(script_id) || 
        self.task_queue.is_script_running(script_id)
    }

    /// 获取调度器概览信息
    pub fn get_overview(&self) -> String {
        format!(
            "Scheduler[状态:{:?}, 脚本数:{}, 执行中:{}, 队列:{}]",
            self.status,
            self.scripts.len(),
            self.executing_scripts.len(),
            self.task_queue.get_queue_overview()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_state_initialization() {
        let config = SchedulerConfig::default();
        let mut scheduler = SchedulerState::new(config);
        
        assert!(matches!(scheduler.status, SchedulerStatus::Stopped));
        assert!(scheduler.initialize().is_ok());
        assert!(matches!(scheduler.status, SchedulerStatus::Stopped));
    }

    #[test]
    fn test_script_registration() {
        let config = SchedulerConfig::default();
        let mut scheduler = SchedulerState::new(config);
        
        let script = ScriptInfo::new(
            "test_script".to_string(),
            "Test Script".to_string(),
            "Test Description".to_string(),
            "/path/to/script".to_string(),
        );
        
        assert!(scheduler.register_script(script).is_ok());
        assert!(scheduler.scripts.contains_key("test_script"));
        
        // 测试重复注册
        let duplicate_script = ScriptInfo::new(
            "test_script".to_string(),
            "Duplicate".to_string(),
            "Duplicate".to_string(),
            "/duplicate".to_string(),
        );
        assert!(scheduler.register_script(duplicate_script).is_err());
    }

    #[test]
    fn test_scheduler_lifecycle() {
        let config = SchedulerConfig::default();
        let mut scheduler = SchedulerState::new(config);
        
        // 初始化
        assert!(scheduler.initialize().is_ok());
        
        // 启动
        assert!(scheduler.start().is_ok());
        assert!(matches!(scheduler.get_status(), SchedulerStatus::Running));
        
        // 暂停
        assert!(scheduler.pause().is_ok());
        assert!(matches!(scheduler.get_status(), SchedulerStatus::Paused));
        
        // 停止
        assert!(scheduler.stop().is_ok());
        assert!(matches!(scheduler.get_status(), SchedulerStatus::Stopped));
    }
}
