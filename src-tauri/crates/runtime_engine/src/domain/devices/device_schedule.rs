// 设备脚本分配（队列定义）+ 调度记录

use crate::infrastructure::core::{
    Deserialize, DeviceId, ExecutionId, ScheduleId, ScriptId, Serialize, TaskId, TemplateId,
};
use sqlx::FromRow;

/// 任务周期
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TaskCycle {
    /// 每次运行
    EveryRun,
    /// 每天
    Daily,
    /// 每周（完成后 +7 天）
    Weekly,
    /// 每周指定周几 (1=周一..7=周日)
    WeekDay(u8),
    /// 每月
    Monthly,
    /// 每月指定日期
    MonthDay(u8),
}

/// 运行状态
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RunStatus {
    Success,
    Failed,
    Skipped,
}

/// 队列定义：用户在 TaskManagement 中给设备安排的脚本计划
/// 一行 = 一个脚本分配
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceScriptAssignment {
    pub id: ScheduleId,
    pub device_id: DeviceId,
    pub script_id: ScriptId,
    pub time_template_id: Option<TemplateId>,
    /// 预留：账号/配置数据
    #[ts(type = "any")]
    pub account_data: sqlx::types::Json<serde_json::Value>,
    /// 排序顺序
    pub index: u32,
}

impl Default for DeviceScriptAssignment {
    fn default() -> Self {
        Self {
            id: ScheduleId::new_v7(),
            device_id: DeviceId::new_v7(),
            script_id: ScriptId::new_v7(),
            time_template_id: None,
            account_data: sqlx::types::Json(serde_json::Value::Null),
            index: 0,
        }
    }
}

/// 调度记录（append-only）：每次调度完成一个任务就 INSERT 一条
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeviceScriptSchedule {
    pub id: ScheduleId,
    pub device_id: DeviceId,
    pub execution_id: Option<ExecutionId>,
    pub assignment_id: Option<ScheduleId>,
    pub script_id: ScriptId,
    pub task_id: TaskId,
    /// 从 ScriptTaskTable 继承的周期
    #[ts(type = "string")]
    pub task_cycle: String,
    /// 运行状态
    #[ts(type = "string")]
    pub status: String,
    /// 开始时间
    #[ts(type = "string")]
    pub started_at: String,
    /// 完成时间
    #[ts(type = "string | null")]
    pub completed_at: Option<String>,
    /// 错误信息等
    pub message: Option<String>,
}
