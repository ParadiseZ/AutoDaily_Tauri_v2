use crate::infrastructure::entities::devices::device_info::DeviceInfo;
use crate::domain::entities::scheduler::script_scheduler::ScriptScheduler;
use crate::infrastructure::performance::ProcessConfig;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::scripts::script_info::{ScriptId, ScriptInfo, ScriptStatus};

/// 设备状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// 离线状态
    Offline,
    /// 在线空闲
    Idle,
    /// 运行中
    Running,
    /// 错误状态
    Error(String),
    /// 正在初始化
    Initializing,
    /// 正在停止
    Stopping,
}

/// 设备性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePerformanceConfig {
    /// 分配的CPU核心数
    pub allocated_cores: u32,
    /// 分配的物理核心列表
    pub physical_core_ids: Vec<usize>,
    /// 分配的逻辑核心列表
    pub logical_core_ids: Vec<usize>,
    /// 最大内存使用量（MB）
    pub max_memory_mb: u64,
    /// 进程优先级
    pub process_priority: crate::infrastructure::performance::ProcessPriority,
}

impl Default for DevicePerformanceConfig {
    fn default() -> Self {
        Self {
            allocated_cores: 4,
            physical_core_ids: Vec::new(),
            logical_core_ids: Vec::new(),
            max_memory_mb: 1024,
            process_priority: crate::infrastructure::performance::ProcessPriority::Normal,
        }
    }
}

/// 设备统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    /// 设备启动时间
    pub started_at: Option<DateTime<Local>>,
    /// 总运行时间（秒）
    pub total_uptime_seconds: u64,
    /// 执行的脚本总数
    pub total_scripts_executed: u64,
    /// 成功执行的脚本数
    pub successful_scripts: u64,
    /// 失败的脚本数
    pub failed_scripts: u64,
    /// 当前CPU使用率（0-100%）
    pub cpu_usage_percent: f32,
    /// 当前内存使用量（MB）
    pub memory_usage_mb: u64,
    /// 平均脚本执行时间（毫秒）
    pub average_script_execution_time_ms: u64,
    /// 最后活动时间
    pub last_activity_time: DateTime<Local>,
}

impl Default for DeviceStats {
    fn default() -> Self {
        Self {
            started_at: None,
            total_uptime_seconds: 0,
            total_scripts_executed: 0,
            successful_scripts: 0,
            failed_scripts: 0,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            average_script_execution_time_ms: 0,
            last_activity_time: Local::now(),
        }
    }
}

/// 设备上下文信息
/// 
/// 重新设计的设备上下文，支持进程模式和调度引擎
#[derive(Debug)]
pub struct DeviceContext {
    /// 设备信息
    pub device_info: DeviceInfo,
    /// 设备当前状态
    pub status: DeviceStatus,
    /// 分配到该设备的脚本列表 (ScriptId -> Script)
    pub scripts: HashMap<ScriptId, ScriptInfo>,
    // 注意：调度器现在在主进程的DeviceManager中，不再是每设备独立
    /// 设备进程配置
    pub process_config: ProcessConfig,
    /// 设备进程ID（如果已启动）
    pub process_id: Option<String>,
    /// 性能配置
    pub performance_config: DevicePerformanceConfig,
    /// 统计信息
    pub stats: DeviceStats,
    /// 设备创建时间
    pub created_at: DateTime<Local>,
    /// 最后更新时间
    pub updated_at: DateTime<Local>,
    /// 是否启用
    pub enabled: bool,
    /// 设备标签
    pub tags: Vec<String>,
    ///
}

impl DeviceContext {
    /// 创建新的设备上下文
    pub fn new(
        device_id: String,
        device_info: DeviceInfo,
        process_config: ProcessConfig,
    ) -> Self {
        let now = Local::now();
        Self {
            device_id,
            device_info,
            status: DeviceStatus::Offline,
            scripts: HashMap::new(),
            // scheduler 已移到主进程 DeviceManager 中
            process_config,
            process_id: None,
            performance_config: DevicePerformanceConfig::default(),
            stats: DeviceStats::default(),
            created_at: now,
            updated_at: now,
            enabled: true,
            tags: Vec::new(),
        }
    }

