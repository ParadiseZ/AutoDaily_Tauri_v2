use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri::path::BaseDirectory;
use crate::domain::app_handle::get_app_handle;
use crate::domain::entities::app_result::{AppError, AppResult};

pub async fn get_or_create_absolute_dirs(path : &str, base_dir : BaseDirectory) -> AppResult<()>{
    let dirs = if path.contains('/') || path.contains('\\') {
        PathBuf::from(path)
    } else {
        get_app_handle().await
            .path()
            .resolve(path, base_dir)
            .map_err(|e| AppError::SystemError(format!("解析目录失败: {}", e)))?
    };
    create_dirs(&dirs)
}

pub fn create_dirs(dirs : &PathBuf) -> AppResult<()>{
    if dirs.exists() {
        if dirs.is_file() {
            Err(AppError::SystemError(format!(
                "日志目录路径是一个文件: {}",
                dirs.display()
            )))
        }else { 
            Ok(())
        }
    } else {
        fs::create_dir_all(&dirs)
            .map_err(|e| AppError::IoError(format!("无法创建目录 {}: {}", dirs.display(), e)))?;
        Ok(())
    }
}