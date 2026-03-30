use crate::infrastructure::core::{Deserialize, ScriptId, Serialize, UserId};
use crate::infrastructure::vision::det::DetectorType;
use crate::infrastructure::vision::rec::RecognizerType;
use crate::domain::scripts::script_variable::ScriptVariableCatalog;
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTable {
    pub id : ScriptId,
    #[ts(as = "ScriptInfo")]
    pub data : Json<ScriptInfo>
}

impl Default for ScriptTable {
    fn default() -> Self {
        Self {
            id: ScriptId::new_v7(),
            data: Json(ScriptInfo::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ScriptType {
    /// 本地开发模式
    /// - 自定义模型路径: 用户指定的绝对路径
    /// - 内置模型路径: resources/models/ (程序资源目录)
    /// - 适用于: 开发者在本地调试脚本
    /// - cloud_id: 可能关联一个已上传的云端版本
    Dev,

    /// 已发布/云端模式
    /// - 自定义模型路径: scripts/{script_id}/models/ (相对路径)
    /// - 内置模型路径: resources/models/ (程序资源目录)
    /// - 适用于: 从云端下载的脚本
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeType {
    Rhai,
    JavaScript,
    Lua,
    AIAndVision,
    AI
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInfo {
    pub name : String,
    pub description : Option<String>,
    pub user_id : UserId,
    pub user_name : Option<String>,
    
    pub runtime_type: RuntimeType,

    pub sponsorship_qr : Option<String>,
    pub sponsorship_url : Option<String>,
    pub contact_info: Option<String>,

    pub img_det_model : Option<DetectorType>,
    pub txt_det_model : Option<DetectorType>,
    pub txt_rec_model : Option<RecognizerType>,
    pub pkg_name : Option<String>,

    pub create_time : Option<String>,
    pub update_time : Option<String>,
    pub ver_name : String,
    pub ver_num : u64,
    pub latest_ver : u64,
    pub download_count: u64,
    pub script_type: ScriptType,
    pub is_valid: bool,
    pub allow_clone: bool,
    /// 脚本变量目录，统一描述 input / runtime / system 三类变量定义。
    pub variable_catalog: ScriptVariableCatalog,
    
    /// 云端脚本 ID (仅 Dev 类型有此字段)
    /// - None: 从未上传过
    /// - Some(id): 已上传，关联的云端版本 ID
    pub cloud_id: Option<ScriptId>,

    // 模板排序时间 (秒)
    //pub template_time: Option<u64>,
}

impl Default for ScriptInfo {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            description: None,
            user_id: UserId::new_v7(),
            user_name: None,
            runtime_type: RuntimeType::Rhai,
            sponsorship_qr: None,
            sponsorship_url: None,
            contact_info: None,
            img_det_model: None,
            txt_det_model: None,
            txt_rec_model: None,
            pkg_name: None,
            create_time: None,
            update_time: None,
            ver_name: "".to_string(),
            ver_num: 0,
            latest_ver: 0,
            download_count: 0,
            script_type: ScriptType::Dev,
            is_valid: false,
            allow_clone: true,
            variable_catalog: ScriptVariableCatalog::default(),
            cloud_id: None,
            //template_time: None,
        }
    }
 }
