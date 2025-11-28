use crate::domain::scripts::script_decision::{
    DecisionError, DecisionResult, GuardDef, GuardRepository, PolicyDef, PolicyRepository,
    SubFlowDef, SubFlowRepository,
};
use crate::infrastructure::core::{Deserialize, ScriptId};
use std::path::PathBuf;

// 简单 JSON 文件仓储实现（公共库 + 脚本内策略），便于后续替换为更复杂的来源

pub struct JsonDecisionRepository {
    pub base_dir: PathBuf, // 根目录：defaults/common 或 scripts/{id}/decision
}

impl JsonDecisionRepository {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn load_json<T: for<'de> Deserialize<'de>>(&self, file: &str) -> DecisionResult<Vec<T>> {
        let path = self.base_dir.join(file);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let list: Vec<T> = serde_json::from_str(&text)
                    .map_err(|e| DecisionError::Repository(e.to_string()))?;
                Ok(list)
            }
            Err(_) => Ok(vec![]), // 不存在则返回空，简化首次集成
        }
    }
}

impl GuardRepository for JsonDecisionRepository {
    fn load_common_guards(&self) -> DecisionResult<Vec<GuardDef>> {
        self.load_json("guards.common.json")
    }
    fn load_script_guards(&self, script_id: ScriptId) -> DecisionResult<Vec<GuardDef>> {
        let sub = JsonDecisionRepository::new(self.base_dir.join(script_id.to_string().as_str()));
        sub.load_json("guards.json")
    }
}

impl PolicyRepository for JsonDecisionRepository {
    fn load_common_policies(&self) -> DecisionResult<Vec<PolicyDef>> {
        self.load_json("policies.common.json")
    }
    fn load_script_policies(&self, script_id: ScriptId) -> DecisionResult<Vec<PolicyDef>> {
        let sub = JsonDecisionRepository::new(self.base_dir.join(script_id.to_string().as_str()));
        sub.load_json("policies.json")
    }
}

impl SubFlowRepository for JsonDecisionRepository {
    fn load_common_subflows(&self) -> DecisionResult<Vec<SubFlowDef>> {
        self.load_json("subflows.common.json")
    }
    fn load_script_subflows(&self, script_id: ScriptId) -> DecisionResult<Vec<SubFlowDef>> {
        let sub = JsonDecisionRepository::new(self.base_dir.join(script_id.to_string().as_str()));
        sub.load_json("subflows.json")
    }
}
