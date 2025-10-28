// 主进程管理器
// 统一管理所有子进程的创建、监控和销毁

use crate::infrastructure::core::{ProcessId, DeviceId, HashMap, Error};
use crate::infrastructure::core::allocator::CpuCoreAllocator;
use crate::infrastructure::core::rayon_pool::RayonPoolManager;
use crate::infrastructure::process::handle::{ProcessHandle, DeviceProcessHandle, NodeProcessHandle, ProcessState, ProcessType, ProcessPriority, ProcessError, ProcessResult};
use crate::infrastructure::process::config::{ProcessConfig, ResourceConstraints};
use crate::infrastructure::process::monitor::{ProcessMonitor, PerformanceSummary};
use crate::infrastructure::process::lifecycle::ProcessLifecycleManager;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{SystemTime, Duration};
use std::path::PathBuf;
use tokio::sync::mpsc;
use uuid::Uuid;

/// 主进程管理器错误
#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("进程管理器错误: {message}")]
    ProcessError { message: String },
    
    #[error("资源分配错误: {message}")]
    ResourceError { message: String },
    
    #[error("配置错误: {message}")]
    ConfigError { message: String },
    
    #[error("通信错误: {message}")]
    CommunicationError { message: String },
}

pub type ManagerResult<T> = Result<T, ManagerError>;

/// 全局进程状态
#[derive(Debug, Clone)]
pub struct GlobalState {
    pub total_processes: usize,
    pub running_processes: usize,
    pub failed_processes: usize,
    pub total_allocated_cores: usize,
    pub total_memory_usage_mb: usize,
    pub last_updated: SystemTime,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            total_processes: 0,
            running_processes: 0,
            failed_processes: 0,
            total_allocated_cores: 0,
            total_memory_usage_mb: 0,
            last_updated: SystemTime::now(),
        }
    }
}

/// 内存管理器（简化版）
pub struct MemoryManager {
    total_memory_mb: usize,
    allocated_memory_mb: usize,
    memory_allocations: HashMap<ProcessId, usize>,
}

impl MemoryManager {
    pub fn new() -> Self {
        // 获取系统总内存
        let total_memory = Self::get_system_memory_mb();
        
        Self {
            total_memory_mb: total_memory,
            allocated_memory_mb: 0,
            memory_allocations: HashMap::default(),
        }
    }
    
    fn get_system_memory_mb() -> usize {
        use sysinfo::System;
        let mut system = System::new();
        system.refresh_memory();
        (system.total_memory() / 1024 / 1024) as usize
    }
    
    pub fn allocate_memory(&mut self, process_id: ProcessId, memory_mb: usize) -> Result<(), String> {
        if self.allocated_memory_mb + memory_mb > self.total_memory_mb * 80 / 100 { // 80%限制
            return Err(format!("内存不足，请求{}MB，可用{}MB", 
                             memory_mb, 
                             self.total_memory_mb * 80 / 100 - self.allocated_memory_mb));
        }
        
        self.memory_allocations.insert(process_id, memory_mb);
        self.allocated_memory_mb += memory_mb;
        Ok(())
    }
    
    pub fn deallocate_memory(&mut self, process_id: &ProcessId) -> Option<usize> {
        if let Some(memory_mb) = self.memory_allocations.remove(process_id) {
            self.allocated_memory_mb = self.allocated_memory_mb.saturating_sub(memory_mb);
            Some(memory_mb)
        } else {
            None
        }
    }
    
    pub fn get_available_memory_mb(&self) -> usize {
        (self.total_memory_mb * 80 / 100).saturating_sub(self.allocated_memory_mb)
    }
}

/// 主进程管理器
pub struct MainProcessManager {
    // 进程注册表
    device_processes: Arc<RwLock<HashMap<DeviceId, Arc<Mutex<DeviceProcessHandle>>>>>,
    node_processes: Arc<RwLock<HashMap<ProcessId, Arc<Mutex<NodeProcessHandle>>>>>,
    
