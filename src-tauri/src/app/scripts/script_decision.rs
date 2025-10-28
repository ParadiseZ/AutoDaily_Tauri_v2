use crate::domain::scripts::script_decision::{GuardDef, PolicyDef, SubFlowDef};
use crate::infrastructure::core::{HashMap, SubFlowId};

pub struct ReactiveExecutor {
    pub guards: Vec<GuardDef>,
    pub policies: Vec<PolicyDef>,
    pub subflows: HashMap<SubFlowId, SubFlowDef>,
}

impl ReactiveExecutor {
    pub fn new(guards: Vec<GuardDef>, policies: Vec<PolicyDef>, subflows: Vec<SubFlowDef>) -> Self {
        let mut map = HashMap::default();
        for sf in subflows {
            map.insert(sf.id.clone(), sf);
        }
        Self { guards, policies, subflows: map }
    }

    pub fn tick(&mut self) {
        // TODO: 感知→守卫→策略→执行
    }
}


