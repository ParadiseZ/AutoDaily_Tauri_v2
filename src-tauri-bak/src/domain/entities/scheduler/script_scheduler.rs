use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::scheduler::scheduler_state::{SchedulerState, SchedulerConfig, SchedulerStatus};
use crate::domain::entities::scheduler::task_queue::ScheduledTask;
use crate::infrastructure::performance::ProcessManager;
use chrono::{DateTime, Local, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{info, warn, error};
use crate::domain::entities::scripts::script_info::{ScriptId, ScriptInfo, ScriptStatus};

/// 脚本调度引擎
/// 
/// 负责管理脚本的调度执行，支持：
/// - 基于时间段的脚本调度
/// - 脚本优先级管理
/// - 进程级别的脚本执行
/// - 主进程与设备进程的状态同步
#[derive(Debug)]
pub struct ScriptScheduler {
    /// 调度器状态
    state: Arc<RwLock<SchedulerState>>,
    /// 进程管理器
    process_manager: Arc<ProcessManager>,
    /// 设备进程映射 (device_id -> process_id)
    device_processes: Arc<RwLock<HashMap<String, String>>>,
    /// 脚本进程映射 (script_id -> (device_id, process_id))
    script_processes: Arc<RwLock<HashMap<ScriptId, (String, String)>>>,
    /// 调度任务运行标志
    running: Arc<Mutex<bool>>,
}

impl ScriptScheduler {
    /// 创建新的脚本调度器
    pub fn new(config: SchedulerConfig, process_manager: ProcessManager) -> Self {
        Self {
            state: Arc::new(RwLock::new(SchedulerState::new(config))),
            process_manager: Arc::new(process_manager),
            device_processes: Arc::new(RwLock::new(HashMap::new())),
            script_processes: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// 初始化调度器
    pub async fn initialize(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.initialize()?;
        info!("脚本调度器初始化完成");
        Ok(())
    }

    /// 启动调度器
    pub async fn start(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.start()?;
        drop(state);

        // 启动调度循环
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        // 启动调度任务
        self.start_scheduling_loop().await;
        
        info!("脚本调度器已启动");
        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> AppResult<()> {
        // 停止调度循环
        {
            let mut running = self.running.lock().await;
            *running = false;
        }

        // 停止所有正在运行的脚本
        self.stop_all_scripts().await?;

        // 停止调度器状态
        let mut state = self.state.write().await;
        state.stop()?;

        info!("脚本调度器已停止");
        Ok(())
    }

    /// 暂停调度器
    pub async fn pause(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.pause()?;
        info!("脚本调度器已暂停");
        Ok(())
    }

    /// 恢复调度器
    pub async fn resume(&self) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.start()?;
        info!("脚本调度器已恢复");
        Ok(())
    }

    /// 注册脚本
    pub async fn register_script(&self, script: ScriptInfo) -> AppResult<()> {
        let mut state = self.state.write().await;
        let script_id = script.id.clone();
        state.register_script(script)?;
        
        info!("脚本已注册: {}", script_id);
        Ok(())
    }

    /// 取消注册脚本
    pub async fn unregister_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 先停止脚本执行
        self.stop_script(script_id).await?;
        
        // 从状态中移除
        let mut state = self.state.write().await;
        state.unregister_script(script_id)?;
        
        info!("脚本已取消注册: {}", script_id);
        Ok(())
    }

    /// 启动脚本执行
    pub async fn start_script(&self, script_id: &ScriptId, device_id: Option<String>) -> AppResult<()> {
        let mut state = self.state.write().await;
        
        // 检查脚本是否存在
        if !state.get_scripts().contains_key(script_id) {
            return Err(AppError::ConfigError(format!("脚本不存在: {}", script_id)));
        }

        // 检查脚本是否已在运行
        if state.is_script_running(script_id) {
            return Err(AppError::ConfigError(format!("脚本已在运行: {}", script_id)));
        }

        // 更新脚本状态为启动中
        state.update_script_status(script_id, ScriptStatus::Starting)?;
        
        // 调度脚本执行
        state.schedule_script_execution(script_id)?;
        
        info!("脚本执行已启动: script_id={}, device_id={:?}", script_id, device_id);
        Ok(())
    }

    /// 停止脚本执行
    pub async fn stop_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 更新脚本状态
        {
            let mut state = self.state.write().await;
            state.update_script_status(script_id, ScriptStatus::Stopping)?;
        }

        // 终止相关进程
        {
            let mut script_processes = self.script_processes.write().await;
            if let Some((device_id, process_id)) = script_processes.remove(script_id) {
                if let Err(e) = self.process_manager.terminate_process(&process_id) {
                    warn!("终止脚本进程失败: script_id={}, process_id={}, error={}", 
                          script_id, process_id, e);
                }
                info!("脚本进程已终止: script_id={}, device_id={}, process_id={}", 
                      script_id, device_id, process_id);
            }
        }

        // 更新最终状态
        {
            let mut state = self.state.write().await;
            state.update_script_status(script_id, ScriptStatus::Stopped)?;
        }

        info!("脚本执行已停止: {}", script_id);
        Ok(())
    }

    /// 暂停脚本执行
    pub async fn pause_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.update_script_status(script_id, ScriptStatus::Paused)?;
        
        info!("脚本执行已暂停: {}", script_id);
        Ok(())
    }

    /// 恢复脚本执行
    pub async fn resume_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let mut state = self.state.write().await;
        state.update_script_status(script_id, ScriptStatus::Running)?;
        
        info!("脚本执行已恢复: {}", script_id);
        Ok(())
    }

    /// 获取脚本状态
    pub async fn get_script_status(&self, script_id: &ScriptId) -> Option<ScriptStatus> {
        let state = self.state.read().await;
        state.get_script_status(script_id)
    }

    /// 获取所有脚本状态
    pub async fn get_all_script_status(&self) -> HashMap<ScriptId, ScriptStatus> {
        let state = self.state.read().await;
        state.get_scripts()
            .iter()
            .map(|(id, script)| (id.clone(), script.status.clone()))
            .collect()
    }

    /// 获取调度器状态
    pub async fn get_scheduler_status(&self) -> SchedulerStatus {
        let state = self.state.read().await;
        state.get_status().clone()
    }

    /// 获取调度器统计信息
    pub async fn get_scheduler_stats(&self) -> crate::domain::entities::scheduler::scheduler_state::SchedulerStats {
        let state = self.state.read().await;
        state.get_stats().clone()
    }

    /// 更新活动时间（由主界面调用）
    pub async fn update_activity(&self) {
        let mut state = self.state.write().await;
        state.update_activity();
    }

    /// 注册设备进程
    pub async fn register_device_process(&self, device_id: String, process_id: String) {
        let mut device_processes = self.device_processes.write().await;
        device_processes.insert(device_id.clone(), process_id.clone());
        info!("设备进程已注册: device_id={}, process_id={}", device_id, process_id);
    }

    /// 取消注册设备进程
    pub async fn unregister_device_process(&self, device_id: &str) {
        let mut device_processes = self.device_processes.write().await;
        if let Some(process_id) = device_processes.remove(device_id) {
            // 终止进程
            if let Err(e) = self.process_manager.terminate_process(&process_id) {
                warn!("终止设备进程失败: device_id={}, process_id={}, error={}", 
                      device_id, process_id, e);
            }
            info!("设备进程已取消注册: device_id={}, process_id={}", device_id, process_id);
        }
    }

    /// 获取调度器概览
    pub async fn get_overview(&self) -> String {
        let state = self.state.read().await;
        let device_count = self.device_processes.read().await.len();
        let script_count = self.script_processes.read().await.len();
        
        format!(
            "ScriptScheduler[{}, 设备进程:{}, 脚本进程:{}]",
            state.get_overview(),
            device_count,
            script_count
        )
    }

    /// 启动调度循环
    async fn start_scheduling_loop(&self) {
        let state = self.state.clone();
        let process_manager = self.process_manager.clone();
        let script_processes = self.script_processes.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let check_interval = {
                let state_guard = state.read().await;
                state_guard.get_config().check_interval_seconds
            };
            
            let mut interval = interval(TokioDuration::from_secs(check_interval));

            loop {
                interval.tick().await;

                // 检查是否应该继续运行
                {
                    let running_guard = running.lock().await;
                    if !*running_guard {
                        break;
                    }
                }

                // 检查调度器状态
                let scheduler_status = {
                    let state_guard = state.read().await;
                    state_guard.get_status().clone()
                };

                if scheduler_status != SchedulerStatus::Running {
                    continue;
                }

                // 获取下一个待执行任务
                let next_task = {
                    let mut state_guard = state.write().await;
                    state_guard.get_next_task()
                };

                if let Some(task) = next_task {
                    if let Err(e) = Self::execute_task_static(
                        &state,
                        &process_manager,
                        &script_processes,
                        task,
                    ).await {
                        error!("执行任务失败: {}", e);
                    }
                }
            }

            info!("调度循环已退出");
        });
    }

    /// 执行任务（静态方法，用于异步任务）
    async fn execute_task_static(
        state: &Arc<RwLock<SchedulerState>>,
        process_manager: &Arc<ProcessManager>,
        script_processes: &Arc<RwLock<HashMap<ScriptId, (String, String)>>>,
        task: ScheduledTask,
    ) -> AppResult<()> {
        let script_id = task.script_id.clone();
        
        info!("开始执行任务: {}", script_id);

        // 这里应该实现实际的脚本执行逻辑
        // 例如：启动设备进程执行脚本
        
        // 模拟脚本执行
        let start_time = Local::now();
        
        // 标记任务开始
        {
            let mut state_guard = state.write().await;
            state_guard.mark_task_started(&task, None, None)?;
        }

        // 模拟执行时间
        tokio::time::sleep(TokioDuration::from_millis(task.estimated_duration_ms)).await;

        // 计算执行时间
        let execution_time = Local::now().signed_duration_since(start_time);
        let execution_time_ms = execution_time.num_milliseconds() as u64;

        // 标记任务完成
        {
            let mut state_guard = state.write().await;
            state_guard.mark_task_completed(&script_id, true, execution_time_ms)?;
        }

        info!("任务执行完成: script_id={}, duration={}ms", script_id, execution_time_ms);
        Ok(())
    }

    /// 停止所有脚本
    async fn stop_all_scripts(&self) -> AppResult<()> {
        let script_ids: Vec<ScriptId> = {
            let state = self.state.read().await;
            state.get_executing_scripts().keys().cloned().collect()
        };

        for script_id in script_ids {
            if let Err(e) = self.stop_script(&script_id).await {
                warn!("停止脚本失败: script_id={}, error={}", script_id, e);
            }
        }

        Ok(())
    }
}

