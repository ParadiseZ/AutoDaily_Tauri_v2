/// 单个脚本任务本次执行的结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum TaskRunStatus {
    Success,
    Failed,
    Stopped,
    Skipped,
}
