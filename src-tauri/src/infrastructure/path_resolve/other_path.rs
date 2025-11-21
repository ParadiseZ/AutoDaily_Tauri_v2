use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::path_resolve::model_path::PathUtil;
use crate::infrastructure::path_resolve::path_error::{PathError, PathResult};
use std::fs;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::Manager;

impl PathUtil {
    pub fn sure_parent_exists(path_buf: &PathBuf) -> PathResult<()> {
        // 确保目录存在
        if let Some(parent) = path_buf.parent() {
            fs::create_dir_all(parent).map_err(|e| PathError::CreateDirErr {
                path: path_buf.to_string_lossy().to_string(),
                e: e.to_string(),
            })?
        }
        Ok(())
    }

    pub fn get_absolute_path(target_dir: &str, base: BaseDirectory) -> PathResult<PathBuf> {
        if target_dir.contains("//") || target_dir.contains("/") {
            let path_buf = PathBuf::from(target_dir);
            PathUtil::sure_parent_exists(&path_buf)?;
            return Ok(path_buf);
        }
        // 使用新 API 获取配置文件路径
        let path_buf = get_app_handle()
            .path()
            .resolve(target_dir, base)
            .map_err(|e| PathError::CreateDirErr {
                path: target_dir.into(),
                e: e.to_string(),
            })?;
        // 确保目录存在
        PathUtil::sure_parent_exists(&path_buf)?;
        Ok(path_buf)
    }
}
