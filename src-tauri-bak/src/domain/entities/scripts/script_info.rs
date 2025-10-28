use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Local, NaiveTime};
use crate::domain::entities::app_result::AppError;

/// 脚本ID类型定义
pub type ScriptId = String;

/// 脚本状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptStatus {
    /// 停止状态
    Stopped,
    /// 运行中
    Running,
    /// 暂停状态
    Paused,
    /// 错误状态
    Error(String),
    /// 等待调度
    Scheduled,
    /// 准备启动
    Starting,
    /// 正在停止
    Stopping,
}

/// 脚本执行优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScriptPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// 脚本执行时间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptScheduleConfig {
    /// 是否启用调度
    pub enabled: bool,
    /// 开始时间
    pub start_time: Option<NaiveTime>,
    /// 结束时间  
    pub end_time: Option<NaiveTime>,
    /// 执行间隔（秒）
    pub interval_seconds: Option<u64>,
    /// 最大执行次数（None表示无限制）
    pub max_executions: Option<u32>,
    /// 当前已执行次数
    pub current_executions: u32,
    /// 是否在系统空闲时执行
    pub run_on_idle: bool,
    /// 最大运行时长（秒）
    pub max_duration_seconds: Option<u64>,
}

impl Default for ScriptScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start_time: None,
            end_time: None,
            interval_seconds: None,
            max_executions: None,
            current_executions: 0,
            run_on_idle: false,
            max_duration_seconds: None,
        }
    }
}

/// 脚本配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// 脚本依赖的模型列表
    pub required_models: Vec<String>,
    /// 脚本参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 超时设置（秒）
    pub timeout_seconds: u64,
    /// 失败重试次数
    pub retry_count: u32,
    /// 资源需求（预估）
    pub resource_requirements: ScriptResourceRequirements,
}

/// 脚本资源需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResourceRequirements {
    /// 预估内存需求（MB）
    pub memory_mb: u64,
    /// 预估CPU使用率（0-100）
    pub cpu_usage_percent: u8,
    /// 是否需要GPU
    pub requires_gpu: bool,
    /// 预估执行时间（秒）
    pub estimated_duration_seconds: u64,
}

impl Default for ScriptResourceRequirements {
    fn default() -> Self {
        Self {
            memory_mb: 100,
            cpu_usage_percent: 10,
            requires_gpu: false,
            estimated_duration_seconds: 30,
        }
    }
}

/// 脚本执行统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionStats {
    /// 总执行次数
    pub total_executions: u64,
    /// 成功次数
    pub successful_executions: u64,
    /// 失败次数
    pub failed_executions: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: u64,
    /// 最后执行时间
    pub last_execution_time: Option<DateTime<Local>>,
    /// 最后执行结果
    pub last_execution_result: Option<ScriptExecutionResult>,
}

impl Default for ScriptExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0,
            last_execution_time: None,
            last_execution_result: None,
        }
    }
}

/// 脚本执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptExecutionResult {
    Success,
    Failed(String),
    Timeout,
    Cancelled,
}

/// 脚本结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    /// 脚本唯一标识
    pub id: ScriptId,
    /// 脚本名称
    pub name: String,
    /// 脚本描述
    pub description: String,
    /// 脚本版本
    pub version: String,
    /// 脚本作者
    pub author: String,
    /// 脚本文件路径
    pub script_path: String,
    /// 当前状态
    pub status: ScriptStatus,
    /// 执行优先级
    pub priority: ScriptPriority,
    /// 调度配置
    pub schedule_config: ScriptScheduleConfig,
    /// 脚本配置
    pub config: ScriptConfig,
    /// 执行统计
    pub execution_stats: ScriptExecutionStats,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
    /// 是否启用
    pub enabled: bool,
    /// 标签（用于分类和筛选）
    pub tags: Vec<String>,
}

impl ScriptInfo {
    /// 创建新脚本
    pub fn new(
        id: ScriptId,
        name: String,
        description: String,
        script_path: String,
    ) -> Self {
        let now = Local::now();
        Self {
            id,
            name,
            description,
            version: "1.0.0".to_string(),
            author: "Unknown".to_string(),
            script_path,
            status: ScriptStatus::Stopped,
            priority: ScriptPriority::Normal,
            schedule_config: ScriptScheduleConfig::default(),
            config: ScriptConfig {
                required_models: Vec::new(),
                parameters: HashMap::new(),
                timeout_seconds: 300, // 默认5分钟超时
                retry_count: 3,
                resource_requirements: ScriptResourceRequirements::default(),
            },
            execution_stats: ScriptExecutionStats::default(),
            created_at: now,
            updated_at: now,
            enabled: true,
            tags: Vec::new(),
        }
    }

