use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::performance::Performance;
use crate::domain::entities::devices::device_context::{DeviceContext, DeviceStatus};
use crate::domain::entities::scheduler::scheduler_state::SchedulerConfig;
use crate::domain::entities::scheduler::script_scheduler::ScriptScheduler;
use crate::domain::entities::scripts::script_info::{ScriptId, ScriptInfo, ScriptStatus};
use crate::infrastructure::performance::{ProcessConfig, ProcessManager};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// 设备管理器 - 在主进程中管理所有设备和脚本调度
/// 
/// 正确的架构：
/// - 调度引擎在主进程中（单一调度器）
/// - 每个设备进程同时只能运行一个脚本
/// - 最大并发数 = 设备数量
/// - 主进程通过进程通信控制设备脚本执行
#[derive(Debug)]
pub struct DeviceManager {
    /// 设备上下文映射 (device_id -> DeviceContext)
    devices: Arc<RwLock<HashMap<String, DeviceContext>>>,
    /// 全局脚本调度器（在主进程中）
    scheduler: Arc<RwLock<ScriptScheduler>>,
    /// 进程管理器
    process_manager: Arc<ProcessManager>,
    /// 脚本到设备的映射 (script_id -> device_id)
    script_device_mapping: Arc<RwLock<HashMap<ScriptId, String>>>,
    /// 性能配置
    performance_config: Performance,
}

/// 设备脚本分配请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptAssignmentRequest {
    pub script_id: ScriptId,
    pub device_id: String,
    /// 是否替换设备当前运行的脚本
    pub replace_current: bool,
}

/// 设备管理器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceManagerStats {
    /// 总设备数
    pub total_devices: usize,
    /// 在线设备数
    pub online_devices: usize,
    /// 运行中设备数
    pub running_devices: usize,
    /// 总脚本数
    pub total_scripts: usize,
    /// 运行中脚本数
    pub running_scripts: usize,
    /// 系统负载（0-100%）
    pub system_load_percent: u8,
}

