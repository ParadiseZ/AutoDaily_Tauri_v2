use std::process::{Command, Child, Stdio};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use tracing::{info, error, warn};
use serde::{Serialize, Deserialize};

/// 进程管理器，负责管理子进程和CPU核心分配
#[derive(Clone, Debug)]
pub struct ProcessManager {
    /// 存储活跃进程的信息
    active_processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
    /// 系统CPU核心数
    cpu_count: usize,
}

/// 进程信息结构体
#[derive(Debug)]
struct ProcessInfo {
    /// 进程句柄
    child: Child,
    /// 分配的CPU核心ID（单核心模式，向后兼容）
    core_id: Option<usize>,
    /// 分配的CPU核心ID列表（多核心模式）
    core_ids: Vec<usize>,
    /// 进程名称
    name: String,
    /// 命令行
    command: String,
}

/// 进程配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// 进程名称
    pub name: String,
    /// 要执行的程序路径
    pub program: String,
    /// 命令行参数
    pub args: Vec<String>,
    /// 工作目录（可选）
    pub working_dir: Option<String>,
    /// 环境变量（可选）
    pub env_vars: Option<HashMap<String, String>>,
    /// 指定的CPU核心ID（可选，单核心模式）
    pub core_id: Option<usize>,
    /// 指定的CPU核心ID列表（可选，多核心模式）
    pub core_ids: Option<Vec<usize>>,
    /// 进程优先级（可选）
    pub priority: Option<ProcessPriority>,
}

/// 进程优先级枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProcessPriority {
    Low,
    Normal,
    High,
    RealTime,
}

