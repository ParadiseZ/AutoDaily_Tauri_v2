// 性能监控模块
// 监控进程的CPU、内存、网络等资源使用情况

use crate::infrastructure::core::{Deserialize, ProcessId, Serialize};
use std::time::{SystemTime, Duration};
use std::collections::VecDeque;
use crate::infrastructure::core::HashMap;

/// 性能指标（详细版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // 进程标识
    pub process_id: ProcessId,
    pub timestamp: SystemTime,
    
    // CPU 使用情况
    pub cpu_usage_percent: f64,
    pub allocated_cores: Vec<usize>,
    pub core_utilization: HashMap<usize, f64>, // 每个核心的使用率
    pub cpu_time_user: Duration,
    pub cpu_time_system: Duration,
    
    // 内存使用情况
    pub memory_usage_mb: usize,
    pub memory_usage_percent: f64,
    pub peak_memory_mb: usize,
    pub shared_model_memory_mb: usize,
    pub memory_rss_mb: usize,      // 物理内存
    pub memory_vms_mb: usize,      // 虚拟内存
    
    // ORT 推理性能
    pub inference_metrics: InferenceMetrics,
    
    // 磁盘IO
    pub disk_read_mb: usize,
    pub disk_write_mb: usize,
    pub disk_read_ops: u64,
    pub disk_write_ops: u64,
    
    // 网络IO
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub network_packets_sent: u64,
    pub network_packets_received: u64,
    
    // 进程健康状态
    pub uptime: Duration,
    pub restart_count: u32,
    pub last_error: Option<String>,
    pub health_score: f64,        // 0-100的健康评分
    
    // 业务指标
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub pending_tasks: u64,
    pub average_task_duration: Duration,
    
    // 线程信息
    pub thread_count: u32,
    pub active_thread_count: u32,
}

/// 推理性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetrics {
    // 推理延迟统计（最近100次）
    pub detection_latency_ms: VecDeque<f64>,
    pub text_detection_latency_ms: VecDeque<f64>,
    pub text_recognition_latency_ms: VecDeque<f64>,
    pub overall_latency_ms: VecDeque<f64>,
    
    // 吞吐量
    pub inference_throughput: f64, // 每秒推理次数
    pub detection_throughput: f64,
    pub recognition_throughput: f64,
    
    // 成功率
    pub inference_success_rate: f64,
    pub detection_success_rate: f64,
    pub recognition_success_rate: f64,
    
    // 队列状态
    pub inference_queue_size: usize,
    pub max_queue_size: usize,
    pub queue_wait_time_ms: f64,
}

impl Default for InferenceMetrics {
    fn default() -> Self {
        Self {
            detection_latency_ms: VecDeque::with_capacity(100),
            text_detection_latency_ms: VecDeque::with_capacity(100),
            text_recognition_latency_ms: VecDeque::with_capacity(100),
            overall_latency_ms: VecDeque::with_capacity(100),
            inference_throughput: 0.0,
            detection_throughput: 0.0,
            recognition_throughput: 0.0,
            inference_success_rate: 100.0,
            detection_success_rate: 100.0,
            recognition_success_rate: 100.0,
            inference_queue_size: 0,
            max_queue_size: 0,
            queue_wait_time_ms: 0.0,
        }
    }
}

impl InferenceMetrics {
    /// 添加检测延迟样本
    pub fn add_detection_latency(&mut self, latency_ms: f64) {
        if self.detection_latency_ms.len() >= 100 {
            self.detection_latency_ms.pop_front();
        }
        self.detection_latency_ms.push_back(latency_ms);
    }
    
    /// 添加文字检测延迟样本
    pub fn add_text_detection_latency(&mut self, latency_ms: f64) {
        if self.text_detection_latency_ms.len() >= 100 {
            self.text_detection_latency_ms.pop_front();
        }
        self.text_detection_latency_ms.push_back(latency_ms);
    }
    
    /// 添加文字识别延迟样本
    pub fn add_text_recognition_latency(&mut self, latency_ms: f64) {
        if self.text_recognition_latency_ms.len() >= 100 {
            self.text_recognition_latency_ms.pop_front();
        }
        self.text_recognition_latency_ms.push_back(latency_ms);
    }
    
