use crate::infrastructure::core::{Deserialize, Serialize, TaskId};

#[derive(Debug, Clone, Serialize, Deserialize,Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTask{
    pub task_id : TaskId,
    pub task_name : String,
    pub exec_period : Period,
}

#[derive(Debug, Clone, Serialize, Deserialize,Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Period{
    EachTime,
    Everyday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}