use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::devices::device_context::{DeviceContext, DeviceStatus};
use crate::infrastructure::performance::{ProcessManager, ProcessConfig};
use crate::domain::entities::config::performance::Performance;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error};
use crate::domain::entities::scripts::script_info::{ScriptId, ScriptInfo, ScriptStatus};

/// 主进程到设备进程的指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceCommand {
    /// 启动脚本
    StartScript { script_id: ScriptId },
    /// 停止脚本
    StopScript { script_id: ScriptId },
    /// 暂停脚本
    PauseScript { script_id: ScriptId },
    /// 恢复脚本
    ResumeScript { script_id: ScriptId },
    /// 添加脚本到设备
    AddScript { script: ScriptInfo },
    /// 从设备移除脚本
    RemoveScript { script_id: ScriptId },
    /// 请求设备状态
    GetStatus,
    /// 关闭设备进程
    Shutdown,
}

/// 设备进程到主进程的状态报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceStatusReport {
    /// 脚本状态更新
    ScriptStatusUpdate { script_id: ScriptId, status: ScriptStatus },
    /// 设备状态更新
    DeviceStatusUpdate { status: DeviceStatus },
    /// 脚本执行结果
    ScriptExecutionResult { 
        script_id: ScriptId, 
        success: bool, 
        execution_time_ms: u64,
        error_message: Option<String>
    },
    /// 设备统计信息
    DeviceStats { 
        cpu_usage: f32, 
        memory_usage: u64, 
        running_script: Option<ScriptId>
    },
    /// 错误报告
    Error { message: String },
    /// 心跳
    Heartbeat,
}

/// 脚本分配请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptAssignmentRequest {
    pub script_id: ScriptId,
    pub device_id: String,
    /// 是否替换设备当前脚本
    pub replace_current: bool,
}

/// 主进程管理器 - 负责管理设备进程和前端交互
/// 
/// 正确的架构：
/// - 主进程：管理设备进程，处理前端请求，不包含调度引擎
/// - 设备进程：每个设备有独立的调度引擎，同时只运行一个脚本
/// - 通信：通过进程间通信（IPC）协调工作
#[derive(Debug)]
pub struct MainProcessManager {
    /// 设备上下文映射 (device_id -> DeviceContext)
    devices: Arc<RwLock<HashMap<String, DeviceContext>>>,
    /// 进程管理器
    process_manager: Arc<ProcessManager>,
    /// 脚本到设备的映射 (script_id -> device_id)
    script_device_mapping: Arc<RwLock<HashMap<ScriptId, String>>>,
    /// 全局脚本注册表 (script_id -> Script)
    global_scripts: Arc<RwLock<HashMap<ScriptId, ScriptInfo>>>,
    /// 设备通信通道 (device_id -> sender)
    device_channels: Arc<RwLock<HashMap<String, mpsc::Sender<DeviceCommand>>>>,
    /// 性能配置
    performance_config: Performance,
}

/// 主进程管理器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainProcessStats {
    /// 总设备数
    pub total_devices: usize,
    /// 在线设备数
    pub online_devices: usize,
    /// 运行中设备数
    pub running_devices: usize,
    /// 总脚本数
    pub total_scripts: usize,
    /// 已分配脚本数
    pub assigned_scripts: usize,
    /// 运行中脚本数
    pub running_scripts: usize,
    /// 系统负载（0-100%）
    pub system_load_percent: u8,
}

