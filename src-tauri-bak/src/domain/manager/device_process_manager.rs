use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::devices::script::{Script, ScriptId, ScriptStatus};
use crate::domain::entities::scheduler::script_scheduler::ScriptScheduler;
use crate::domain::entities::scheduler::scheduler_state::SchedulerConfig;
use crate::domain::manager::main_process_manager::{DeviceCommand, DeviceStatusReport};
use crate::infrastructure::performance::ProcessManager;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{info, warn, error};

/// 设备进程管理器 - 运行在设备进程中
/// 
/// 每个设备进程包含：
/// - 独立的调度引擎
/// - 分配给该设备的脚本列表
/// - 同时只能运行一个脚本的限制
/// - 与主进程的IPC通信
#[derive(Debug)]
pub struct DeviceProcessManager {
    /// 设备ID
    device_id: String,
    /// 设备调度器（每个设备进程独立的调度引擎）
    scheduler: Arc<RwLock<ScriptScheduler>>,
    /// 分配到此设备的脚本 (script_id -> Script)
    device_scripts: Arc<RwLock<HashMap<ScriptId, Script>>>,
    /// 当前运行的脚本ID（单设备同时只能运行一个脚本）
    current_running_script: Arc<RwLock<Option<ScriptId>>>,
    /// 与主进程的通信通道
    main_process_sender: Option<mpsc::Sender<DeviceStatusReport>>,
    /// 设备进程管理器是否运行中
    is_running: Arc<RwLock<bool>>,
}

/// 设备进程配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProcessConfig {
    /// 设备ID
    pub device_id: String,
    /// 调度器配置
    pub scheduler_config: SchedulerConfig,
    /// 心跳间隔（秒）
    pub heartbeat_interval_seconds: u64,
    /// 状态报告间隔（秒）
    pub status_report_interval_seconds: u64,
}

impl Default for DeviceProcessConfig {
    fn default() -> Self {
        Self {
            device_id: "device_0".to_string(),
            scheduler_config: SchedulerConfig {
                max_concurrent_tasks: 1, // 设备同时只能运行一个脚本
                check_interval_seconds: 10,
                default_task_timeout_seconds: 300,
                enable_auto_retry: true,
                default_retry_count: 3,
                enable_idle_detection: true,
                idle_threshold_seconds: 300,
            },
            heartbeat_interval_seconds: 30,
            status_report_interval_seconds: 60,
        }
    }
}

