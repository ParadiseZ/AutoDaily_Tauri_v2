use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::path_resolve::path_error::PathError;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

/// 解析模型路径
/*async fn resolve_model_path(path_type: &ModelPathType) -> Result<String, PathError> {
    match path_type {
        ModelPathType::Resource(path) => {
            let absolute_path = get_app_handle().await.path()
                .resolve(path, BaseDirectory::Resource)
                .map_err(|e| PathError::ParsingFailed{ path: path.to_string(), e })?
                .to_string_lossy()
                .into_owned();
            Log::debug(&format!("默认模型路径转换：{}", absolute_path));
            Ok(absolute_path)
        },
        ModelPathType::Custom(path) => {
            let absolute_path = ScriptsConfig::get_dir() + "/ "+ path;
            Log::debug(&format!("自定义模型路径转换[{}]", absolute_path));
            Ok(absolute_path)
        },
        ModelPathType::Absolute(path)=>{
            Ok(path.into())
        }
    }
}*/
pub struct PathUtil;
impl PathUtil {
    pub fn resolve_path(
        app_handle: &AppHandle,
        model_type: &str,
        path: &str,
    ) -> Result<PathBuf, PathError> {
        match model_type {
            "build-in" => {
                let absolute_path = app_handle
                    .path()
                    .resolve(path, BaseDirectory::Resource)
                    .map_err(|e| PathError::ParsingFailed {
                        path: path.to_string(),
                        e: e.to_string(),
                    })?;
                //.to_string_lossy()
                //.into_owned();
                Log::debug(&format!(
                    "默认模型路径转换：{}",
                    absolute_path.to_string_lossy().to_string()
                ));
                Ok(absolute_path)
            }
            "custom" => {
                tokio::runtime::Handle::current()
                    .block_on(async {
                        let absolute_path = ScriptsConfig::get_dir().await.join("/").join(path) ;
                        Log::debug(&format!("自定义模型路径转换[{:?}]", absolute_path));
                        Ok(PathBuf::from(absolute_path))
                    })
            }
            _ => Ok(PathBuf::from(path)),
        }
    }
}