    /// 添加综合延迟样本
    pub fn add_overall_latency(&mut self, latency_ms: f64) {
        if self.overall_latency_ms.len() >= 100 {
            self.overall_latency_ms.pop_front();
        }
        self.overall_latency_ms.push_back(latency_ms);
    }
    
    /// 获取平均检测延迟
    pub fn get_average_detection_latency(&self) -> f64 {
        self.calculate_average(&self.detection_latency_ms)
    }
    
    /// 获取平均文字检测延迟
    pub fn get_average_text_detection_latency(&self) -> f64 {
        self.calculate_average(&self.text_detection_latency_ms)
    }
    
    /// 获取平均文字识别延迟
    pub fn get_average_text_recognition_latency(&self) -> f64 {
        self.calculate_average(&self.text_recognition_latency_ms)
    }
    
    /// 获取平均综合延迟
    pub fn get_average_overall_latency(&self) -> f64 {
        self.calculate_average(&self.overall_latency_ms)
    }
    
    fn calculate_average(&self, queue: &VecDeque<f64>) -> f64 {
        if queue.is_empty() {
            0.0
        } else {
            queue.iter().sum::<f64>() / queue.len() as f64
        }
    }
    
    /// 获取P95延迟
    pub fn get_p95_latency(&self, queue: &VecDeque<f64>) -> f64 {
        if queue.is_empty() {
            return 0.0;
        }
        
        let mut sorted: Vec<f64> = queue.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0.0)
    }
}

/// 进程监控器
pub struct ProcessMonitor {
    process_id: ProcessId,
    current_metrics: PerformanceMetrics,
    metrics_history: VecDeque<PerformanceMetrics>,
    max_history_size: usize,
    monitoring_interval: Duration,
    last_update: SystemTime,
}

impl ProcessMonitor {
    /// 创建新的进程监控器
    pub fn new(process_id: ProcessId) -> Self {
        Self {
            process_id: process_id.clone(),
            current_metrics: PerformanceMetrics {
                process_id,
                timestamp: SystemTime::now(),
                cpu_usage_percent: 0.0,
                allocated_cores: Vec::new(),
                core_utilization: HashMap::default(),
                cpu_time_user: Duration::new(0, 0),
                cpu_time_system: Duration::new(0, 0),
                memory_usage_mb: 0,
                memory_usage_percent: 0.0,
                peak_memory_mb: 0,
                shared_model_memory_mb: 0,
                memory_rss_mb: 0,
                memory_vms_mb: 0,
                inference_metrics: InferenceMetrics::default(),
                disk_read_mb: 0,
                disk_write_mb: 0,
                disk_read_ops: 0,
                disk_write_ops: 0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                network_packets_sent: 0,
                network_packets_received: 0,
                uptime: Duration::new(0, 0),
                restart_count: 0,
                last_error: None,
                health_score: 100.0,
                completed_tasks: 0,
                failed_tasks: 0,
                pending_tasks: 0,
                average_task_duration: Duration::new(0, 0),
                thread_count: 0,
                active_thread_count: 0,
            },
            metrics_history: VecDeque::new(),
            max_history_size: 1000, // 保留最近1000个样本
            monitoring_interval: Duration::from_secs(5),
            last_update: SystemTime::now(),
        }
    }
    
    /// 更新性能指标
    pub fn update_metrics(&mut self) -> Result<(), String> {
        let now = SystemTime::now();
        
        // 检查更新间隔
        if let Ok(elapsed) = self.last_update.elapsed() {
            if elapsed < self.monitoring_interval {
                return Ok(());
            }
        }
        
        // 更新系统级指标
        self.update_system_metrics()?;
        
        // 更新业务指标
        self.update_business_metrics();
        
        // 计算健康评分
        self.calculate_health_score();
        
        // 保存历史记录
        self.save_to_history();
        
        self.last_update = now;
        Ok(())
    }
    
