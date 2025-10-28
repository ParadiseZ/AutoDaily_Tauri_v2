use std::path::Path;
use crate::infrastructure::scripts::script_error::ScriptResult;
use crate::infrastructure::scripts::script_info::ScriptInfo;
use crate::infrastructure::scripts::script_info_model::{ScriptMeta, ScriptPageReq};
use crate::infrastructure::core::ScriptId;

#[derive(Debug, Clone)]
pub struct ScriptMetaPage {
    pub items: Vec<ScriptMeta>,
    pub total_count: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

pub trait ScriptRepository: Send + Sync {
    fn reload_index(&mut self) -> ScriptResult<()>;
    fn page_metadata(&self, req: &ScriptPageReq) -> ScriptResult<ScriptMetaPage>;
    fn get_detail(&mut self, script_id: ScriptId) -> ScriptResult<ScriptInfo>;
    fn save(&mut self, script: &ScriptInfo) -> ScriptResult<()>;
    fn delete(&mut self, script_id: ScriptId) -> ScriptResult<()>;
}

pub trait CloudScriptRepository: Send + Sync {
    fn search(&self, req: &ScriptPageReq) -> ScriptResult<ScriptMetaPage>;
    fn get_detail(&self, script_id: ScriptId) -> ScriptResult<ScriptInfo>;
    fn download(&self, script_id: ScriptId, dest_root: &Path) -> ScriptResult<()>;
}