    /// 更新脚本状态
    pub fn update_status(&mut self, status: ScriptStatus) {
        self.status = status;
        self.updated_at = Local::now();
    }

    /// 检查脚本是否可以执行
    pub fn can_execute(&self) -> Result<(), AppError> {
        if !self.enabled {
            return Err(AppError::ConfigError("脚本已禁用".to_string()));
        }

        match self.status {
            ScriptStatus::Running => Err(AppError::ConfigError("脚本正在运行".to_string())),
            ScriptStatus::Starting => Err(AppError::ConfigError("脚本正在启动".to_string())),
            ScriptStatus::Stopping => Err(AppError::ConfigError("脚本正在停止".to_string())),
            _ => Ok(()),
        }
    }

    /// 检查脚本是否在指定时间段内应该运行
    pub fn should_run_at_time(&self, time: NaiveTime) -> bool {
        if !self.schedule_config.enabled {
            return false;
        }

        // 检查时间范围
        if let (Some(start), Some(end)) = (self.schedule_config.start_time, self.schedule_config.end_time) {
            if start <= end {
                // 同一天的时间范围
                return time >= start && time <= end;
            } else {
                // 跨天的时间范围
                return time >= start || time <= end;
            }
        }

        true
    }

    /// 检查是否达到最大执行次数
    pub fn has_reached_max_executions(&self) -> bool {
        if let Some(max) = self.schedule_config.max_executions {
            return self.schedule_config.current_executions >= max;
        }
        false
    }

    /// 记录执行结果
    pub fn record_execution(&mut self, result: ScriptExecutionResult, execution_time_ms: u64) {
        self.execution_stats.total_executions += 1;
        self.execution_stats.last_execution_time = Some(Local::now());
        self.execution_stats.last_execution_result = Some(result.clone());

        match result {
            ScriptExecutionResult::Success => {
                self.execution_stats.successful_executions += 1;
                if self.schedule_config.enabled {
                    self.schedule_config.current_executions += 1;
                }
            }
            ScriptExecutionResult::Failed(_) => {
                self.execution_stats.failed_executions += 1;
            }
            _ => {}
        }

        // 更新平均执行时间
        let total_time = self.execution_stats.average_execution_time_ms
            * (self.execution_stats.total_executions - 1)
            + execution_time_ms;
        self.execution_stats.average_execution_time_ms = total_time / self.execution_stats.total_executions;

        self.updated_at = Local::now();
    }

    /// 重置执行统计
    pub fn reset_execution_stats(&mut self) {
        self.execution_stats = ScriptExecutionStats::default();
        self.schedule_config.current_executions = 0;
        self.updated_at = Local::now();
    }

    /// 获取成功率
    pub fn get_success_rate(&self) -> f64 {
        if self.execution_stats.total_executions == 0 {
            return 0.0;
        }

        self.execution_stats.successful_executions as f64 / self.execution_stats.total_executions as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_creation() {
        let script = ScriptInfo::new(
            "test_script".to_string(),
            "Test Script".to_string(),
            "A test script".to_string(),
            "/path/to/script.json".to_string(),
        );

        assert_eq!(script.id, "test_script");
        assert_eq!(script.name, "Test Script");
        assert_eq!(script.status, ScriptStatus::Stopped);
        assert!(script.enabled);
    }

    #[test]
    fn test_script_time_check() {
        let mut script = ScriptInfo::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            "/test".to_string(),
        );

        // 设置调度时间 9:00-17:00
        script.schedule_config.enabled = true;
        script.schedule_config.start_time = Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        script.schedule_config.end_time = Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap());

        // 测试时间范围内
        assert!(script.should_run_at_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()));

        // 测试时间范围外
        assert!(!script.should_run_at_time(NaiveTime::from_hms_opt(20, 0, 0).unwrap()));
    }

    #[test]
    fn test_execution_recording() {
        let mut script = ScriptInfo::new(
            "test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            "/test".to_string(),
        );

        script.record_execution(ScriptExecutionResult::Success, 1000);

        assert_eq!(script.execution_stats.total_executions, 1);
        assert_eq!(script.execution_stats.successful_executions, 1);
        assert_eq!(script.execution_stats.average_execution_time_ms, 1000);
        assert_eq!(script.get_success_rate(), 100.0);
    }
}
