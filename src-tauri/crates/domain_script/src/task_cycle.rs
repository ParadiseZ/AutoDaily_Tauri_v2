/// 脚本任务的执行周期。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TaskCycle {
    EveryRun,
    Daily,
    Weekly,
    WeekDay(u8),
    Monthly,
    MonthDay(u8),
}
