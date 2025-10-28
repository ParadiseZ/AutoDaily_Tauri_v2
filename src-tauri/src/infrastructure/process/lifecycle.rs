// 生命周期管理模块
// 管理进程的启动、停止、重启和清理

use crate::infrastructure::core::{Error, ProcessId};
use crate::infrastructure::process::handle::{ProcessHandle, ProcessState, ProcessError, ProcessResult};
use crate::infrastructure::process::config::{ProcessConfig, RestartPolicy};
use crate::infrastructure::process::monitor::ProcessMonitor;
use std::time::{SystemTime, Duration};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::sleep;

/// 生命周期管理错误
#[derive(Error, Debug)]
pub enum LifecycleError {
    #[error("启动超时: {process_id}")]
    StartupTimeout { process_id: ProcessId },
    
    #[error("停止超时: {process_id}")]
    ShutdownTimeout { process_id: ProcessId },
    
    #[error("进程配置无效: {reason}")]
    InvalidConfig { reason: String },
    
    #[error("资源清理失败: {reason}")]
    CleanupFailed { reason: String },
    
    #[error("状态转换无效: 从 {from} 到 {to}")]
    InvalidStateTransition { from: String, to: String },
}

/// 生命周期管理器
pub struct ProcessLifecycleManager {
    process_id: ProcessId,
    config: ProcessConfig,
    handle: Arc<Mutex<ProcessHandle>>,
    monitor: Arc<Mutex<ProcessMonitor>>,
    child_process: Option<Child>,
    shutdown_requested: bool,
}