/// 进程状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub id: String,
    pub name: String,
    pub pid: u32,
    pub core_id: Option<usize>,
    pub command: String,
    pub is_running: bool,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    /// 创建新的进程管理器实例
    pub fn new() -> Self {
        let logical_cpu_count = num_cpus::get();
        let physical_cpu_count = num_cpus::get_physical();
        
        info!("初始化进程管理器，检测到 {} 个物理CPU核心，{} 个逻辑CPU核心", 
              physical_cpu_count, logical_cpu_count);
        
        Self {
            active_processes: Arc::new(Mutex::new(HashMap::new())),
            cpu_count: logical_cpu_count, // 保持向后兼容，亲和性设置基于逻辑核心
        }
    }

    /// 获取系统逻辑CPU核心数（用于亲和性设置）
    pub fn get_logical_cpu_count(&self) -> usize {
        self.cpu_count
    }
    
    /// 获取系统物理CPU核心数（用于ORT线程配置）
    pub fn get_physical_cpu_count(&self) -> usize {
        num_cpus::get_physical()
    }
    
    /// 获取系统CPU核心数（向后兼容）
    pub fn get_cpu_count(&self) -> usize {
        self.cpu_count
    }

    /// 获取可用的逻辑CPU核心ID列表
    pub fn get_available_logical_core_ids(&self) -> Vec<usize> {
        (0..self.cpu_count).collect()
    }
    
    /// 获取可用的CPU核心ID列表（向后兼容）
    pub fn get_available_core_ids(&self) -> Vec<usize> {
        (0..self.cpu_count).collect()
    }
    
    /// 为指定数量的物理核心分配对应的逻辑核心
    /// 
    /// 优先分配每个物理核心的主线程，避免超线程带来的性能干扰
    /// 
    /// # 参数
    /// * `physical_cores_needed` - 需要的物理核心数量
    /// 
    /// # 返回
    /// * `Vec<usize>` - 分配的逻辑核心ID列表
    pub fn allocate_logical_cores_for_physical(&self, physical_cores_needed: usize) -> Vec<usize> {
        let physical_cpu_count = self.get_physical_cpu_count();
        let logical_cpu_count = self.get_logical_cpu_count();
        
        if physical_cores_needed > physical_cpu_count {
            warn!("请求的物理核心数({})超过系统物理核心数({})", 
                  physical_cores_needed, physical_cpu_count);
            return (0..physical_cores_needed.min(logical_cpu_count)).collect();
        }
        
        // 计算是否支持超线程
        let hyperthreading_enabled = logical_cpu_count > physical_cpu_count;
        
        if hyperthreading_enabled {
            // 超线程CPU：优先分配每个物理核心的主线程
            // 通常映射：物理核心0→逻辑核心0, 物理核心1→逻辑核心1, ...
            info!("检测到超线程CPU，为{}个物理核心分配主线程逻辑核心", physical_cores_needed);
            (0..physical_cores_needed).collect()
        } else {
            // 非超线程CPU：物理核心和逻辑核心1:1对应
            info!("检测到非超线程CPU，直接分配{}个逻辑核心", physical_cores_needed);
            (0..physical_cores_needed).collect()
        }
    }

    /// 启动子进程并设置CPU核心亲和性
    /// 
    /// # 参数
    /// * `config` - 进程配置
    /// 
    /// # 返回
    /// * `Result<String>` - 成功时返回进程ID，失败时返回错误
    pub fn spawn_process_with_affinity(&self, config: ProcessConfig) -> Result<String> {
        let process_id = format!("{}-{}", config.name, chrono::Utc::now().timestamp_millis());
        
        // 确定要使用的核心列表（优先使用多核心配置）
        let cores_to_bind = if let Some(core_ids) = &config.core_ids {
            core_ids.clone()
        } else if let Some(core_id) = config.core_id {
            vec![core_id]
        } else {
            Vec::new()
        };
        
        // 验证核心ID是否有效
        for &core in &cores_to_bind {
            if core >= self.cpu_count {
                return Err(anyhow::anyhow!(
                    "无效的CPU核心ID: {}，系统只有 {} 个核心",
                    core,
                    self.cpu_count
                ));
            }
        }

        info!(
            "启动进程 '{}' (ID: {})，程序: {}，CPU核心: {:?}",
            config.name, process_id, config.program, cores_to_bind
        );

        // 构建命令
        let mut command = Command::new(&config.program);
        command.args(&config.args);
        
        // 设置工作目录
        if let Some(working_dir) = &config.working_dir {
            command.current_dir(working_dir);
        }
        
        // 设置环境变量
        if let Some(env_vars) = &config.env_vars {
            command.envs(env_vars);
        }
        
        // 设置标准输入输出
        command.stdout(Stdio::piped())
               .stderr(Stdio::piped())
               .stdin(Stdio::null());

        // 启动进程
        let mut child = command
            .spawn()
            .context(format!("启动进程失败: {}", config.program))?;

        let pid = child.id();
        info!("进程已启动，PID: {}", pid);

        // 设置CPU核心亲和性
        if !cores_to_bind.is_empty() {
            if let Err(e) = self.set_process_multi_core_affinity(pid, &cores_to_bind) {
                warn!("设置进程 {} CPU亲和性失败: {}", pid, e);
            } else {
                info!("成功将进程 {} 绑定到CPU核心 {:?}", pid, cores_to_bind);
            }
        }

        // 设置进程优先级
        if let Some(priority) = config.priority {
            if let Err(e) = self.set_process_priority(pid, priority) {
                warn!("设置进程 {} 优先级失败: {}", pid, e);
            } else {
                info!("成功设置进程 {} 优先级: {:?}", pid, priority);
            }
        }

        // 存储进程信息
        let command_str = format!("{} {}", config.program, config.args.join(" "));
        let process_info = ProcessInfo {
            child,
            core_id: cores_to_bind.get(0).copied(), // 第一个核心作为主核心（向后兼容）
            core_ids: cores_to_bind,
            name: config.name.clone(),
            command: command_str,
        };

        {
            let mut processes = self.active_processes.lock().unwrap();
            processes.insert(process_id.clone(), process_info);
        }

        Ok(process_id)
    }

    /// 设置进程CPU核心亲和性（Windows实现）
    #[cfg(target_os = "windows")]
    fn set_process_affinity(&self, pid: u32, core_id: usize) -> Result<()> {
        // 使用PowerShell设置CPU亲和性
        let affinity_mask = 1u64 << core_id;
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "$process = Get-Process -Id {}; $process.ProcessorAffinity = {}",
                    pid, affinity_mask
                ),
            ])
            .output()
            .context("执行PowerShell命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置CPU亲和性失败: {}", error_msg));
        }

        Ok(())
    }

    /// 设置进程CPU核心亲和性（Linux实现）
    #[cfg(target_os = "linux")]
    fn set_process_affinity(&self, pid: u32, core_id: usize) -> Result<()> {
        // 使用taskset命令设置CPU亲和性
        let output = Command::new("taskset")
            .args(["-cp", &core_id.to_string(), &pid.to_string()])
            .output()
            .context("执行taskset命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置CPU亲和性失败: {}", error_msg));
        }

        Ok(())
    }

    /// 设置进程CPU核心亲和性（macOS实现）
    #[cfg(target_os = "macos")]
    fn set_process_affinity(&self, _pid: u32, _core_id: usize) -> Result<()> {
        // macOS 不支持CPU亲和性设置
        warn!("macOS 不支持CPU亲和性设置");
        Ok(())
    }

    /// 设置进程多核心CPU亲和性（Windows实现）
    #[cfg(target_os = "windows")]
    fn set_process_multi_core_affinity(&self, pid: u32, core_ids: &[usize]) -> Result<()> {
        if core_ids.is_empty() {
            return Ok(());
        }
        
        // 计算多核心亲和性掩码：每个核心对应一个位
        let affinity_mask: u64 = core_ids.iter()
            .map(|&core_id| 1u64 << core_id)
            .fold(0u64, |acc, mask| acc | mask);
        
        info!("设置进程 {} 多核心亲和性掩码: 0x{:X} (核心: {:?})", 
              pid, affinity_mask, core_ids);

        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "$process = Get-Process -Id {}; $process.ProcessorAffinity = {}",
                    pid, affinity_mask
                ),
            ])
            .output()
            .context("执行PowerShell命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置多核心CPU亲和性失败: {}", error_msg));
        }

        Ok(())
    }

    /// 设置进程多核心CPU亲和性（Linux实现）
    #[cfg(target_os = "linux")]
    fn set_process_multi_core_affinity(&self, pid: u32, core_ids: &[usize]) -> Result<()> {
        if core_ids.is_empty() {
            return Ok(());
        }
        
        // 使用taskset命令设置多核心CPU亲和性
        let core_list = core_ids.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        
        info!("设置进程 {} 多核心亲和性: 核心列表 {}", pid, core_list);
        
        let output = Command::new("taskset")
            .args(["-cp", &core_list, &pid.to_string()])
            .output()
            .context("执行taskset命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置多核心CPU亲和性失败: {}", error_msg));
        }

        Ok(())
    }

    /// 设置进程多核心CPU亲和性（macOS实现）
    #[cfg(target_os = "macos")]
    fn set_process_multi_core_affinity(&self, _pid: u32, _core_ids: &[usize]) -> Result<()> {
        warn!("macOS 不支持CPU亲和性设置");
        Ok(())
    }

    /// 设置进程优先级（Windows实现）
    #[cfg(target_os = "windows")]
    fn set_process_priority(&self, pid: u32, priority: ProcessPriority) -> Result<()> {
        let priority_class = match priority {
            ProcessPriority::Low => "IDLE",
            ProcessPriority::Normal => "NORMAL",
            ProcessPriority::High => "HIGH",
            ProcessPriority::RealTime => "REALTIME",
        };

        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("processid={}", pid),
                "CALL",
                "setpriority",
                priority_class,
            ])
            .output()
            .context("执行wmic命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置进程优先级失败: {}", error_msg));
        }

        Ok(())
    }

    /// 设置进程优先级（Linux实现）
    #[cfg(target_os = "linux")]
    fn set_process_priority(&self, pid: u32, priority: ProcessPriority) -> Result<()> {
        let nice_value = match priority {
            ProcessPriority::Low => "10",
            ProcessPriority::Normal => "0",
            ProcessPriority::High => "-10",
            ProcessPriority::RealTime => "-20",
        };

        let output = Command::new("renice")
            .args([nice_value, "-p", &pid.to_string()])
            .output()
            .context("执行renice命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置进程优先级失败: {}", error_msg));
        }

        Ok(())
    }

    /// 设置进程优先级（macOS实现）
    #[cfg(target_os = "macos")]
    fn set_process_priority(&self, pid: u32, priority: ProcessPriority) -> Result<()> {
        let nice_value = match priority {
            ProcessPriority::Low => "10",
            ProcessPriority::Normal => "0",
            ProcessPriority::High => "-10",
            ProcessPriority::RealTime => "-20",
        };

        let output = Command::new("renice")
            .args([nice_value, "-p", &pid.to_string()])
            .output()
            .context("执行renice命令失败")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("设置进程优先级失败: {}", error_msg));
        }

        Ok(())
    }

    /// 终止指定进程
    pub fn terminate_process(&self, process_id: &str) -> Result<()> {
        let mut process_info = {
            let mut processes = self.active_processes.lock().unwrap();
            processes.remove(process_id)
        };

        match process_info {
            Some(ref mut info) => {
                info!("终止进程 '{}' (ID: {})", info.name, process_id);
                
                // 尝试优雅终止
                if let Err(e) = info.child.kill() {
                    warn!("终止进程失败: {}", e);
                }
                
                // 等待进程结束
                match info.child.wait() {
                    Ok(status) => {
                        info!("进程 '{}' 已终止，退出状态: {:?}", info.name, status);
                    }
                    Err(e) => {
                        warn!("等待进程终止失败: {}", e);
                    }
                }
                
                Ok(())
            }
            None => Err(anyhow::anyhow!("未找到进程ID: {}", process_id)),
        }
    }

    /// 获取进程输出
    pub fn get_process_output(&self, process_id: &str) -> Result<(String, String)> {
        let mut processes = self.active_processes.lock().unwrap();
        
        if let Some(info) = processes.get_mut(process_id) {
            let mut stdout_data = String::new();
            let mut stderr_data = String::new();
            
            // 读取标准输出
            if let Some(ref mut stdout) = info.child.stdout {
                use std::io::Read;
                stdout.read_to_string(&mut stdout_data)
                    .context("读取进程输出失败")?;
            }
            
            // 读取标准错误
            if let Some(ref mut stderr) = info.child.stderr {
                use std::io::Read;
                stderr.read_to_string(&mut stderr_data)
                    .context("读取进程错误输出失败")?;
            }
            
            Ok((stdout_data, stderr_data))
        } else {
            Err(anyhow::anyhow!("未找到进程ID: {}", process_id))
        }
    }

    /// 获取活跃进程数量
    pub fn get_active_process_count(&self) -> usize {
        let processes = self.active_processes.lock().unwrap();
        processes.len()
    }

    /// 获取活跃进程状态列表
    pub fn get_active_process_status(&self) -> Vec<ProcessStatus> {
        let mut processes = self.active_processes.lock().unwrap();
        let mut status_list = Vec::new();
        
        // 检查进程状态并清理已结束的进程
        let mut finished_ids = Vec::new();
        
        for (id, info) in processes.iter_mut() {
            match info.child.try_wait() {
                Ok(Some(_)) => {
                    // 进程已结束
                    finished_ids.push(id.clone());
                }
                Ok(None) => {
                    // 进程仍在运行
                    status_list.push(ProcessStatus {
                        id: id.clone(),
                        name: info.name.clone(),
                        pid: info.child.id(),
                        core_id: info.core_id,
                        command: info.command.clone(),
                        is_running: true,
                    });
                }
                Err(_) => {
                    // 检查状态失败，可能进程已结束
                    finished_ids.push(id.clone());
                }
            }
        }
        
        // 清理已结束的进程
        for id in finished_ids {
            processes.remove(&id);
        }
        
        status_list
    }

    /// 清理所有已完成的进程
    pub fn cleanup_finished_processes(&self) {
        let mut processes = self.active_processes.lock().unwrap();
        let mut finished_ids = Vec::new();

        for (id, info) in processes.iter_mut() {
            match info.child.try_wait() {
                Ok(Some(status)) => {
                    info!("清理已完成的进程 '{}' (ID: {})，退出状态: {:?}", info.name, id, status);
                    finished_ids.push(id.clone());
                }
                Ok(None) => {
                    // 进程仍在运行
                }
                Err(_) => {
                    // 检查失败，假设进程已结束
                    finished_ids.push(id.clone());
                }
            }
        }

        for id in finished_ids {
            processes.remove(&id);
        }
    }
}

