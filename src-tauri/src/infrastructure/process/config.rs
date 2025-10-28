// 进程配置模块
// 定义进程配置、资源约束和重启策略

use crate::infrastructure::core::{ProcessId, DeviceId, Serialize, Deserialize, HashMap};
use crate::infrastructure::process::handle::{ProcessPriority, ProcessType};
use crate::infrastructure::core::rayon_pool::RayonConfig;
use std::time::Duration;
use std::path::PathBuf;

/// 进程配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    // 基本配置
    pub process_id: ProcessId,
    pub process_type: ProcessType,
    pub priority: ProcessPriority,
    
    // 资源配置
    pub cpu_core_count: usize,
    pub allocated_cores: Option<Vec<usize>>,
    pub memory_limit_mb: Option<usize>,
    pub disk_quota_mb: Option<usize>,
    
    // 运行时配置
    pub working_directory: Option<PathBuf>,
    pub environment_vars: HashMap<String, String>,
    pub startup_timeout: Duration,
    pub shutdown_timeout: Duration,
    
    // 重启策略
    pub restart_policy: RestartPolicy,
    
    // 资源约束
    pub resource_constraints: ResourceConstraints,
    
    // Rayon线程池配置
    pub thread_pool_config: Option<RayonConfig>,
    
    // 日志配置
    pub log_config: ProcessLogConfig,
    
    // 健康检查配置
    pub health_check: HealthCheckConfig,
}

impl ProcessConfig {
    /// 创建设备进程配置
    pub fn for_device(
        process_id: ProcessId,
        device_id: DeviceId,
        cpu_cores: usize,
    ) -> Self {
        Self {
            process_id: process_id.clone(),
            process_type: ProcessType::Device { device_id },
            priority: ProcessPriority::Normal,
            cpu_core_count: cpu_cores,
            allocated_cores: None,
            memory_limit_mb: Some(1024), // 默认1GB
            disk_quota_mb: Some(5120),   // 默认5GB
            working_directory: None,
            environment_vars: HashMap::new(),
            startup_timeout: Duration::from_secs(60),
            shutdown_timeout: Duration::from_secs(30),
            restart_policy: RestartPolicy::OnFailure {
                max_attempts: 3,
                delay: Duration::from_secs(5),
            },
            resource_constraints: ResourceConstraints::default(),
            thread_pool_config: Some(RayonConfig::from_allocated_cores(&process_id, &[])),
            log_config: ProcessLogConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
    
    /// 创建Node.js进程配置
    pub fn for_node_script(
        process_id: ProcessId,
        script_path: PathBuf,
        cpu_cores: usize,
    ) -> Self {
        let script_id = script_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        Self {
            process_id: process_id.clone(),
            process_type: ProcessType::Node { script_id },
            priority: ProcessPriority::BelowNormal,
            cpu_core_count: cpu_cores,
            allocated_cores: None,
            memory_limit_mb: Some(512), // 默认512MB
            disk_quota_mb: Some(1024),  // 默认1GB
            working_directory: script_path.parent().map(|p| p.to_path_buf()),
            environment_vars: HashMap::new(),
            startup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(15),
            restart_policy: RestartPolicy::OnFailure {
                max_attempts: 2,
                delay: Duration::from_secs(3),
            },
            resource_constraints: ResourceConstraints::for_node_script(),
            thread_pool_config: Some(RayonConfig::from_allocated_cores(&process_id, &[])),
            log_config: ProcessLogConfig::for_script(),
            health_check: HealthCheckConfig::for_script(),
        }
    }
    
    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.cpu_core_count == 0 {
            return Err("CPU核心数不能为0".to_string());
        }
        
        if let Some(ref cores) = self.allocated_cores {
            if cores.len() != self.cpu_core_count {
                return Err(format!(
                    "分配的核心数({})与配置的核心数({})不匹配",
                    cores.len(),
                    self.cpu_core_count
                ));
            }
        }
        
        if let Some(memory_limit) = self.memory_limit_mb {
            if memory_limit < 64 {
                return Err("内存限制不能小于64MB".to_string());
            }
        }
        
        self.resource_constraints.validate()?;
        
        Ok(())
    }
    
    /// 设置分配的CPU核心
    pub fn set_allocated_cores(&mut self, cores: Vec<usize>) {
        self.allocated_cores = Some(cores.clone());
        
        // 更新线程池配置
        if let Some(ref mut config) = self.thread_pool_config {
            *config = RayonConfig::from_allocated_cores(&self.process_id, &cores);
        }
    }
    
    /// 添加环境变量
    pub fn add_environment_var(&mut self, key: String, value: String) {
        self.environment_vars.insert(key, value);
    }
    
    /// 获取启动命令
    pub fn get_startup_command(&self) -> Result<std::process::Command, String> {
        match &self.process_type {
            ProcessType::Device { device_id } => {
                // 设备进程启动命令
                let mut cmd = std::process::Command::new("autodaily-device");
                cmd.arg("--device-id").arg(device_id.to_string());
                cmd.arg("--process-id").arg(&self.process_id);
                
                if let Some(ref cores) = self.allocated_cores {
                    cmd.arg("--cpu-cores").arg(
                        cores.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",")
                    );
                }
                
                Ok(cmd)
            },
            ProcessType::Node { script_id } => {
                // Node.js脚本启动命令
                let mut cmd = std::process::Command::new("node");
                if let Some(ref wd) = self.working_directory {
                    if let Some(script_file) = wd.join(format!("{}.js", script_id)).to_str() {
                        cmd.arg(script_file);
                    } else {
                        return Err("无效的脚本路径".to_string());
                    }
                } else {
                    return Err("未设置工作目录".to_string());
                }
                
                Ok(cmd)
            },
        }
    }
}