impl DeviceProcessManager {
    /// 创建设备进程管理器
    pub fn new(config: DeviceProcessConfig, process_manager: ProcessManager) -> Self {
        let scheduler = ScriptScheduler::new(config.scheduler_config.clone(), process_manager);

        Self {
            device_id: config.device_id.clone(),
            scheduler: Arc::new(RwLock::new(scheduler)),
            device_scripts: Arc::new(RwLock::new(HashMap::new())),
            current_running_script: Arc::new(RwLock::new(None)),
            main_process_sender: None,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 初始化设备进程管理器
    pub async fn initialize(&mut self, main_process_sender: mpsc::Sender<DeviceStatusReport>) -> AppResult<()> {
        // 初始化调度器
        let scheduler = self.scheduler.read().await;
        scheduler.initialize().await?;
        drop(scheduler);

        // 设置与主进程的通信通道
        self.main_process_sender = Some(main_process_sender);

        info!("设备进程管理器初始化完成: device_id={}", self.device_id);
        Ok(())
    }

    /// 启动设备进程管理器
    pub async fn start(&self) -> AppResult<()> {
        // 标记为运行中
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        // 启动调度器
        let scheduler = self.scheduler.read().await;
        scheduler.start().await?;
        drop(scheduler);

        // 启动心跳和状态报告循环
        self.start_heartbeat_loop().await;
        self.start_status_report_loop().await;

        // 向主进程报告设备状态
        self.report_device_status(crate::domain::entities::devices::device_context::DeviceStatus::Idle).await?;

        info!("设备进程管理器已启动: device_id={}", self.device_id);
        Ok(())
    }

    /// 停止设备进程管理器
    pub async fn stop(&self) -> AppResult<()> {
        // 标记为停止
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // 停止当前运行的脚本
        if let Some(script_id) = self.get_current_running_script().await {
            self.stop_script(&script_id).await?;
        }

        // 停止调度器
        let scheduler = self.scheduler.read().await;
        scheduler.stop().await?;

        // 向主进程报告设备状态
        self.report_device_status(crate::domain::entities::devices::device_context::DeviceStatus::Offline).await?;

        info!("设备进程管理器已停止: device_id={}", self.device_id);
        Ok(())
    }

    /// 处理来自主进程的指令
    pub async fn handle_command(&self, command: DeviceCommand) -> AppResult<()> {
        match command {
            DeviceCommand::StartScript { script_id } => {
                self.start_script(&script_id).await?;
            }
            DeviceCommand::StopScript { script_id } => {
                self.stop_script(&script_id).await?;
            }
            DeviceCommand::PauseScript { script_id } => {
                self.pause_script(&script_id).await?;
            }
            DeviceCommand::ResumeScript { script_id } => {
                self.resume_script(&script_id).await?;
            }
            DeviceCommand::AddScript { script } => {
                self.add_script(script).await?;
            }
            DeviceCommand::RemoveScript { script_id } => {
                self.remove_script(&script_id).await?;
            }
            DeviceCommand::GetStatus => {
                self.report_current_status().await?;
            }
            DeviceCommand::Shutdown => {
                self.stop().await?;
            }
        }
        Ok(())
    }

    /// 添加脚本到设备
    pub async fn add_script(&self, script: Script) -> AppResult<()> {
        let script_id = script.id.clone();

        // 检查脚本是否已存在
        {
            let scripts = self.device_scripts.read().await;
            if scripts.contains_key(&script_id) {
                return Err(AppError::ConfigError(format!(
                    "脚本已存在于设备: device_id={}, script_id={}",
                    self.device_id, script_id
                )));
            }
        }

        // 注册到调度器
        let scheduler = self.scheduler.read().await;
        scheduler.register_script(script.clone()).await?;
        drop(scheduler);

        // 添加到本地脚本列表
        {
            let mut scripts = self.device_scripts.write().await;
            scripts.insert(script_id.clone(), script);
        }

        info!("脚本已添加到设备: device_id={}, script_id={}", self.device_id, script_id);
        Ok(())
    }

    /// 从设备移除脚本
    pub async fn remove_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 如果脚本正在运行，先停止
        if let Some(running_script) = self.get_current_running_script().await {
            if running_script == *script_id {
                self.stop_script(script_id).await?;
            }
        }

        // 从调度器取消注册
        let scheduler = self.scheduler.read().await;
        scheduler.unregister_script(script_id).await?;
        drop(scheduler);

        // 从本地脚本列表移除
        {
            let mut scripts = self.device_scripts.write().await;
            scripts.remove(script_id);
        }

        info!("脚本已从设备移除: device_id={}, script_id={}", self.device_id, script_id);
        Ok(())
    }

    /// 启动脚本执行
    pub async fn start_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 检查是否已有脚本在运行（单设备同时只能运行一个脚本）
        if let Some(running_script) = self.get_current_running_script().await {
            return Err(AppError::ConfigError(format!(
                "设备已有脚本在运行: device_id={}, running_script={}, requested_script={}",
                self.device_id, running_script, script_id
            )));
        }

        // 检查脚本是否存在
        {
            let scripts = self.device_scripts.read().await;
            if !scripts.contains_key(script_id) {
                return Err(AppError::ConfigError(format!(
                    "脚本不存在于设备: device_id={}, script_id={}",
                    self.device_id, script_id
                )));
            }
        }

        // 通过调度器启动脚本
        let scheduler = self.scheduler.read().await;
        scheduler.start_script(script_id, Some(self.device_id.clone())).await?;
        drop(scheduler);

        // 设置当前运行脚本
        {
            let mut current = self.current_running_script.write().await;
            *current = Some(script_id.clone());
        }

        // 向主进程报告脚本状态和设备状态
        self.report_script_status(script_id, ScriptStatus::Running).await?;
        self.report_device_status(crate::domain::entities::devices::device_context::DeviceStatus::Running).await?;

        info!("脚本执行已启动: device_id={}, script_id={}", self.device_id, script_id);
        Ok(())
    }