/// 便捷函数：创建基本进程配置
pub fn create_process_config(name: &str, program: &str, args: Vec<String>) -> ProcessConfig {
    ProcessConfig {
        name: name.to_string(),
        program: program.to_string(),
        args,
        working_dir: None,
        env_vars: None,
        core_id: None,
        core_ids: None,
        priority: Some(ProcessPriority::Normal),
    }
}

/// 便捷函数：创建绑定CPU核心的进程配置（单核心）
pub fn create_process_config_with_core(
    name: &str,
    program: &str,
    args: Vec<String>,
    core_id: usize,
) -> ProcessConfig {
    ProcessConfig {
        name: name.to_string(),
        program: program.to_string(),
        args,
        working_dir: None,
        env_vars: None,
        core_id: Some(core_id),
        core_ids: None,
        priority: Some(ProcessPriority::Normal),
    }
}

/// 便捷函数：创建绑定多CPU核心的进程配置（逻辑核心）
/// 
/// # 参数
/// * `name` - 进程名称
/// * `program` - 程序路径
/// * `args` - 命令行参数
/// * `core_ids` - 逻辑CPU核心ID列表
/// ```
pub fn create_multi_core_process_config(
    name: &str,
    program: &str,
    args: Vec<String>,
    core_ids: Vec<usize>,
) -> ProcessConfig {
    ProcessConfig {
        name: name.to_string(),
        program: program.to_string(),
        args,
        working_dir: None,
        env_vars: None,
        core_id: None,
        core_ids: Some(core_ids),
        priority: Some(ProcessPriority::Normal),
    }
}