    fn update_system_metrics(&mut self) -> Result<(), String> {
        // 这里应该调用系统API获取实际的性能数据
        // 目前提供模拟实现
        
        use sysinfo::System;
        
        let mut system = System::new_all();
        system.refresh_all();
        
        // 尝试找到对应的系统进程
        if let Some(process) = system.processes().values().find(|p| {
            p.name().contains("autodaily") || p.name().contains(&self.process_id)
        }) {
            // 更新CPU使用率
            self.current_metrics.cpu_usage_percent = process.cpu_usage() as f64;
            
            // 更新内存使用
            self.current_metrics.memory_usage_mb = (process.memory() / 1024 / 1024) as usize;
            self.current_metrics.memory_rss_mb = (process.memory() / 1024 / 1024) as usize;
            self.current_metrics.memory_vms_mb = (process.virtual_memory() / 1024 / 1024) as usize;
            
            // 更新峰值内存
            if self.current_metrics.memory_usage_mb > self.current_metrics.peak_memory_mb {
                self.current_metrics.peak_memory_mb = self.current_metrics.memory_usage_mb;
            }
            
            // 更新运行时间
            self.current_metrics.uptime = Duration::from_secs(process.run_time());
            
            // 计算内存使用百分比
            let total_memory = system.total_memory() / 1024 / 1024; // MB
            if total_memory > 0 {
                self.current_metrics.memory_usage_percent = 
                    (self.current_metrics.memory_usage_mb as f64 / total_memory as f64) * 100.0;
            }
        }
        
        self.current_metrics.timestamp = SystemTime::now();
        Ok(())
    }
    
    fn update_business_metrics(&mut self) {
        // 更新业务相关的指标
        // 这些通常由业务代码调用相应的方法来更新
        
        // 计算任务成功率
        let total_tasks = self.current_metrics.completed_tasks + self.current_metrics.failed_tasks;
        if total_tasks > 0 {
            let success_rate = (self.current_metrics.completed_tasks as f64 / total_tasks as f64) * 100.0;
            self.current_metrics.inference_metrics.inference_success_rate = success_rate;
        }
        
        // 计算平均任务持续时间
        if self.current_metrics.completed_tasks > 0 {
            // 这里应该维护一个任务持续时间的累计值
            // 目前使用模拟值
        }
    }
    
    fn calculate_health_score(&mut self) {
        let mut score = 100.0;
        
        // CPU使用率评分（权重30%）
        let cpu_score = if self.current_metrics.cpu_usage_percent > 90.0 {
            0.0
        } else if self.current_metrics.cpu_usage_percent > 80.0 {
            50.0
        } else {
            100.0
        };
        score = score * 0.7 + cpu_score * 0.3;
        
        // 内存使用率评分（权重30%）
        let memory_score = if self.current_metrics.memory_usage_percent > 90.0 {
            0.0
        } else if self.current_metrics.memory_usage_percent > 80.0 {
            50.0
        } else {
            100.0
        };
        score = score * 0.7 + memory_score * 0.3;
        
        // 任务成功率评分（权重40%）
        let task_score = self.current_metrics.inference_metrics.inference_success_rate;
        score = score * 0.6 + task_score * 0.4;
        
        self.current_metrics.health_score = score.max(0.0).min(100.0);
    }
    
    fn save_to_history(&mut self) {
        if self.metrics_history.len() >= self.max_history_size {
            self.metrics_history.pop_front();
        }
        self.metrics_history.push_back(self.current_metrics.clone());
    }
    
    /// 获取当前性能指标
    pub fn get_current_metrics(&self) -> &PerformanceMetrics {
        &self.current_metrics
    }
    
    /// 获取历史性能指标
    pub fn get_metrics_history(&self) -> &VecDeque<PerformanceMetrics> {
        &self.metrics_history
    }
    
