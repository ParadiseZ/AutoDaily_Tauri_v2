use crate::domain::scripts::script_decision::Step;
use crate::domain::vision::ocr_search::{OcrSearcher, SearchRule, VisionSnapshot};
use crate::infrastructure::core::{Deserialize, PolicyGroupId, PolicyId, PolicySetId, ScriptId, Serialize};
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTable {
    pub id: PolicyId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub data: Json<PolicyInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInfo {
    pub name: String,
    pub note: String,
    pub log_print: Option<String>,

    pub cur_pos: u16,

    pub skip_flag : bool,
    pub exec_cur: u16,
    pub exec_max: u16,

    pub before_action: Vec<Step>,

    pub cond: SearchRule,

    pub after_action: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupTable {
    pub id: PolicyGroupId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub data: Json<PolicyGroupInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyGroupInfo {
    pub name: String,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetTable {
    pub id: PolicySetId,
    pub script_id: ScriptId,
    pub order_index: i32,
    pub data: Json<PolicySetInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetInfo {
    pub name: String,
    pub note: String,
}

// Structs for Many-to-Many fetching
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GroupPolicyRelation {
    pub group_id: PolicyGroupId,
    pub policy_id: PolicyId,
    pub order_index: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SetGroupRelation {
    pub set_id: PolicySetId,
    pub group_id: PolicyGroupId,
    pub order_index: i32,
}

/// 策略集评估器：用于高效地在一组策略中寻找命中项
pub struct PolicySetEvaluator<'a> {
    pub policies: Vec<&'a PolicyInfo>,
}

impl<'a> PolicySetEvaluator<'a> {
    pub fn new(policies: Vec<&'a PolicyInfo>) -> Self {
        Self { policies }
    }

    /// 执行单次批量搜索并返回所有命中的策略
    pub fn evaluate(&self, snapshot: &VisionSnapshot) -> Vec<&'a PolicyInfo> {
        if self.policies.is_empty() {
            return Vec::new();
        }

        // 1. 聚合所有策略的关键字
        let mut keywords = Vec::new();
        for policy in &self.policies {
            keywords.extend(policy.cond.get_all_keywords());
        }
        
        // 2. 去重并构建搜索器
        keywords.sort();
        keywords.dedup();
        
        if keywords.is_empty() {
            return Vec::new();
        }

        let searcher = OcrSearcher::new(keywords);
        
        // 3. 执行单次视觉搜索
        let hits = searcher.search(snapshot);

        // 4. 返回所有满足条件的策略
        self.policies.iter()
            .filter(|p| p.cond.evaluate(&hits))
            .copied()
            .collect()
    }
}
