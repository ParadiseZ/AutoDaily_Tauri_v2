use crate::constant::table_name::SCRIPT_TABLE;
use crate::infrastructure::core::ScriptId;
use crate::infrastructure::db::DbRepo;
use tauri::command;
use crate::domain::scripts::script_info::ScriptTable;

/// 获取所有脚本配置
#[command]
pub async fn get_all_scripts_cmd() -> Result<Vec<ScriptTable>, String> {
    DbRepo::get_all::<ScriptTable>(SCRIPT_TABLE).await
}

/// 根据 ID 获取脚本配置
#[command]
pub async fn get_script_by_id_cmd(script_id: ScriptId) -> Result<Option<ScriptTable>, String> {
    DbRepo::get_by_id(SCRIPT_TABLE, &script_id.to_string()).await
}

/// 保存（新增或更新）脚本配置
#[command]
pub async fn save_script_cmd(script: ScriptTable) -> Result<(), String> {
    DbRepo::upsert_id_data(SCRIPT_TABLE, &script.id.to_string(), &script.data).await
}

/// 删除脚本配置
#[command]
pub async fn delete_script_cmd(script_id: ScriptId) -> Result<(), String> {
    DbRepo::delete(SCRIPT_TABLE, &script_id.to_string()).await
}