/// 重启策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// 从不重启
    Never,
    
    /// 总是重启
    Always {
        delay: Duration,
    },
    
    /// 仅在失败时重启
    OnFailure {
        max_attempts: u32,
        delay: Duration,
    },
    
    /// 仅在崩溃时重启
    OnCrash {
        max_attempts: u32,
        delay: Duration,
        exponential_backoff: bool,
    },
}

impl Default for RestartPolicy {
    fn default() -> Self {
        RestartPolicy::OnFailure {
            max_attempts: 3,
            delay: Duration::from_secs(5),
        }
    }
}

impl RestartPolicy {
    /// 检查是否应该重启
    pub fn should_restart(&self, restart_count: u32, exit_code: Option<i32>) -> bool {
        match self {
            RestartPolicy::Never => false,
            RestartPolicy::Always { .. } => true,
            RestartPolicy::OnFailure { max_attempts, .. } => {
                restart_count < *max_attempts && exit_code.map_or(true, |code| code != 0)
            },
            RestartPolicy::OnCrash { max_attempts, .. } => {
                restart_count < *max_attempts && exit_code.map_or(true, |code| code < 0)
            },
        }
    }
    
    /// 获取重启延迟
    pub fn get_restart_delay(&self, restart_count: u32) -> Duration {
        match self {
            RestartPolicy::Never => Duration::from_secs(0),
            RestartPolicy::Always { delay } => *delay,
            RestartPolicy::OnFailure { delay, .. } => *delay,
            RestartPolicy::OnCrash { delay, exponential_backoff, .. } => {
                if *exponential_backoff {
                    Duration::from_secs(delay.as_secs() * 2_u64.pow(restart_count.min(5)))
                } else {
                    *delay
                }
            },
        }
    }
}

/// 资源约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraints {
    // CPU约束
    pub max_cpu_percent: Option<f64>,
    pub cpu_quota_period: Option<Duration>,
    
    // 内存约束
    pub max_memory_mb: Option<usize>,
    pub memory_swap_limit_mb: Option<usize>,
    pub oom_kill_disable: bool,
    
    // 磁盘IO约束
    pub max_disk_read_mb_per_sec: Option<usize>,
    pub max_disk_write_mb_per_sec: Option<usize>,
    pub max_disk_iops: Option<u32>,
    
    // 网络约束
    pub max_network_requests_per_sec: Option<u32>,
    pub max_bandwidth_mb_per_sec: Option<usize>,
    
    // 进程约束
    pub max_open_files: Option<u32>,
    pub max_threads: Option<u32>,
    pub max_child_processes: Option<u32>,
    
    // 任务约束
    pub max_concurrent_tasks: Option<usize>,
    pub max_task_queue_size: Option<usize>,
    pub task_timeout: Option<Duration>,
}