impl ProcessLifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new(
        process_id: ProcessId,
        config: ProcessConfig,
        handle: ProcessHandle,
    ) -> Self {
        let monitor = ProcessMonitor::new(process_id.clone());
        
        Self {
            process_id: process_id.clone(),
            config,
            handle: Arc::new(Mutex::new(handle)),
            monitor: Arc::new(Mutex::new(monitor)),
            child_process: None,
            shutdown_requested: false,
        }
    }
    
    /// 启动进程
    pub async fn start(&mut self) -> ProcessResult<()> {
        // 验证配置
        self.config.validate().map_err(|e| ProcessError::StartupError {
            reason: format!("配置验证失败: {}", e),
        })?;
        
        // 检查当前状态
        {
            let handle = self.handle.lock().unwrap();
            if handle.state != ProcessState::Stopped && handle.state != ProcessState::Failed {
                return Err(ProcessError::StartupError {
                    reason: format!("进程当前状态不允许启动: {}", handle.state),
                });
            }
        }
        
        // 更新状态为启动中
        self.update_state(ProcessState::Starting)?;
        
        tracing::info!("开始启动进程: {}", self.process_id);
        
        // 准备启动命令
        let mut command = self.config.get_startup_command().map_err(|e| {
            ProcessError::StartupError { reason: e }
        })?;
        
        // 配置命令
        self.configure_command(&mut command)?;
        
        // 启动进程
        let child = command.spawn().map_err(|e| {
            ProcessError::StartupError {
                reason: format!("进程启动失败: {}", e),
            }
        })?;
        
        // 记录系统PID
        let system_pid = child.id();
        self.child_process = Some(child);
        
        // 更新句柄信息
        {
            let mut handle = self.handle.lock().unwrap();
            handle.system_pid = Some(system_pid);
            handle.started_at = SystemTime::now();
        }
        
        tracing::info!("进程启动成功: {} (PID: {})", self.process_id, system_pid);
        
        // 等待进程稳定运行
        if let Err(e) = self.wait_for_startup().await {
            self.update_state(ProcessState::Failed)?;
            return Err(e);
        }
        
        // 更新状态为运行中
        self.update_state(ProcessState::Running)?;
        
        // 开始监控
        self.start_monitoring().await;
        
        Ok(())
    }
    
    /// 停止进程
    pub async fn stop(&mut self, force: bool) -> ProcessResult<()> {
        tracing::info!("开始停止进程: {} (force: {})", self.process_id, force);
        
        // 标记停止请求
        self.shutdown_requested = true;
        
        // 更新状态为停止中
        self.update_state(ProcessState::Stopping)?;
        
        if let Some(mut child) = self.child_process.take() {
            if force {
                // 强制终止
                if let Err(e) = child.kill() {
                    tracing::error!("强制终止进程失败: {}", e);
                }
            } else {
                // 优雅关闭
                if let Err(e) = self.graceful_shutdown(&mut child).await {
                    tracing::warn!("优雅关闭失败，进行强制终止: {}", e);
                    let _ = child.kill();
                }
            }
            
            // 等待进程退出
            match tokio::time::timeout(
                self.config.shutdown_timeout,
                self.wait_for_exit(&mut child),
            ).await {
                Ok(Ok(exit_status)) => {
                    tracing::info!("进程已退出: {} (状态: {:?})", self.process_id, exit_status);
                },
                Ok(Err(e)) => {
                    tracing::error!("等待进程退出时出错: {}", e);
                },
                Err(_) => {
                    tracing::warn!("等待进程退出超时，强制终止");
                    let _ = child.kill();
                },
            }
        }
        
        // 清理资源
        self.cleanup().await?;
        
        // 更新状态为已停止
        self.update_state(ProcessState::Stopped)?;
        
        tracing::info!("进程停止完成: {}", self.process_id);
        Ok(())
    }
    
    /// 重启进程
    pub async fn restart(&mut self) -> ProcessResult<()> {
        tracing::info!("重启进程: {}", self.process_id);
        
        // 增加重启计数
        {
            let mut handle = self.handle.lock().unwrap();
            handle.increment_restart_count();
        }
        
        // 停止当前进程
        if !matches!(self.get_current_state()?, ProcessState::Stopped) {
            self.stop(false).await?;
        }
        
        // 等待重启延迟
        let restart_count = {
            let handle = self.handle.lock().unwrap();
            handle.restart_count
        };
        
        let delay = self.config.restart_policy.get_restart_delay(restart_count);
        if delay.as_secs() > 0 {
            tracing::info!("等待重启延迟: {:?}", delay);
            sleep(delay).await;
        }
        
        // 重新启动
        self.start().await
    }
    
    /// 暂停进程
    pub async fn pause(&mut self) -> ProcessResult<()> {
        tracing::info!("暂停进程: {}", self.process_id);
        
        // 检查当前状态
        if self.get_current_state()? != ProcessState::Running {
            return Err(ProcessError::StartupError {
                reason: "进程不在运行状态，无法暂停".to_string(),
            });
        }
        
        // 发送暂停信号（这里需要根据具体实现来处理）
        // 目前只更新状态
        self.update_state(ProcessState::Paused)?;
        
        Ok(())
    }
    
    /// 恢复进程
    pub async fn resume(&mut self) -> ProcessResult<()> {
        tracing::info!("恢复进程: {}", self.process_id);
        
        // 检查当前状态
        if self.get_current_state()? != ProcessState::Paused {
            return Err(ProcessError::StartupError {
                reason: "进程不在暂停状态，无法恢复".to_string(),
            });
        }
        
        // 发送恢复信号（这里需要根据具体实现来处理）
        // 目前只更新状态
        self.update_state(ProcessState::Running)?;
        
        Ok(())
    }
    
    /// 检查进程健康状态
    pub fn check_health(&self) -> ProcessResult<bool> {
        let handle = self.handle.lock().unwrap();
        Ok(handle.is_healthy())
    }
    
    /// 获取当前状态
    pub fn get_current_state(&self) -> ProcessResult<ProcessState> {
        let handle = self.handle.lock().unwrap();
        Ok(handle.state)
    }
    
    /// 是否应该重启
    pub fn should_restart(&self, exit_code: Option<i32>) -> bool {
        let handle = self.handle.lock().unwrap();
        self.config.restart_policy.should_restart(handle.restart_count, exit_code)
    }
    
    // 私有辅助方法
    
    fn configure_command(&self, command: &mut Command) -> ProcessResult<()> {
        // 设置工作目录
        if let Some(ref wd) = self.config.working_directory {
            command.current_dir(wd);
        }
        
        // 设置环境变量
        for (key, value) in &self.config.environment_vars {
            command.env(key, value);
        }
        
        // 配置标准输入输出
        if self.config.log_config.stdout_capture {
            command.stdout(Stdio::piped());
        }
        
        if self.config.log_config.stderr_capture {
            command.stderr(Stdio::piped());
        }
        
        // 设置进程组（用于信号处理）
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        
        Ok(())
    }
    
    async fn wait_for_startup(&self) -> ProcessResult<()> {
        let timeout = self.config.startup_timeout;
        let check_interval = Duration::from_millis(500);
        let start_time = SystemTime::now();
        
        loop {
            // 检查超时
            if start_time.elapsed().unwrap_or(Duration::ZERO) > timeout {
                return Err(ProcessError::StartupError {
                    reason: format!("启动超时: {:?}", timeout),
                });
            }
            
            // 检查进程是否还在运行
            if let Some(ref mut child) = &mut self.child_process.as_ref() {
                if let Ok(Some(exit_status)) = child.try_wait() {
                    return Err(ProcessError::StartupError {
                        reason: format!("进程启动后立即退出: {:?}", exit_status),
                    });
                }
            }
            
            // 检查健康状态（如果配置了健康检查）
            if self.config.health_check.enabled {
                if self.perform_health_check().await {
                    break; // 启动成功
                }
            } else {
                // 没有健康检查，等待一段时间后认为启动成功
                if start_time.elapsed().unwrap_or(Duration::ZERO) > Duration::from_secs(5) {
                    break;
                }
            }
            
            sleep(check_interval).await;
        }
        
        Ok(())
    }
    
    async fn graceful_shutdown(&self, child: &mut Child) -> ProcessResult<()> {
        // 发送SIGTERM信号（Unix）或者WM_CLOSE消息（Windows）
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            
            if let Some(pid) = child.id() {
                let pid = Pid::from_raw(pid as i32);
                signal::kill(pid, Signal::SIGTERM).map_err(|e| {
                    ProcessError::TerminationError {
                        reason: format!("发送SIGTERM失败: {}", e),
                    }
                })?;
            }
        }
        
        #[cfg(windows)]
        {
            // Windows下可以尝试发送Ctrl+C或者WM_CLOSE消息
            // 这里简化处理，直接调用terminate
            child.kill().map_err(|e| {
                ProcessError::TerminationError {
                    reason: format!("终止进程失败: {}", e),
                }
            })?;
        }
        
        Ok(())
    }
    
    async fn wait_for_exit(&self, child: &mut Child) -> ProcessResult<std::process::ExitStatus> {
        // 使用tokio的异步等待
        let exit_status = child.wait().map_err(|e| {
            ProcessError::TerminationError {
                reason: format!("等待进程退出失败: {}", e),
            }
        })?;
        
        Ok(exit_status)
    }
    
    async fn cleanup(&self) -> ProcessResult<()> {
        tracing::info!("清理进程资源: {}", self.process_id);
        
        // 清理临时文件
        // 释放端口
        // 清理内存映射
        // 等等...
        
        // 重置句柄状态
        {
            let mut handle = self.handle.lock().unwrap();
            handle.system_pid = None;
            handle.last_heartbeat = None;
        }
        
        Ok(())
    }
    
    fn update_state(&self, new_state: ProcessState) -> ProcessResult<()> {
        let mut handle = self.handle.lock().unwrap();
        
        // 验证状态转换的有效性
        if !self.is_valid_state_transition(handle.state, new_state) {
            return Err(ProcessError::StartupError {
                reason: format!("无效的状态转换: {} -> {}", handle.state, new_state),
            });
        }
        
        handle.update_state(new_state);
        Ok(())
    }
    
    fn is_valid_state_transition(&self, from: ProcessState, to: ProcessState) -> bool {
        use ProcessState::*;
        
        match (from, to) {
            // 从停止状态可以启动
            (Stopped, Starting) | (Failed, Starting) => true,
            
            // 从启动中可以到运行或失败
            (Starting, Running) | (Starting, Failed) => true,
            
            // 从运行中可以暂停、停止或失败
            (Running, Paused) | (Running, Stopping) | (Running, Failed) | (Running, Crashed) => true,
            
            // 从暂停可以恢复或停止
            (Paused, Running) | (Paused, Stopping) => true,
            
            // 从停止中到停止或失败
            (Stopping, Stopped) | (Stopping, Failed) => true,
            
            // 相同状态
            (state1, state2) if state1 == state2 => true,
            
            _ => false,
        }
    }
    
    async fn start_monitoring(&self) {
        // 启动性能监控
        // 这里应该启动一个后台任务来定期更新性能指标
        tracing::info!("开始监控进程: {}", self.process_id);
    }
    
    async fn perform_health_check(&self) -> bool {
        // 执行健康检查
        // 这里应该根据配置的健康检查类型来执行相应的检查
        match &self.config.health_check.check_type {
            crate::infrastructure::process::config::HealthCheckType::Heartbeat => {
                // 检查心跳
                let handle = self.handle.lock().unwrap();
                !handle.is_timeout(self.config.health_check.timeout)
            },
            crate::infrastructure::process::config::HealthCheckType::MemoryUsage => {
                // 检查内存使用
                true // 简化实现
            },
            crate::infrastructure::process::config::HealthCheckType::CpuUsage => {
                // 检查CPU使用
                true // 简化实现
            },
            crate::infrastructure::process::config::HealthCheckType::ResponseTime => {
                // 检查响应时间
                true // 简化实现
            },
            crate::infrastructure::process::config::HealthCheckType::Custom(_) => {
                // 自定义检查
                true // 简化实现
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::process::handle::{ProcessHandle, ProcessType, ProcessPriority};
    use uuid::Uuid;

    #[test]
    fn test_state_transitions() {
        let config = ProcessConfig::for_device(
            "test".to_string(),
            Uuid::new_v4(),
            2,
        );
        
        let handle = ProcessHandle::new(
            "test".to_string(),
            ProcessType::Device { device_id: Uuid::new_v4() },
            ProcessPriority::Normal,
        );
        
        let manager = ProcessLifecycleManager::new(
            "test".to_string(),
            config,
            handle,
        );
        
        // 测试有效的状态转换
        assert!(manager.is_valid_state_transition(ProcessState::Stopped, ProcessState::Starting));
        assert!(manager.is_valid_state_transition(ProcessState::Starting, ProcessState::Running));
        assert!(manager.is_valid_state_transition(ProcessState::Running, ProcessState::Stopping));
        
        // 测试无效的状态转换
        assert!(!manager.is_valid_state_transition(ProcessState::Running, ProcessState::Starting));
        assert!(!manager.is_valid_state_transition(ProcessState::Stopped, ProcessState::Running));
    }
}