    /// 启动设备进程
    pub async fn start_device_process(
        &mut self,
        process_manager: &crate::infrastructure::performance::ProcessManager,
    ) -> AppResult<()> {
        if self.process_id.is_some() {
            return Err(AppError::ConfigError("设备进程已启动".to_string()));
        }

        self.status = DeviceStatus::Initializing;
        
        // 启动设备进程
        let process_id = process_manager
            .spawn_process_with_affinity(self.process_config.clone())
            .map_err(|e| AppError::InternalError(format!("启动设备进程失败: {}", e)))?;

        self.process_id = Some(process_id.clone());
        self.status = DeviceStatus::Idle;
        self.stats.started_at = Some(Local::now());
        self.updated_at = Local::now();

        // 设备进程注册现在由DeviceManager处理

        tracing::info!("设备进程已启动: device_id={}, process_id={:?}", 
                      self.device_id, self.process_id);
        Ok(())
    }

    /// 停止设备进程
    pub async fn stop_device_process(
        &mut self,
        process_manager: &crate::infrastructure::performance::ProcessManager,
    ) -> AppResult<()> {
        if let Some(process_id) = &self.process_id {
            self.status = DeviceStatus::Stopping;

            // 终止进程
            process_manager
                .terminate_process(process_id)
                .map_err(|e| AppError::InternalError(format!("终止设备进程失败: {}", e)))?;

            // 设备进程取消注册现在由DeviceManager处理

            self.process_id = None;
            self.status = DeviceStatus::Offline;
            self.updated_at = Local::now();

            tracing::info!("设备进程已停止: device_id={}", self.device_id);
        }

        Ok(())
    }

    /// 添加脚本到设备
    pub async fn add_script(&mut self, script: ScriptInfo) -> AppResult<()> {
        let script_id = script.id.clone();
        
        // 检查脚本是否已存在
        if self.scripts.contains_key(&script_id) {
            return Err(AppError::ConfigError(format!(
                "脚本已存在于设备中: device_id={}, script_id={}",
                self.device_id, script_id
            )));
        }

        // 添加到本地脚本列表
        self.scripts.insert(script_id.clone(), script.clone());
        
        // 脚本注册到调度器现在由DeviceManager处理

        self.updated_at = Local::now();
        tracing::info!("脚本已添加到设备: device_id={}, script_id={}", 
                      self.device_id, script_id);
        Ok(())
    }

    /// 从设备移除脚本
    pub async fn remove_script(&mut self, script_id: &ScriptId) -> AppResult<()> {
        // 从本地脚本列表移除
        self.scripts.remove(script_id);
        
        // 从调度器取消注册现在由DeviceManager处理

        self.updated_at = Local::now();
        tracing::info!("脚本已从设备移除: device_id={}, script_id={}", 
                      self.device_id, script_id);
        Ok(())
    }

    /// 启动脚本执行
    pub async fn start_script(&mut self, script_id: &ScriptId) -> AppResult<()> {
        // 检查脚本是否存在
        if !self.scripts.contains_key(script_id) {
            return Err(AppError::ConfigError(format!(
                "脚本不存在于设备中: device_id={}, script_id={}",
                self.device_id, script_id
            )));
        }

        // 更新本地脚本状态（实际启动由DeviceManager处理）
        if let Some(script) = self.scripts.get_mut(script_id) {
            script.update_status(ScriptStatus::Starting);
        }

        self.status = DeviceStatus::Running;
        self.updated_at = Local::now();
        
        tracing::info!("脚本执行已启动: device_id={}, script_id={}", 
                      self.device_id, script_id);
        Ok(())
    }

    /// 停止脚本执行
    pub async fn stop_script(&mut self, script_id: &ScriptId) -> AppResult<()> {
        // 更新本地脚本状态（实际停止由DeviceManager处理）
        if let Some(script) = self.scripts.get_mut(script_id) {
            script.update_status(ScriptStatus::Stopping);
        }

        // 检查是否还有其他脚本在运行
        let has_running_scripts = self.scripts.values()
            .any(|script| matches!(script.status, ScriptStatus::Running));

        if !has_running_scripts {
            self.status = DeviceStatus::Idle;
        }

        self.updated_at = Local::now();
        
        tracing::info!("脚本执行已停止: device_id={}, script_id={}", 
                      self.device_id, script_id);
        Ok(())
    }