/// 便捷函数：基于物理核心数创建进程配置（推荐用于ORT推理）
/// 
/// 自动计算并分配对应的逻辑核心，优化ORT推理性能
/// 
/// # 参数
/// * `name` - 进程名称
/// * `program` - 程序路径
/// * `args` - 命令行参数
/// * `physical_cores_needed` - 需要的物理核心数量
/// 
/// # 示例
/// ```rust
/// ```
pub fn create_physical_core_process_config(
    name: &str,
    program: &str,
    args: Vec<String>,
    physical_cores_needed: usize,
) -> ProcessConfig {
    let process_manager = ProcessManager::new();
    let logical_cores = process_manager.allocate_logical_cores_for_physical(physical_cores_needed);
    
    // 添加环境变量，告诉子进程ORT应该使用多少线程
    let mut env_vars = HashMap::new();
    env_vars.insert("ORT_INTRA_THREADS".to_string(), physical_cores_needed.to_string());
    env_vars.insert("ORT_INTER_THREADS".to_string(), "1".to_string()); // 推理通常设为1
    env_vars.insert("PHYSICAL_CORES_ALLOCATED".to_string(), physical_cores_needed.to_string());
    env_vars.insert("LOGICAL_CORES_ALLOCATED".to_string(), 
                   logical_cores.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(","));
    
    info!("为进程'{}'分配{}个物理核心，映射到逻辑核心{:?}，ORT将使用{}个推理线程", 
          name, physical_cores_needed, logical_cores, physical_cores_needed);
    
    ProcessConfig {
        name: name.to_string(),
        program: program.to_string(),
        args,
        working_dir: None,
        env_vars: Some(env_vars),
        core_id: None,
        core_ids: Some(logical_cores),
        priority: Some(ProcessPriority::Normal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_process_manager_creation() {
        let cpu_count = num_cpus::get();
        info!("初始化进程管理器，检测到 {} 个CPU核心", cpu_count);
    }

    #[test]
    fn test_create_process_config() {
        let config = create_process_config(
            "test_process",
            "echo",
            vec!["Hello, World!".to_string()],
        );
        
        assert_eq!(config.name, "test_process");
        assert_eq!(config.program, "echo");
        assert_eq!(config.args[0], "Hello, World!");
        assert!(config.core_id.is_none());
    }

    #[test]
    fn test_create_process_config_with_core() {
        let config = create_process_config_with_core(
            "core_bound_process",
            "echo",
            vec!["test".to_string()],
            1,
        );

        assert_eq!(config.core_id, Some(1));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_spawn_simple_process() {
        let manager = ProcessManager::new();
        let config = create_process_config(
            "echo_test",
            "echo",
            vec!["test_output".to_string()],
        );
        
        let process_id = manager.spawn_process_with_affinity(config).unwrap();
        
        // 等待一段时间让进程完成
        std::thread::sleep(Duration::from_millis(100));
        
        // 检查进程状态
        let statuses = manager.get_active_process_status();
        println!("Active processes: {:?}", statuses);
        
        // 清理
        manager.cleanup_finished_processes();
    }
}