    /// 记录推理延迟
    pub fn record_inference_latency(&mut self, inference_type: InferenceType, latency_ms: f64) {
        match inference_type {
            InferenceType::Detection => {
                let queue = &mut self.current_metrics.inference_metrics.detection_latency_ms;
                Self::add_latency_sample_static(queue, latency_ms);
            },
            InferenceType::TextDetection => {
                let queue = &mut self.current_metrics.inference_metrics.text_detection_latency_ms;
                Self::add_latency_sample_static(queue, latency_ms);
            },
            InferenceType::TextRecognition => {
                let queue = &mut self.current_metrics.inference_metrics.text_recognition_latency_ms;
                Self::add_latency_sample_static(queue, latency_ms);
            },
            InferenceType::Overall => {
                let queue = &mut self.current_metrics.inference_metrics.overall_latency_ms;
                Self::add_latency_sample_static(queue, latency_ms);
            },
        }
    }
    
    // 静态方法避免借用冲突
    fn add_latency_sample_static(queue: &mut VecDeque<f64>, latency_ms: f64) {
        if queue.len() >= 100 {
            queue.pop_front();
        }
        queue.push_back(latency_ms);
    }
    
    /// 记录任务完成
    pub fn record_task_completion(&mut self, duration: Duration, success: bool) {
        if success {
            self.current_metrics.completed_tasks += 1;
        } else {
            self.current_metrics.failed_tasks += 1;
        }
        
        // 更新平均任务持续时间
        let total_tasks = self.current_metrics.completed_tasks;
        if total_tasks > 0 {
            let current_avg = self.current_metrics.average_task_duration.as_millis() as f64;
            let new_duration = duration.as_millis() as f64;
            let new_avg = (current_avg * (total_tasks - 1) as f64 + new_duration) / total_tasks as f64;
            self.current_metrics.average_task_duration = Duration::from_millis(new_avg as u64);
        }
    }
    
    /// 设置分配的CPU核心
    pub fn set_allocated_cores(&mut self, cores: Vec<usize>) {
        self.current_metrics.allocated_cores = cores;
    }
    
    /// 更新核心利用率
    pub fn update_core_utilization(&mut self, core_utilization: HashMap<usize, f64>) {
        self.current_metrics.core_utilization = core_utilization;
    }
    
    /// 获取性能摘要
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            process_id: self.process_id.clone(),
            health_score: self.current_metrics.health_score,
            cpu_usage_percent: self.current_metrics.cpu_usage_percent,
            memory_usage_percent: self.current_metrics.memory_usage_percent,
            average_inference_latency: self.current_metrics.inference_metrics.get_average_overall_latency(),
            task_success_rate: self.current_metrics.inference_metrics.inference_success_rate,
            uptime: self.current_metrics.uptime,
            last_update: self.last_update,
        }
    }
}

/// 推理类型
#[derive(Debug, Clone, Copy)]
pub enum InferenceType {
    Detection,          // 目标检测
    TextDetection,      // 文字检测
    TextRecognition,    // 文字识别
    Overall,            // 综合推理
}

/// 性能摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub process_id: ProcessId,
    pub health_score: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub average_inference_latency: f64,
    pub task_success_rate: f64,
    pub uptime: Duration,
    pub last_update: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_monitor_creation() {
        let monitor = ProcessMonitor::new("test_process".to_string());
        assert_eq!(monitor.process_id, "test_process");
        assert_eq!(monitor.current_metrics.health_score, 100.0);
    }

    #[test]
    fn test_inference_metrics() {
        let mut metrics = InferenceMetrics::default();
        
        // 添加一些延迟样本
        metrics.add_detection_latency(10.0);
        metrics.add_detection_latency(15.0);
        metrics.add_detection_latency(12.0);
        
        let avg = metrics.get_average_detection_latency();
        assert!((avg - 12.33).abs() < 0.1);
    }

    #[test]
    fn test_health_score_calculation() {
        let mut monitor = ProcessMonitor::new("test".to_string());
        
        // 设置高CPU使用率
        monitor.current_metrics.cpu_usage_percent = 95.0;
        monitor.current_metrics.memory_usage_percent = 50.0;
        monitor.current_metrics.inference_metrics.inference_success_rate = 90.0;
        
        monitor.calculate_health_score();
        
        // 健康评分应该较低
        assert!(monitor.current_metrics.health_score < 100.0);
    }
}