impl Drop for ScriptScheduler {
    fn drop(&mut self) {
        // 确保清理所有资源
        info!("脚本调度器正在销毁");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_scheduler_lifecycle() {
        let config = SchedulerConfig::default();
        let process_manager = ProcessManager::new();
        let scheduler = ScriptScheduler::new(config, process_manager);

        // 初始化
        assert!(scheduler.initialize().await.is_ok());

        // 启动
        assert!(scheduler.start().await.is_ok());
        assert_eq!(scheduler.get_scheduler_status().await, SchedulerStatus::Running);

        // 停止
        assert!(scheduler.stop().await.is_ok());
        assert_eq!(scheduler.get_scheduler_status().await, SchedulerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_script_management() {
        let config = SchedulerConfig::default();
        let process_manager = ProcessManager::new();
        let scheduler = ScriptScheduler::new(config, process_manager);

        assert!(scheduler.initialize().await.is_ok());

        // 注册脚本
        let script = ScriptInfo::new(
            "test_script".to_string(),
            "Test Script".to_string(),
            "Test Description".to_string(),
            "/path/to/script".to_string(),
        );

        assert!(scheduler.register_script(script).await.is_ok());

        // 检查脚本状态
        let status = scheduler.get_script_status("test_script".into()).await;
        assert!(status.is_some());
        assert_eq!(status.unwrap(), ScriptStatus::Stopped);

        // 取消注册脚本
        assert!(scheduler.unregister_script("test_script".into()).await.is_ok());
        assert!(scheduler.get_script_status("test_script".int()).await.is_none());
    }
}