impl DeviceManager {
    /// 创建设备管理器
    pub fn new(performance_config: Performance, process_manager: ProcessManager) -> Self {
        // 调度器配置：最大并发数 = 最大设备数（因为每设备最多1个脚本）
        let scheduler_config = SchedulerConfig {
            max_concurrent_tasks: performance_config.max_devices,
            check_interval_seconds: 30,
            default_task_timeout_seconds: 300,
            enable_auto_retry: true,
            default_retry_count: 3,
            enable_idle_detection: true,
            idle_threshold_seconds: 300,
        };

        let scheduler = ScriptScheduler::new(scheduler_config, process_manager.clone());

        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            scheduler: Arc::new(RwLock::new(scheduler)),
            process_manager: Arc::new(process_manager),
            script_device_mapping: Arc::new(RwLock::new(HashMap::new())),
            performance_config,
        }
    }

    /// 初始化设备管理器
    pub async fn initialize(&self) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.initialize().await?;
        info!("设备管理器初始化完成");
        Ok(())
    }

    /// 启动设备管理器
    pub async fn start(&self) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.start().await?;
        info!("设备管理器已启动");
        Ok(())
    }

    /// 停止设备管理器
    pub async fn stop(&self) -> AppResult<()> {
        // 停止所有设备
        let device_ids: Vec<String> = {
            let devices = self.devices.read().await;
            devices.keys().cloned().collect()
        };

        for device_id in device_ids {
            if let Err(e) = self.stop_device(&device_id).await {
                warn!("停止设备失败: device_id={}, error={}", device_id, e);
            }
        }

        // 停止调度器
        let scheduler = self.scheduler.read().await;
        scheduler.stop().await?;

        info!("设备管理器已停止");
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

        // 创建设备上下文，注意：不再为每个设备创建独立调度器
        let device_context = DeviceContext::new(
            device_id.clone(),
            device_info,
            process_config,
        );

        devices.insert(device_id.clone(), device_context);
        info!("设备已注册: {}", device_id);
        Ok(())
    }

    /// 取消注册设备
    pub async fn unregister_device(&self, device_id: &str) -> AppResult<()> {
        // 先停止设备
        self.stop_device(device_id).await?;

        // 移除设备
        let mut devices = self.devices.write().await;
        devices.remove(device_id);

        // 清理脚本映射
        let mut mapping = self.script_device_mapping.write().await;
        mapping.retain(|_, dev_id| dev_id != device_id);

        info!("设备已取消注册: {}", device_id);
        Ok(())
    }

    /// 启动设备进程
    pub async fn start_device(&self, device_id: &str) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        let device = devices.get_mut(device_id)
            .ok_or_else(|| AppError::ConfigError(format!("设备不存在: {}", device_id)))?;

        device.start_device_process(&self.process_manager).await?;
        info!("设备进程已启动: {}", device_id);
        Ok(())
    }

    /// 停止设备进程
    pub async fn stop_device(&self, device_id: &str) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        let device = devices.get_mut(device_id)
            .ok_or_else(|| AppError::ConfigError(format!("设备不存在: {}", device_id)))?;

        // 停止设备上所有脚本
        let script_ids: Vec<ScriptId> = device.get_scripts().keys().cloned().collect();
        drop(devices); // 释放锁

        for script_id in script_ids {
            if let Err(e) = self.stop_script(&script_id).await {
                warn!("停止脚本失败: script_id={}, error={}", script_id, e);
            }
        }

        // 重新获取锁并停止设备进程
        let mut devices = self.devices.write().await;
        let device = devices.get_mut(device_id).unwrap();
        device.stop_device_process(&self.process_manager).await?;

        info!("设备进程已停止: {}", device_id);
        Ok(())
    }

    /// 分配脚本到设备
    pub async fn assign_script_to_device(&self, request: ScriptAssignmentRequest) -> AppResult<()> {
        let mut devices = self.devices.write().await;
        let device = devices.get_mut(&request.device_id)
            .ok_or_else(|| AppError::ConfigError(format!("设备不存在: {}", request.device_id)))?;

        // 检查设备是否在线
        if device.status == DeviceStatus::Offline {
            return Err(AppError::ConfigError(format!("设备离线: {}", request.device_id)));
        }

        // 检查设备是否已有脚本在运行
        if device.get_running_scripts_count() > 0 && !request.replace_current {
            return Err(AppError::ConfigError(format!(
                "设备已有脚本在运行: {}, 当前运行脚本数: {}",
                request.device_id,
                device.get_running_scripts_count()
            )));
        }

        // 如果需要替换当前脚本，先停止所有运行中的脚本
        if request.replace_current {
            let running_scripts: Vec<ScriptId> = device.get_scripts()
                .iter()
                .filter(|(_, script)| matches!(script.status, ScriptStatus::Running))
                .map(|(id, _)| id.clone())
                .collect();

            drop(devices); // 释放锁

            for script_id in running_scripts {
                self.stop_script(&script_id).await?;
            }
        }

        // 更新脚本设备映射
        let mut mapping = self.script_device_mapping.write().await;
        mapping.insert(request.script_id.clone(), request.device_id.clone());

        info!("脚本已分配到设备: script_id={}, device_id={}", 
              request.script_id, request.device_id);
        Ok(())
    }

    /// 启动脚本执行（通过主进程调度器）
    pub async fn start_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 检查脚本是否已分配到设备
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
                .ok_or_else(|| AppError::ConfigError(format!("脚本未分配到任何设备: {}", script_id)))?
        };

        // 检查设备状态
        {
            let devices = self.devices.read().await;
            let device = devices.get(&device_id)
                .ok_or_else(|| AppError::ConfigError(format!("设备不存在: {}", device_id)))?;

            if device.status == DeviceStatus::Offline {
                return Err(AppError::ConfigError(format!("设备离线: {}", device_id)));
            }

            // 检查设备是否已有脚本在运行（每设备同时只能运行一个脚本）
            if device.get_running_scripts_count() > 0 {
                return Err(AppError::ConfigError(format!(
                    "设备已有脚本在运行: device_id={}, 运行中脚本数: {}",
                    device_id,
                    device.get_running_scripts_count()
                )));
            }
        }

        // 通过主进程调度器启动脚本
        let scheduler = self.scheduler.read().await;
        scheduler.start_script(script_id, Some(device_id.clone())).await?;

        // 更新设备中的脚本状态
        {
            let mut devices = self.devices.write().await;
            if let Some(device) = devices.get_mut(&device_id) {
                device.update_script_status(script_id, ScriptStatus::Running).await?;
            }
        }

        info!("脚本执行已启动: script_id={}, device_id={}", script_id, device_id);
        Ok(())
    }

    /// 停止脚本执行
    pub async fn stop_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 获取脚本所在设备
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        // 通过主进程调度器停止脚本
        let scheduler = self.scheduler.read().await;
        scheduler.stop_script(script_id).await?;

        // 更新设备中的脚本状态
        if let Some(device_id) = device_id {
            let mut devices = self.devices.write().await;
            if let Some(device) = devices.get_mut(&device_id) {
                device.update_script_status(script_id, ScriptStatus::Stopped).await?;
            }
        }

        info!("脚本执行已停止: script_id={}", script_id);
        Ok(())
    }

    /// 暂停脚本执行
    pub async fn pause_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.pause_script(script_id).await?;

        // 更新设备中的脚本状态
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        if let Some(device_id) = device_id {
            let mut devices = self.devices.write().await;
            if let Some(device) = devices.get_mut(&device_id) {
                device.update_script_status(script_id, ScriptStatus::Paused).await?;
            }
        }

        info!("脚本执行已暂停: script_id={}", script_id);
        Ok(())
    }

    /// 恢复脚本执行
    pub async fn resume_script(&self, script_id: &ScriptId) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.resume_script(script_id).await?;

        // 更新设备中的脚本状态
        let device_id = {
            let mapping = self.script_device_mapping.read().await;
            mapping.get(script_id).cloned()
        };

        if let Some(device_id) = device_id {
            let mut devices = self.devices.write().await;
            if let Some(device) = devices.get_mut(&device_id) {
                device.update_script_status(script_id, ScriptStatus::Running).await?;
            }
        }

        info!("脚本执行已恢复: script_id={}", script_id);
        Ok(())
    }

    /// 注册脚本到调度器
    pub async fn register_script(&self, script: ScriptInfo) -> AppResult<()> {
        let scheduler = self.scheduler.read().await;
        scheduler.register_script(script).await?;
        info!("脚本已注册到调度器");
        Ok(())
    }

    /// 取消注册脚本
    pub async fn unregister_script(&self, script_id: &ScriptId) -> AppResult<()> {
        // 停止脚本执行
        self.stop_script(script_id).await?;

        // 从调度器取消注册
        let scheduler = self.scheduler.read().await;
        scheduler.unregister_script(script_id).await?;

        // 清理脚本设备映射
        let mut mapping = self.script_device_mapping.write().await;
        mapping.remove(script_id);

        info!("脚本已从调度器取消注册: script_id={}", script_id);
        Ok(())
    }

    /// 获取脚本状态
    pub async fn get_script_status(&self, script_id: &ScriptId) -> Option<ScriptStatus> {
        let scheduler = self.scheduler.read().await;
        scheduler.get_script_status(script_id).await
    }

    /// 获取所有脚本状态
    pub async fn get_all_script_status(&self) -> HashMap<ScriptId, ScriptStatus> {
        let scheduler = self.scheduler.read().await;
        scheduler.get_all_script_status().await
    }

    /// 获取设备状态
    pub async fn get_device_status(&self, device_id: &str) -> Option<DeviceStatus> {
        let devices = self.devices.read().await;
        devices.get(device_id).map(|device| device.status.clone())
    }

    /// 获取所有设备状态
    pub async fn get_all_device_status(&self) -> HashMap<String, DeviceStatus> {
        let devices = self.devices.read().await;
        devices.iter()
            .map(|(id, device)| (id.clone(), device.status.clone()))
            .collect()
    }

    /// 获取设备脚本映射
    pub async fn get_script_device_mapping(&self) -> HashMap<ScriptId, String> {
        let mapping = self.script_device_mapping.read().await;
        mapping.clone()
    }

    /// 获取设备管理器统计信息
    pub async fn get_stats(&self) -> DeviceManagerStats {
        let devices = self.devices.read().await;
        let mapping = self.script_device_mapping.read().await;

        let total_devices = devices.len();
        let online_devices = devices.values()
            .filter(|device| device.status != DeviceStatus::Offline)
            .count();
        let running_devices = devices.values()
            .filter(|device| device.status == DeviceStatus::Running)
            .count();
        let total_scripts = mapping.len();
        let running_scripts = devices.values()
            .map(|device| device.get_running_scripts_count())
            .sum();

        // 计算系统负载
        let system_load_percent = if self.performance_config.max_devices > 0 {
            ((running_devices as f64 / self.performance_config.max_devices as f64) * 100.0) as u8
        } else {
            0
        };

        DeviceManagerStats {
            total_devices,
            online_devices,
            running_devices,
            total_scripts,
            running_scripts,
            system_load_percent,
        }
    }

    /// 获取设备管理器概览
    pub async fn get_overview(&self) -> String {
        let stats = self.get_stats().await;
        let scheduler = self.scheduler.read().await;
        let scheduler_overview = scheduler.get_overview().await;

        format!(
            "DeviceManager[设备:{}/{}, 脚本:{}/{}, 负载:{}%, {}]",
            stats.online_devices,
            stats.total_devices,
            stats.running_scripts,
            stats.total_scripts,
            stats.system_load_percent,
            scheduler_overview
        )
    }

    /// 更新活动时间（由主界面调用）
    pub async fn update_activity(&self) {
        let scheduler = self.scheduler.read().await;
        scheduler.update_activity().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_manager_lifecycle() {
        let performance_config = Performance {
            cores_per_device: 4,
            max_devices: 2,
        };
        let process_manager = ProcessManager::new();
        let manager = DeviceManager::new(performance_config, process_manager);

        assert!(manager.initialize().await.is_ok());
        assert!(manager.start().await.is_ok());
        assert!(manager.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_single_script_per_device_constraint() {
        let performance_config = Performance {
            cores_per_device: 4,
            max_devices: 1,
        };
        let process_manager = ProcessManager::new();
        let manager = DeviceManager::new(performance_config, process_manager);

        // 这里可以添加更多测试，验证每个设备只能运行一个脚本的约束
    }
}