    // 生命周期管理器
    lifecycle_managers: Arc<RwLock<HashMap<ProcessId, Arc<Mutex<ProcessLifecycleManager>>>>>,
    
    // 资源管理
    cpu_allocator: Arc<CpuCoreAllocator>,
    memory_manager: Arc<Mutex<MemoryManager>>,
    
    // 通信管理
    ipc_channels: Arc<RwLock<HashMap<ProcessId, mpsc::UnboundedSender<String>>>>,
    
    // 配置与状态
    process_configs: Arc<RwLock<HashMap<ProcessId, ProcessConfig>>>,
    global_state: Arc<RwLock<GlobalState>>,
    
    // 监控
    process_monitors: Arc<RwLock<HashMap<ProcessId, Arc<Mutex<ProcessMonitor>>>>>,
    
    // 控制
    shutdown_requested: Arc<Mutex<bool>>,
}

impl MainProcessManager {
    /// 创建新的主进程管理器
    pub fn new() -> Self {
        Self {
            device_processes: Arc::new(RwLock::new(HashMap::default())),
            node_processes: Arc::new(RwLock::new(HashMap::default())),
            lifecycle_managers: Arc::new(RwLock::new(HashMap::default())),
            cpu_allocator: Arc::new(CpuCoreAllocator::new()),
            rayon_pool_manager: Arc::new(RayonPoolManager::new()),
            memory_manager: Arc::new(Mutex::new(MemoryManager::new())),
            ipc_channels: Arc::new(RwLock::new(HashMap::default())),
            process_configs: Arc::new(RwLock::new(HashMap::default())),
            global_state: Arc::new(RwLock::new(GlobalState::default())),
            process_monitors: Arc::new(RwLock::new(HashMap::default())),
            shutdown_requested: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 创建设备进程
    pub async fn create_device_process(
        &self,
        device_id: DeviceId,
        device_name: String,
        cpu_cores: usize,
        priority: ProcessPriority,
    ) -> ManagerResult<ProcessId> {
        let process_id = format!("device_{}", device_id);
        
        tracing::info!("创建设备进程: {} (设备: {})", process_id, device_name);
        
        // 创建进程配置
        let mut config = ProcessConfig::for_device(process_id.clone(), device_id, cpu_cores);
        config.priority = priority;
        
        // 验证配置
        config.validate().map_err(|e| ManagerError::ConfigError { 
            message: format!("设备进程配置无效: {}", e) 
        })?;
        
        // 分配CPU核心
        let allocation = self.cpu_allocator.allocate_cores(
            process_id.clone(),
            cpu_cores,
            crate::infrastructure::core::strategies::Priority::from(priority),
            crate::infrastructure::core::strategies::AllocationType::Exclusive,
        ).map_err(|e| ManagerError::ResourceError { 
            message: format!("CPU核心分配失败: {}", e) 
        })?;
        
        // 更新配置中的核心分配
        config.set_allocated_cores(allocation.allocated_cores.clone());
        
        // 分配内存
        if let Some(memory_limit) = config.memory_limit_mb {
            self.memory_manager.lock().unwrap().allocate_memory(
                process_id.clone(), 
                memory_limit
            ).map_err(|e| ManagerError::ResourceError { 
                message: format!("内存分配失败: {}", e) 
            })?;
        }
        
        // 创建Rayon线程池
        if let Some(ref rayon_config) = config.thread_pool_config {
            self.rayon_pool_manager.create_pool(
                process_id.clone(), 
                rayon_config.clone()
            ).map_err(|e| ManagerError::ResourceError { 
                message: format!("线程池创建失败: {}", e) 
            })?;
        }
        
        // 创建进程句柄
        let mut device_handle = DeviceProcessHandle::new(
            process_id.clone(),
            device_id,
            device_name,
            priority,
        );
        device_handle.set_allocated_cores(allocation.allocated_cores);
        
        // 创建生命周期管理器
        let lifecycle_manager = ProcessLifecycleManager::new(
            process_id.clone(),
            config.clone(),
            device_handle.base.clone(),
        );
        
        // 创建进程监控器
        let mut monitor = ProcessMonitor::new(process_id.clone());
        monitor.set_allocated_cores(allocation.allocated_cores);
        
        // 保存到注册表
        {
            let mut device_processes = self.device_processes.write().unwrap();
            device_processes.insert(device_id, Arc::new(Mutex::new(device_handle)));
        }
        
        {
            let mut configs = self.process_configs.write().unwrap();
            configs.insert(process_id.clone(), config);
        }
        
        {
            let mut managers = self.lifecycle_managers.write().unwrap();
            managers.insert(process_id.clone(), Arc::new(Mutex::new(lifecycle_manager)));
        }
        
        {
            let mut monitors = self.process_monitors.write().unwrap();
            monitors.insert(process_id.clone(), Arc::new(Mutex::new(monitor)));
        }
        
        // 更新全局状态
        self.update_global_state().await;
        
        tracing::info!("设备进程创建成功: {}", process_id);
        Ok(process_id)
    }
    
    /// 创建Node.js脚本进程
    pub async fn create_node_process(
        &self,
        script_path: PathBuf,
        cpu_cores: usize,
        priority: ProcessPriority,
    ) -> ManagerResult<ProcessId> {
        let script_id = script_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let process_id = format!("node_{}", Uuid::new_v4());
        
        tracing::info!("创建Node.js进程: {} (脚本: {})", process_id, script_id);
        
        // 创建进程配置
        let mut config = ProcessConfig::for_node_script(process_id.clone(), script_path.clone(), cpu_cores);
        config.priority = priority;
        
        // 验证配置
        config.validate().map_err(|e| ManagerError::ConfigError { 
            message: format!("Node.js进程配置无效: {}", e) 
        })?;
        
        // 分配资源（类似设备进程的流程）
        let allocation = self.cpu_allocator.allocate_cores(
            process_id.clone(),
            cpu_cores,
            crate::infrastructure::core::strategies::Priority::from(priority),
            crate::infrastructure::core::strategies::AllocationType::Shared,
        ).map_err(|e| ManagerError::ResourceError { 
            message: format!("CPU核心分配失败: {}", e) 
        })?;
        
        config.set_allocated_cores(allocation.allocated_cores.clone());
        
        // 分配内存
        if let Some(memory_limit) = config.memory_limit_mb {
            self.memory_manager.lock().unwrap().allocate_memory(
                process_id.clone(), 
                memory_limit
            ).map_err(|e| ManagerError::ResourceError { 
                message: format!("内存分配失败: {}", e) 
            })?;
        }
        
        // 创建进程句柄
        let mut node_handle = NodeProcessHandle::new(
            process_id.clone(),
            script_path,
            priority,
        );
        node_handle.set_allocated_cores(allocation.allocated_cores);
        
        // 创建其他组件（类似设备进程）
        let lifecycle_manager = ProcessLifecycleManager::new(
            process_id.clone(),
            config.clone(),
            node_handle.base.clone(),
        );
        
        let mut monitor = ProcessMonitor::new(process_id.clone());
        monitor.set_allocated_cores(allocation.allocated_cores);
        
        // 保存到注册表
        {
            let mut node_processes = self.node_processes.write().unwrap();
            node_processes.insert(process_id.clone(), Arc::new(Mutex::new(node_handle)));
        }
        
        {
            let mut configs = self.process_configs.write().unwrap();
            configs.insert(process_id.clone(), config);
        }
        
        {
            let mut managers = self.lifecycle_managers.write().unwrap();
            managers.insert(process_id.clone(), Arc::new(Mutex::new(lifecycle_manager)));
        }
        
        {
            let mut monitors = self.process_monitors.write().unwrap();
            monitors.insert(process_id.clone(), Arc::new(Mutex::new(monitor)));
        }
        
        // 更新全局状态
        self.update_global_state().await;
        
        tracing::info!("Node.js进程创建成功: {}", process_id);
        Ok(process_id)
    }
    
    /// 启动进程
    pub async fn start_process(&self, process_id: &ProcessId) -> ManagerResult<()> {
        tracing::info!("启动进程: {}", process_id);
        
        let lifecycle_manager = {
            let managers = self.lifecycle_managers.read().unwrap();
            managers.get(process_id).cloned()
        };
        
        if let Some(manager) = lifecycle_manager {
            let mut manager = manager.lock().unwrap();
            manager.start().await.map_err(|e| ManagerError::ProcessError { 
                message: format!("进程启动失败: {}", e) 
            })?;
            
            self.update_global_state().await;
            Ok(())
        } else {
            Err(ManagerError::ProcessError { 
                message: format!("进程不存在: {}", process_id) 
            })
        }
    }
    
    /// 停止进程
    pub async fn stop_process(&self, process_id: &ProcessId, force: bool) -> ManagerResult<()> {
        tracing::info!("停止进程: {} (force: {})", process_id, force);
        
        let lifecycle_manager = {
            let managers = self.lifecycle_managers.read().unwrap();
            managers.get(process_id).cloned()
        };
        
        if let Some(manager) = lifecycle_manager {
            let mut manager = manager.lock().unwrap();
            manager.stop(force).await.map_err(|e| ManagerError::ProcessError { 
                message: format!("进程停止失败: {}", e) 
            })?;
            
            self.update_global_state().await;
            Ok(())
        } else {
            Err(ManagerError::ProcessError { 
                message: format!("进程不存在: {}", process_id) 
            })
        }
    }
    
    /// 重启进程
    pub async fn restart_process(&self, process_id: &ProcessId) -> ManagerResult<()> {
        tracing::info!("重启进程: {}", process_id);
        
        let lifecycle_manager = {
            let managers = self.lifecycle_managers.read().unwrap();
            managers.get(process_id).cloned()
        };
        
        if let Some(manager) = lifecycle_manager {
            let mut manager = manager.lock().unwrap();
            manager.restart().await.map_err(|e| ManagerError::ProcessError { 
                message: format!("进程重启失败: {}", e) 
            })?;
            
            self.update_global_state().await;
            Ok(())
        } else {
            Err(ManagerError::ProcessError { 
                message: format!("进程不存在: {}", process_id) 
            })
        }
    }
    
    /// 删除进程
    pub async fn remove_process(&self, process_id: &ProcessId) -> ManagerResult<()> {
        tracing::info!("删除进程: {}", process_id);
        
        // 先停止进程
        if let Ok(_) = self.stop_process(process_id, true).await {
            // 释放资源
            self.cpu_allocator.deallocate_cores(process_id).ok();
            self.memory_manager.lock().unwrap().deallocate_memory(process_id);
            self.rayon_pool_manager.destroy_pool(process_id).ok();
            
            // 从注册表中移除
            self.lifecycle_managers.write().unwrap().remove(process_id);
            self.process_configs.write().unwrap().remove(process_id);
            self.process_monitors.write().unwrap().remove(process_id);
            self.ipc_channels.write().unwrap().remove(process_id);
            
            // 根据类型移除
            if process_id.starts_with("device_") {
                if let Ok(device_id) = Uuid::parse_str(&process_id[7..]) {
                    self.device_processes.write().unwrap().remove(&device_id);
                }
            } else if process_id.starts_with("node_") {
                self.node_processes.write().unwrap().remove(process_id);
            }
            
            self.update_global_state().await;
            tracing::info!("进程删除成功: {}", process_id);
            Ok(())
        } else {
            Err(ManagerError::ProcessError { 
                message: format!("无法停止进程: {}", process_id) 
            })
        }
    }
    
    /// 获取所有进程状态
    pub fn get_all_process_status(&self) -> Vec<PerformanceSummary> {
        let monitors = self.process_monitors.read().unwrap();
        monitors.values()
            .map(|monitor| {
                let monitor = monitor.lock().unwrap();
                monitor.get_performance_summary()
            })
            .collect()
    }
    
    /// 获取全局状态
    pub fn get_global_state(&self) -> GlobalState {
        let state = self.global_state.read().unwrap();
        state.clone()
    }
    
    /// 获取进程详情
    pub fn get_process_details(&self, process_id: &ProcessId) -> Option<String> {
        if let Some(monitor) = self.process_monitors.read().unwrap().get(process_id) {
            let monitor = monitor.lock().unwrap();
            let metrics = monitor.get_current_metrics();
            Some(format!("{:#?}", metrics))
        } else {
            None
        }
    }
    
    /// 关闭管理器
    pub async fn shutdown(&self) -> ManagerResult<()> {
        tracing::info!("开始关闭进程管理器");
        
        *self.shutdown_requested.lock().unwrap() = true;
        
        // 获取所有进程ID
        let process_ids: Vec<ProcessId> = {
            let managers = self.lifecycle_managers.read().unwrap();
            managers.keys().cloned().collect()
        };
        
        // 并发停止所有进程
        let futures: Vec<_> = process_ids.iter()
            .map(|pid| self.stop_process(pid, false))
            .collect();
        
        // 等待所有进程停止
        for (i, future) in futures.into_iter().enumerate() {
            if let Err(e) = future.await {
                tracing::error!("停止进程 {} 失败: {}", process_ids[i], e);
            }
        }
        
        tracing::info!("进程管理器关闭完成");
        Ok(())
    }
    
    // 私有辅助方法
    
    async fn update_global_state(&self) {
        let mut state = self.global_state.write().unwrap();
        
        let managers = self.lifecycle_managers.read().unwrap();
        state.total_processes = managers.len();
        
        let mut running_count = 0;
        let mut failed_count = 0;
        
        for manager in managers.values() {
            if let Ok(current_state) = manager.lock().unwrap().get_current_state() {
                match current_state {
                    ProcessState::Running => running_count += 1,
                    ProcessState::Failed | ProcessState::Crashed => failed_count += 1,
                    _ => {},
                }
            }
        }
        
        state.running_processes = running_count;
        state.failed_processes = failed_count;
        
        // 更新资源使用统计
        let cpu_stats = self.cpu_allocator.get_stats();
        state.total_allocated_cores = cpu_stats.total_cores_allocated;
        
        let memory_manager = self.memory_manager.lock().unwrap();
        state.total_memory_usage_mb = memory_manager.allocated_memory_mb;
        
        state.last_updated = SystemTime::now();
    }
}

// Priority转换
impl From<ProcessPriority> for crate::infrastructure::core::strategies::Priority {
    fn from(priority: ProcessPriority) -> Self {
        match priority {
            ProcessPriority::Low => crate::infrastructure::core::strategies::Priority::Low,
            ProcessPriority::BelowNormal | ProcessPriority::Normal => crate::infrastructure::core::strategies::Priority::Normal,
            ProcessPriority::AboveNormal | ProcessPriority::High => crate::infrastructure::core::strategies::Priority::High,
            ProcessPriority::Realtime => crate::infrastructure::core::strategies::Priority::Critical,
        }
    }
}

impl Default for MainProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = MainProcessManager::new();
        let state = manager.get_global_state();
        assert_eq!(state.total_processes, 0);
    }

    #[tokio::test]
    async fn test_device_process_creation() {
        let manager = MainProcessManager::new();
        let device_id = Uuid::new_v4();
        
        let result = manager.create_device_process(
            device_id,
            "Test Device".to_string(),
            2,
            ProcessPriority::Normal,
        ).await;
        
        assert!(result.is_ok());
        
        let state = manager.get_global_state();
        assert_eq!(state.total_processes, 1);
    }
}