    /// 停止脚本执行
    pub async fn stop_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 通过调度器停止脚本
        let scheduler = self.scheduler.read().await;
        scheduler.stop_script(script_id).await?;
        drop(scheduler);

        // 清除当前运行脚本（如果是当前运行的脚本）
        {
            let mut current = self.current_running_script.write().await;
            if current.as_ref() == Some(script_id) {
                *current = None;
            }
        }

        // 向主进程报告脚本状态和设备状态
        self.report_script_status(script_id, ScriptStatus::Stopped).await?;
        self.report_device_status(crate::domain::entities::devices::device_context::DeviceStatus::Idle).await?;

        info!("脚本执行已停止: device_id={}, script_id={}", self.device_id, script_id);
        Ok(())
    }

    /// 暂停脚本执行
    pub async fn pause_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.pause_script(script_id).await?;

        self.report_script_status(script_id, ScriptStatus::Paused).await?;
        info!("脚本执行已暂停: device_id={}, script_id={}", self.device_id, script_id);
        Ok(())
    }

    /// 恢复脚本执行
    pub async fn resume_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.resume_script(script_id).await?;

        self.report_script_status(script_id, ScriptStatus::Running).await?;
        self.report_device_status(crate::domain::entities::devices::device_context::DeviceStatus::Running).await?;
        info!("脚本执行已恢复: device_id={}, script_id={}", self.device_id, script_id);
        Ok(())
    }

    /// 获取当前运行的脚本ID
    pub async fn get_current_running_script(&self) -> Option<ScriptId> {
        let current = self.current_running_script.read().await;
        current.clone()
    }

    /// 获取设备脚本列表
    pub async fn get_device_scripts(&self) -> HashMap<ScriptId, Script> {
        let scripts = self.device_scripts.read().await;
        scripts.clone()
    }

    /// 向主进程报告脚本状态
    async fn report_script_status(&self, script_id: &ScriptId, status: ScriptStatus) -> AppResult<()> {
        if let Some(sender) = &self.main_process_sender {
            let report = DeviceStatusReport::ScriptStatusUpdate {
                script_id: script_id.clone(),
                status,
            };
            sender.send(report).await
                .map_err(|e| AppError::InternalError(format!("发送脚本状态报告失败: {}", e)))?;
        }
        Ok(())
    }

    /// 向主进程报告设备状态
    async fn report_device_status(&self, status: crate::domain::entities::devices::device_context::DeviceStatus) -> AppResult<()> {
        if let Some(sender) = &self.main_process_sender {
            let report = DeviceStatusReport::DeviceStatusUpdate { status };
            sender.send(report).await
                .map_err(|e| AppError::InternalError(format!("发送设备状态报告失败: {}", e)))?;
        }
        Ok(())
    }

    /// 向主进程报告脚本执行结果
    async fn report_script_execution_result(
        &self,
        script_id: &ScriptId,
        success: bool,
        execution_time_ms: u64,
        error_message: Option<String>,
    ) -> AppResult<()> {
        if let Some(sender) = &self.main_process_sender {
            let report = DeviceStatusReport::ScriptExecutionResult {
                script_id: script_id.clone(),
                success,
                execution_time_ms,
                error_message,
            };
            sender.send(report).await
                .map_err(|e| AppError::InternalError(format!("发送脚本执行结果失败: {}", e)))?;
        }
        Ok(())
    }

    /// 报告当前状态
    async fn report_current_status(&self) -> AppResult<()> {
        let current_script = self.get_current_running_script().await;
        
        if let Some(sender) = &self.main_process_sender {
            let report = DeviceStatusReport::DeviceStats {
                cpu_usage: 0.0, // 实际实现中应该获取真实的CPU使用率
                memory_usage: 0, // 实际实现中应该获取真实的内存使用量
                running_script: current_script,
            };
            sender.send(report).await
                .map_err(|e| AppError::InternalError(format!("发送设备状态失败: {}", e)))?;
        }
        Ok(())
    }

    /// 启动心跳循环
    async fn start_heartbeat_loop(&self) {
        let device_id = self.device_id.clone();
        let sender = self.main_process_sender.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(30)); // 30秒心跳间隔

            loop {
                interval.tick().await;

                // 检查是否应该继续运行
                {
                    let running = is_running.read().await;
                    if !*running {
                        break;
                    }
                }

                // 发送心跳
                if let Some(sender) = &sender {
                    if let Err(e) = sender.send(DeviceStatusReport::Heartbeat).await {
                        error!("发送心跳失败: device_id={}, error={}", device_id, e);
                        break;
                    }
                }
            }

            info!("设备进程心跳循环已退出: device_id={}", device_id);
        });
    }

    /// 启动状态报告循环
    async fn start_status_report_loop(&self) {
        let device_id = self.device_id.clone();
        let sender = self.main_process_sender.clone();
        let is_running = self.is_running.clone();
        let current_running_script = self.current_running_script.clone();

        tokio::spawn(async move {
            let mut interval = interval(TokioDuration::from_secs(60)); // 60秒状态报告间隔

            loop {
                interval.tick().await;

                // 检查是否应该继续运行
                {
                    let running = is_running.read().await;
                    if !*running {
                        break;
                    }
                }

                // 发送状态报告
                if let Some(sender) = &sender {
                    let current_script = {
                        let current = current_running_script.read().await;
                        current.clone()
                    };

                    let report = DeviceStatusReport::DeviceStats {
                        cpu_usage: 0.0, // 实际实现中应该获取真实的数据
                        memory_usage: 0,
                        running_script: current_script,
                    };

                    if let Err(e) = sender.send(report).await {
                        error!("发送状态报告失败: device_id={}, error={}", device_id, e);
                        break;
                    }
                }
            }

            info!("设备进程状态报告循环已退出: device_id={}", device_id);
        });
    }

    /// 获取设备进程概览
    pub async fn get_overview(&self) -> String {
        let current_script = self.get_current_running_script().await;
        let script_count = {
            let scripts = self.device_scripts.read().await;
            scripts.len()
        };

        format!(
            "DeviceProcess[device_id={}, scripts={}, running_script={:?}]",
            self.device_id,
            script_count,
            current_script
        )
    }
}

