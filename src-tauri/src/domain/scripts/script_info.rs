use crate::infrastructure::core::{Deserialize, ScriptId, Serialize, UserId};
use crate::infrastructure::vision::ocr_factory::{DetectorType, RecognizerType};
use sqlx::types::Json;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTable {
    pub id : ScriptId,
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ScriptType {
    Local,
    Cloud,
    Custom,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInfo {
    pub name : String,
    pub description : Option<String>,
    pub user_id : UserId,
    pub user_name : Option<String>,

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
    pub is_valid: bool
}

impl Default for ScriptInfo {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            description: None,
            user_id: UserId::new_v7(),
            user_name: None,
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
            script_type: ScriptType::Local,
            is_valid: false,
        }
    }
 }