impl Default for ResourceConstraints {
    fn default() -> Self {
        Self {
            max_cpu_percent: Some(80.0),
            cpu_quota_period: Some(Duration::from_millis(100)),
            max_memory_mb: Some(1024),
            memory_swap_limit_mb: Some(2048),
            oom_kill_disable: false,
            max_disk_read_mb_per_sec: Some(100),
            max_disk_write_mb_per_sec: Some(50),
            max_disk_iops: Some(1000),
            max_network_requests_per_sec: Some(100),
            max_bandwidth_mb_per_sec: Some(10),
            max_open_files: Some(1024),
            max_threads: Some(100),
            max_child_processes: Some(10),
            max_concurrent_tasks: Some(50),
            max_task_queue_size: Some(1000),
            task_timeout: Some(Duration::from_secs(300)),
        }
    }
}

impl ResourceConstraints {
    /// 为Node.js脚本创建约束
    pub fn for_node_script() -> Self {
        Self {
            max_cpu_percent: Some(50.0),
            max_memory_mb: Some(512),
            memory_swap_limit_mb: Some(1024),
            max_threads: Some(50),
            max_concurrent_tasks: Some(20),
            max_task_queue_size: Some(100),
            task_timeout: Some(Duration::from_secs(60)),
            ..Default::default()
        }
    }
    
    /// 验证约束配置
    pub fn validate(&self) -> Result<(), String> {
        if let Some(cpu_percent) = self.max_cpu_percent {
            if cpu_percent <= 0.0 || cpu_percent > 100.0 {
                return Err("CPU使用率限制必须在0-100%之间".to_string());
            }
        }
        
        if let Some(memory_mb) = self.max_memory_mb {
            if memory_mb < 64 {
                return Err("内存限制不能小于64MB".to_string());
            }
        }
        
        if let Some(max_threads) = self.max_threads {
            if max_threads == 0 {
                return Err("最大线程数不能为0".to_string());
            }
        }
        
        Ok(())
    }
}

/// 进程日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLogConfig {
    pub log_level: String,
    pub log_file: Option<PathBuf>,
    pub max_log_size_mb: usize,
    pub log_rotation: bool,
    pub stdout_capture: bool,
    pub stderr_capture: bool,
}

impl Default for ProcessLogConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file: None,
            max_log_size_mb: 100,
            log_rotation: true,
            stdout_capture: true,
            stderr_capture: true,
        }
    }
}

impl ProcessLogConfig {
    pub fn for_script() -> Self {
        Self {
            log_level: "warn".to_string(),
            max_log_size_mb: 50,
            ..Default::default()
        }
    }
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub check_interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub check_type: HealthCheckType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    Heartbeat,           // 心跳检查
    MemoryUsage,        // 内存使用检查
    CpuUsage,           // CPU使用检查
    ResponseTime,       // 响应时间检查
    Custom(String),     // 自定义检查
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 1,
            check_type: HealthCheckType::Heartbeat,
        }
    }
}

impl HealthCheckConfig {
    pub fn for_script() -> Self {
        Self {
            check_interval: Duration::from_secs(60),
            failure_threshold: 2,
            check_type: HealthCheckType::MemoryUsage,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_device_config_creation() {
        let device_id = Uuid::new_v4();
        let config = ProcessConfig::for_device(
            "test_device".to_string(),
            device_id,
            4
        );
        
        assert_eq!(config.cpu_core_count, 4);
        assert!(matches!(config.process_type, ProcessType::Device { .. }));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_restart_policy() {
        let policy = RestartPolicy::OnFailure {
            max_attempts: 3,
            delay: Duration::from_secs(5),
        };
        
        // 应该重启（失败次数未达到最大值）
        assert!(policy.should_restart(2, Some(1)));
        
        // 不应该重启（达到最大重启次数）
        assert!(!policy.should_restart(3, Some(1)));
        
        // 不应该重启（正常退出）
        assert!(!policy.should_restart(1, Some(0)));
    }

    #[test]
    fn test_resource_constraints_validation() {
        let mut constraints = ResourceConstraints::default();
        assert!(constraints.validate().is_ok());
        
        // 测试无效的CPU限制
        constraints.max_cpu_percent = Some(150.0);
        assert!(constraints.validate().is_err());
        
        // 测试无效的内存限制
        constraints.max_cpu_percent = Some(80.0);
        constraints.max_memory_mb = Some(32);
        assert!(constraints.validate().is_err());
    }
}