impl MainProcessManager {
    /// 创建主进程管理器
    pub fn new(performance_config: Performance, process_manager: ProcessManager) -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            process_manager: Arc::new(process_manager),
            script_device_mapping: Arc::new(RwLock::new(HashMap::new())),
            global_scripts: Arc::new(RwLock::new(HashMap::new())),
            device_channels: Arc::new(RwLock::new(HashMap::new())),
            performance_config,
        }
    }

    /// 初始化主进程管理器
    pub async fn initialize(&self) -> AppResult<()> {
        info!("主进程管理器初始化完成");
        Ok(())
    }

    /// 注册设备
    pub async fn register_device(
        &self,
        device_id: String,
        device_info: crate::infrastructure::entities::devices::device_info::DeviceInfo,
        process_config: ProcessConfig,
    ) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        
        if devices.contains_key(&device_id) {
            return Err(AppError::ConfigError(format!("设备已存在: {}", device_id)));
        }

        let device_context = DeviceContext::new(
            device_id.clone(),
            device_info,
            process_config,
        );

        devices.insert(device_id.clone(), device_context);
        info!("设备已注册: {}", device_id);
        Ok(())
    }

    /// 启动设备进程
    pub async fn start_device(&self, device_id: &str) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        let device = devices.get_mut(device_id)
            .ok_or_else(|| AppError::ConfigError(format!("设备不存在: {}", device_id)))?;

        // 启动设备进程
        device.start_device_process(&self.process_manager).await?;

        // 建立通信通道
        let (tx, rx) = mpsc::channel::<DeviceCommand>(100);
        {
            let mut channels = self.device_channels.write().await;
            channels.insert(device_id.to_string(), tx);
        }

        // 启动设备进程消息处理循环
        self.start_device_message_loop(device_id.to_string(), rx).await;

        info!("设备进程已启动: {}", device_id);
        Ok(())
    }

    /// 停止设备进程
    pub async fn stop_device(&self, device_id: &str) -> AppResult<()> {
        // 发送关闭指令
        if let Err(e) = self.send_command_to_device(device_id, DeviceCommand::Shutdown).await {
            warn!("发送关闭指令失败: device_id={}, error={}", device_id, e);
        }

        // 移除通信通道
        {
            let mut channels = self.device_channels.write().await;
            channels.remove(device_id);
        }

        // 停止设备进程
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.stop_device_process(&self.process_manager).await?;
        }

        info!("设备进程已停止: {}", device_id);
        Ok(())
    }

    /// 注册脚本
    pub async fn register_script(&self, script: ScriptInfo) -> AppResult<()> {
        let script_id = script.id.clone();
        let mut scripts = self.global_scripts.write().await;
        
        if scripts.contains_key(&script_id) {
            return Err(AppError::ConfigError(format!("脚本已存在: {}", script_id)));
        }

        scripts.insert(script_id.clone(), script);
        info!("脚本已注册: {}", script_id);
        Ok(())
    }

    /// 分配脚本到设备
    pub async fn assign_script_to_device(&self, request: ScriptAssignmentRequest) -> AppResult<()> {
        // 检查脚本是否存在
        let script = {
            let scripts = self.global_scripts.read().await;
            scripts.get(&request.script_id)
                .ok_or_else(|| AppError::ConfigError(format!("脚本不存在: {}", request.script_id)))?
                .clone()
        };

        // 检查设备是否存在
        {
            let devices = self.devices.read().await;
            if !devices.contains_key(&request.device_id) {
                return Err(AppError::ConfigError(format!("设备不存在: {}", request.device_id)));
            }
        }

        // 如果需要替换当前脚本，先停止当前运行的脚本
        if request.replace_current {
            if let Err(e) = self.stop_device_current_script(&request.device_id).await {
                warn!("停止设备当前脚本失败: device_id={}, error={}", request.device_id, e);
            }
        }

        // 向设备进程发送添加脚本指令
        self.send_command_to_device(&request.device_id, DeviceCommand::AddScript { 
            script: script.clone() 
        }).await?;

        // 更新映射关系
        {
            let mut mapping = self.script_device_mapping.write().await;
            mapping.insert(request.script_id.clone(), request.device_id.clone());
        }

        // 更新设备本地脚本列表
        {
            let mut devices = self.devices.write().await;
            if let Some(device) = devices.get_mut(&request.device_id) {
                device.add_script(script).await?;
            }
        }

        info!("脚本已分配到设备: script_id={}, device_id={}", 
              request.script_id, request.device_id);
        Ok(())
    }

    /// 启动脚本执行（从前端调用）
    pub async fn start_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 获取脚本所在设备
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id)
                .ok_or_else(|| AppError::ConfigError(format!("脚本未分配到任何设备: {}", script_id)))?
                .clone()
        };

        // 检查设备状态
        {
            let devices = self.devices.read().await;
            let device = devices.get(&device_id)
                .ok_or_else(|| AppError::ConfigError(format!("设备不存在: {}", device_id)))?;

            if device.status == DeviceStatus::Offline {
                return Err(AppError::ConfigError(format!("设备离线: {}", device_id)));
            }

            // 检查设备是否已有脚本在运行
            if device.get_running_scripts_count() > 0 {
                return Err(AppError::ConfigError(format!(
                    "设备已有脚本在运行: device_id={}, 运行中脚本数: {}",
                    device_id,
                    device.get_running_scripts_count()
                )));
            }
        }

        // 向设备进程发送启动脚本指令
        self.send_command_to_device(&device_id, DeviceCommand::StartScript { 
            script_id: script_id.clone() 
        }).await?;

        info!("脚本启动指令已发送: script_id={}, device_id={}", script_id, device_id);
        Ok(())
    }

    /// 停止脚本执行（从前端调用）
    pub async fn stop_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        if let Some(device_id) = device_id {
            self.send_command_to_device(&device_id, DeviceCommand::StopScript { 
                script_id: script_id.clone() 
            }).await?;
            info!("脚本停止指令已发送: script_id={}, device_id={}", script_id, device_id);
        } else {
            warn!("脚本未分配到任何设备: script_id={}", script_id);
        }

        Ok(())
    }

    /// 暂停脚本执行（从前端调用）
    pub async fn pause_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        if let Some(device_id) = device_id {
            self.send_command_to_device(&device_id, DeviceCommand::PauseScript { 
                script_id: script_id.clone() 
            }).await?;
            info!("脚本暂停指令已发送: script_id={}, device_id={}", script_id, device_id);
        }

        Ok(())
    }

    /// 恢复脚本执行（从前端调用）
    pub async fn resume_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        if let Some(device_id) = device_id {
            self.send_command_to_device(&device_id, DeviceCommand::ResumeScript { 
                script_id: script_id.clone() 
            }).await?;
            info!("脚本恢复指令已发送: script_id={}, device_id={}", script_id, device_id);
        }

        Ok(())
    }

    /// 获取脚本状态（从设备同步）
    pub async fn get_script_status(&self, script_id: &ScriptId) -> Option<ScriptStatus> {
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        if let Some(device_id) = device_id {
            let devices = self.devices.read().await;
            if let Some(device) = devices.get(&device_id) {
                return device.get_script_status(script_id).await;
            }
        }

        None
    }

    /// 获取所有脚本状态
    pub async fn get_all_script_status(&self) -> HashMap<ScriptId, ScriptStatus> {
        let mut all_status = HashMap::new();
        let devices = self.devices.read().await;
        
        for device in devices.values() {
            let device_status = device.get_all_script_status().await;
            all_status.extend(device_status);
        }

        all_status
    }

    /// 获取设备状态
    pub async fn get_device_status(&self, device_id: &str) -> Option<DeviceStatus> {
        let devices = self.devices.read().await;
        devices.get(device_id).map(|device| device.status.clone())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> MainProcessStats {
        let devices = self.devices.read().await;
        let mapping = self.script_device_mapping.read().await;
        let scripts = self.global_scripts.read().await;

        let total_devices = devices.len();
        let online_devices = devices.values()
            .filter(|device| device.status != DeviceStatus::Offline)
            .count();
        let running_devices = devices.values()
            .filter(|device| device.status == DeviceStatus::Running)
            .count();
        let total_scripts = scripts.len();
        let assigned_scripts = mapping.len();
        let running_scripts = devices.values()
            .map(|device| device.get_running_scripts_count())
            .sum();

        let system_load_percent = if self.performance_config.max_devices > 0 {
            ((running_devices as f64 / self.performance_config.max_devices as f64) * 100.0) as u8
        } else {
            0
        };

        MainProcessStats {
            total_devices,
            online_devices,
            running_devices,
            total_scripts,
            assigned_scripts,
            running_scripts,
            system_load_percent,
        }
    }

    /// 发送指令到设备进程
    async fn send_command_to_device(&self, device_id: &str, command: DeviceCommand) -> AppResult<()> {
        let channels = self.device_channels.read().await;
        if let Some(sender) = channels.get(device_id) {
            sender.send(command).await
                .map_err(|e| AppError::InternalError(format!("发送指令到设备失败: {}", e)))?;
            Ok(())
        } else {
            Err(AppError::ConfigError(format!("设备通信通道不存在: {}", device_id)))
        }
    }

    /// 停止设备当前运行的脚本
    async fn stop_device_current_script(&self, device_id: &str) -> AppResult<()> {
        let running_scripts = {
            let devices = self.devices.read().await;
            if let Some(device) = devices.get(device_id) {
                device.get_scripts()
                    .iter()
                    .filter(|(_, script)| matches!(script.status, ScriptStatus::Running))
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for script_id in running_scripts {
            self.stop_script(&script_id).await?;
        }

        Ok(())
    }

    /// 启动设备进程消息处理循环
    async fn start_device_message_loop(&self, device_id: String, mut rx: mpsc::Receiver<DeviceCommand>) {
        // 这里应该实现设备进程的消息处理逻辑
        // 实际实现中，这将是与设备进程的IPC通信
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                match command {
                    DeviceCommand::StartScript { script_id } => {
                        info!("设备进程收到启动脚本指令: device_id={}, script_id={}", device_id, script_id);
                        // 实际实现：通过IPC发送到设备进程
                    }
                    DeviceCommand::StopScript { script_id } => {
                        info!("设备进程收到停止脚本指令: device_id={}, script_id={}", device_id, script_id);
                        // 实际实现：通过IPC发送到设备进程
                    }
                    DeviceCommand::Shutdown => {
                        info!("设备进程收到关闭指令: device_id={}", device_id);
                        break;
                    }
                    _ => {
                        info!("设备进程收到其他指令: device_id={}, command={:?}", device_id, command);
                    }
                }
            }
            info!("设备进程消息循环已退出: device_id={}", device_id);
        });
    }

    /// 处理设备进程状态报告（由设备进程调用）
    pub async fn handle_device_status_report(&self, device_id: &str, report: DeviceStatusReport) -> AppResult<()> {
        match report {
            DeviceStatusReport::ScriptStatusUpdate { script_id, status } => {
                let mut devices = self.devices.write().await;
                if let Some(device) = devices.get_mut(device_id) {
                    device.update_script_status(&script_id, status).await?;
                }
                info!("收到脚本状态更新: device_id={}, script_id={}, status={:?}", 
                      device_id, script_id, status);
            }
            DeviceStatusReport::DeviceStatusUpdate { status } => {
                let mut devices = self.devices.write().await;
                if let Some(device) = devices.get_mut(device_id) {
                    device.status = status.clone();
                    device.updated_at = Local::now();
                }
                info!("收到设备状态更新: device_id={}, status={:?}", device_id, status);
            }
            DeviceStatusReport::ScriptExecutionResult { script_id, success, execution_time_ms, error_message } => {
                let mut devices = self.devices.write().await;
                if let Some(device) = devices.get_mut(device_id) {
                    device.record_script_execution(&script_id, success, execution_time_ms);
                }
                info!("收到脚本执行结果: device_id={}, script_id={}, success={}, duration={}ms", 
                      device_id, script_id, success, execution_time_ms);
                if let Some(error) = error_message {
                    warn!("脚本执行错误: {}", error);
                }
            }
            DeviceStatusReport::DeviceStats { cpu_usage, memory_usage, running_script } => {
                let mut devices = self.devices.write().await;
                if let Some(device) = devices.get_mut(device_id) {
                    device.update_stats(cpu_usage, memory_usage);
                }
            }
            DeviceStatusReport::Error { message } => {
                error!("设备报告错误: device_id={}, error={}", device_id, message);
            }
            DeviceStatusReport::Heartbeat => {
                // 更新最后活动时间
                let mut devices = self.devices.write().await;
                if let Some(device) = devices.get_mut(device_id) {
                    device.stats.last_activity_time = Local::now();
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_main_process_manager() {
        let performance_config = Performance {
            cores_per_device: 4,
            max_devices: 2,
        };
        let process_manager = ProcessManager::new();
        let manager = MainProcessManager::new(performance_config, process_manager);

        assert!(manager.initialize().await.is_ok());
    }
}
