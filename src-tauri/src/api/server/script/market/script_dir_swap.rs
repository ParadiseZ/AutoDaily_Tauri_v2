use std::path::{Path, PathBuf};

pub(super) struct ScriptDirSwapState {
    pub(super) final_dir: PathBuf,
    staging_root: PathBuf,
    pub(super) staging_dir: PathBuf,
    backup_dir: PathBuf,
    backup_created: bool,
}

pub(super) fn prepare(
    scripts_root: &Path,
    script_id: &str,
    transfer_id: &str,
) -> Result<ScriptDirSwapState, String> {
    let staging_root = scripts_root.join(".download-staging").join(transfer_id);
    let backup_dir = scripts_root
        .join(".download-backup")
        .join(format!("{transfer_id}-{script_id}"));
    if staging_root.exists() {
        std::fs::remove_dir_all(&staging_root).map_err(|error| {
            format!("清理旧的暂存目录 {} 失败: {error}", staging_root.display())
        })?;
    }
    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)
            .map_err(|error| format!("清理旧的备份目录 {} 失败: {error}", backup_dir.display()))?;
    }
    Ok(ScriptDirSwapState {
        final_dir: scripts_root.join(script_id),
        staging_dir: staging_root.join(script_id),
        staging_root,
        backup_dir,
        backup_created: false,
    })
}

pub(super) fn activate(state: &mut ScriptDirSwapState) -> Result<(), String> {
    if let Some(parent) = state.backup_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("创建脚本模型备份目录 {} 失败: {error}", parent.display()))?;
    }
    if state.final_dir.exists() {
        std::fs::rename(&state.final_dir, &state.backup_dir).map_err(|error| {
            format!(
                "备份旧脚本模型目录 {} -> {} 失败: {error}",
                state.final_dir.display(),
                state.backup_dir.display()
            )
        })?;
        state.backup_created = true;
    }
    if let Err(error) = std::fs::rename(&state.staging_dir, &state.final_dir) {
        if state.backup_created {
            let _ = std::fs::rename(&state.backup_dir, &state.final_dir);
            state.backup_created = false;
        }
        return Err(format!(
            "启用新的脚本模型目录 {} -> {} 失败: {error}",
            state.staging_dir.display(),
            state.final_dir.display()
        ));
    }
    Ok(())
}

pub(super) fn rollback(state: &mut ScriptDirSwapState) -> Result<(), String> {
    if state.final_dir.exists() {
        std::fs::remove_dir_all(&state.final_dir).map_err(|error| {
            format!(
                "回滚时删除新脚本模型目录 {} 失败: {error}",
                state.final_dir.display()
            )
        })?;
    }
    if state.backup_created && state.backup_dir.exists() {
        std::fs::rename(&state.backup_dir, &state.final_dir).map_err(|error| {
            format!(
                "回滚旧脚本模型目录 {} -> {} 失败: {error}",
                state.backup_dir.display(),
                state.final_dir.display()
            )
        })?;
        state.backup_created = false;
    }
    Ok(())
}

pub(super) fn cleanup(state: &ScriptDirSwapState) {
    if state.staging_root.exists() {
        let _ = std::fs::remove_dir_all(&state.staging_root);
    }
    if state.backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&state.backup_dir);
    }
}