    /// 更新脚本状态（由主进程调用，同步到设备进程）
    pub async fn update_script_status(&mut self, script_id: &ScriptId, status: ScriptStatus) -> AppResult<()> {
        // 更新本地脚本状态
        if let Some(script) = self.scripts.get_mut(script_id) {
            let old_status = script.status.clone();
            script.update_status(status.clone());
            
            tracing::info!("设备中脚本状态已更新: device_id={}, script_id={}, {} -> {:?}", 
                          self.device_id, script_id, format!("{:?}", old_status), status);
        }

        // 状态同步现在由DeviceManager处理

        self.updated_at = Local::now();
        Ok(())
    }

    /// 获取脚本状态
    pub async fn get_script_status(&self, script_id: &ScriptId) -> Option<ScriptStatus> {
        if let Some(script) = self.scripts.get(script_id) {
            Some(script.status.clone())
        } else {
            None
        }
    }

    /// 获取所有脚本状态
    pub async fn get_all_script_status(&self) -> HashMap<ScriptId, ScriptStatus> {
        self.scripts
            .iter()
            .map(|(id, script)| (id.clone(), script.status.clone()))
            .collect()
    }

    /// 更新设备统计信息
    pub fn update_stats(&mut self, cpu_usage: f32, memory_usage: u64) {
        self.stats.cpu_usage_percent = cpu_usage;
        self.stats.memory_usage_mb = memory_usage;
        self.stats.last_activity_time = Local::now();
        self.updated_at = Local::now();

        // 更新运行时间
        if let Some(started_at) = self.stats.started_at {
            let uptime = Local::now().signed_duration_since(started_at);
            self.stats.total_uptime_seconds = uptime.num_seconds() as u64;
        }
    }

    /// 记录脚本执行结果
    pub fn record_script_execution(&mut self, script_id: &ScriptId, success: bool, execution_time_ms: u64) {
        self.stats.total_scripts_executed += 1;
        
        if success {
            self.stats.successful_scripts += 1;
        } else {
            self.stats.failed_scripts += 1;
        }

        // 更新平均执行时间
        let total_time = self.stats.average_script_execution_time_ms 
            * (self.stats.total_scripts_executed - 1) 
            + execution_time_ms;
        self.stats.average_script_execution_time_ms = total_time / self.stats.total_scripts_executed;

        self.updated_at = Local::now();
        
        tracing::info!("脚本执行结果已记录: device_id={}, script_id={}, success={}, duration={}ms",
                      self.device_id, script_id, success, execution_time_ms);
    }

    /// 检查设备是否空闲
    pub fn is_idle(&self) -> bool {
        matches!(self.status, DeviceStatus::Idle) && 
        !self.scripts.values().any(|script| matches!(script.status, ScriptStatus::Running))
    }

    /// 获取设备概览信息
    pub fn get_overview(&self) -> String {
        format!(
            "Device[id={}, status={:?}, scripts={}, process_id={:?}, cpu={}%, memory={}MB]",
            self.device_id,
            self.status,
            self.scripts.len(),
            self.process_id,
            self.stats.cpu_usage_percent,
            self.stats.memory_usage_mb
        )
    }

    /// 获取脚本列表
    pub fn get_scripts(&self) -> &HashMap<ScriptId, ScriptInfo> {
        &self.scripts
    }

    /// 检查脚本是否正在运行
    pub fn is_script_running(&self, script_id: &ScriptId) -> bool {
        if let Some(script) = self.scripts.get(script_id) {
            matches!(script.status, ScriptStatus::Running)
        } else {
            false
        }
    }

    /// 获取正在运行的脚本数量
    pub fn get_running_scripts_count(&self) -> usize {
        self.scripts.values()
            .filter(|script| matches!(script.status, ScriptStatus::Running))
            .count()
    }
}