/// 设备进程入口点 - 这将在独立的设备进程中运行
pub async fn run_device_process(
    config: DeviceProcessConfig,
    main_process_sender: mpsc::Sender<DeviceStatusReport>,
    mut command_receiver: mpsc::Receiver<DeviceCommand>,
) -> AppResult<()> {
    info!("启动设备进程: device_id={}", config.device_id);

    let process_manager = ProcessManager::new();
    let mut device_manager = DeviceProcessManager::new(config.clone(), process_manager);

    // 初始化设备进程管理器
    device_manager.initialize(main_process_sender).await?;

    // 启动设备进程管理器
    device_manager.start().await?;

    // 监听来自主进程的指令
    while let Some(command) = command_receiver.recv().await {
        match command {
            DeviceCommand::Shutdown => {
                info!("收到关闭指令，设备进程将退出: device_id={}", config.device_id);
                break;
            }
            _ => {
                if let Err(e) = device_manager.handle_command(command).await {
                    error!("处理指令失败: device_id={}, error={}", config.device_id, e);
                }
            }
        }
    }

    // 停止设备进程管理器
    device_manager.stop().await?;

    info!("设备进程已退出: device_id={}", config.device_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_process_manager_single_script_constraint() {
        let config = DeviceProcessConfig::default();
        let process_manager = ProcessManager::new();
        let manager = DeviceProcessManager::new(config, process_manager);

        // 测试单脚本约束 - 这里可以添加具体的测试逻辑
        assert!(manager.get_current_running_script().await.is_none());
    }
}
