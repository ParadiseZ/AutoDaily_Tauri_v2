use crate::{ScriptId, ScriptType};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScriptAccessError {
    #[error("云端下载脚本不可直接编辑，请先克隆为本地脚本")]
    PublishedScriptReadOnly,
    #[error("该脚本作者未开放克隆权限")]
    CloneNotAllowed,
}

pub fn ensure_editable(script_type: &ScriptType) -> Result<(), ScriptAccessError> {
    (script_type != &ScriptType::Published)
        .then_some(())
        .ok_or(ScriptAccessError::PublishedScriptReadOnly)
}

pub fn ensure_clone_allowed(allow_clone: bool, is_owner: bool) -> Result<(), ScriptAccessError> {
    (allow_clone || is_owner)
        .then_some(())
        .ok_or(ScriptAccessError::CloneNotAllowed)
}

pub fn clone_cloud_id(
    source_type: &ScriptType,
    source_id: ScriptId,
    source_cloud_id: Option<ScriptId>,
    overwrite_cloud_id: bool,
) -> Option<ScriptId> {
    match (source_type, overwrite_cloud_id) {
        (ScriptType::Published, true) => source_cloud_id.or(Some(source_id)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn protects_published_scripts_but_keeps_owner_cloning_available() {
        assert!(ensure_editable(&ScriptType::Dev).is_ok());
        assert_eq!(
            ensure_editable(&ScriptType::Published),
            Err(ScriptAccessError::PublishedScriptReadOnly)
        );
        assert!(ensure_clone_allowed(false, true).is_ok());
        assert_eq!(
            ensure_clone_allowed(false, false),
            Err(ScriptAccessError::CloneNotAllowed)
        );

        let source_id = ScriptId::from(Uuid::from_u128(1));
        assert_eq!(
            clone_cloud_id(&ScriptType::Published, source_id, None, true),
            Some(source_id)
        );
        assert_eq!(
            clone_cloud_id(&ScriptType::Dev, source_id, Some(source_id), true),
            None
        );
    }
}
