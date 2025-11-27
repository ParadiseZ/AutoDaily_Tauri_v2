use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::config::conf_error::{
    CastErr, ConfigResult, DeserializeErr, LoadErr, NotInitErr, SerializeErr, WriteErr,
};
use crate::infrastructure::config::conf_write_guard::{ConfigCategory, ConfigWriteGuard};
use crate::infrastructure::core::HashMap;
use crate::infrastructure::path_resolve::model_path::PathUtil;
use std::any::Any;
use std::fs;
use std::sync::{Arc};
use serde::Deserialize;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ConfigManager {
    // 临时实现，保持向后兼容
    pub(crate) caches:
        Arc<RwLock<HashMap<String, (Arc<dyn Any + Send + Sync>, std::path::PathBuf)>>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn init_category<C: ConfigCategory + 'static + Deserialize + Send>(
        &self,
        category: &str,
        base_dir: BaseDirectory,
    ) -> ConfigResult<()> {
        // 使用新 API 获取配置文件路径
        let path = get_app_handle()
            .path()
            .resolve(format!("{}", category), base_dir)
            .map_err(|e| LoadErr { path: category.to_string(), e: e.to_string() })?;

        // 确保目录存在
        PathUtil::sure_parent_exists(&path).map_err(|e| LoadErr { path: category.to_string(), e: e.to_string() })?;

        // 尝试从文件加载
        let config = if path.exists() {
            let json = fs::read_to_string(&path).map_err(|e| LoadErr {
                path: path.to_string_lossy().to_string(),
                e: e.to_string(),
            })?;
            serde_json::from_str(&json).map_err(|e| DeserializeErr {
                path: path.to_string_lossy().to_string(),
                e: e.to_string(),
            })?
        } else {
            // 创建默认配置
            C::default()
        };

        // 保存到内存缓存
        {
            let mut caches = self.caches.write().await;
            caches.insert(category.to_string(), (Arc::new(config), path));
        } // 提前释放写锁，避免后续保存时读写锁相互等待

        // 立即持久化（确保文件存在）
        self.save_category::<C>(category).await?;
        Ok(())
    }

    pub(crate) async fn get_conf<C: ConfigCategory + 'static + Clone>(
        &self,
        category: &str,
    ) -> ConfigResult<C> {
        let caches = self.caches.read().await;
        let (data, _) = match caches.get(category) {
            Some(v) => v,
            None => return Err(NotInitErr {
                conf_category: category.into(),
                e: "NotInit".into(),
            }),
        };

        // 尝试转换为具体类型
        let config = data.downcast_ref::<C>().ok_or_else(|e| CastErr { e })?;

        Ok(config.clone())
    }

    pub async fn get_conf_mut<C: ConfigCategory + 'static>(
        &self,
        category: &str,
    ) -> ConfigResult<ConfigWriteGuard<'_, C>> {
        let caches = self.caches.write().await;
        let (data, path) = match caches.get(category) {
            Some(v) => v,
            None => return Err(NotInitErr {
                conf_category: category.into(),
                e: "NotInit".into(),
            }),
        };

        // 先获取当前配置的拷贝作为初始值
        let current_config = {
            let data_ref = data.downcast_ref::<C>().ok_or_else(|e| CastErr { e })?;
            data_ref.clone()
        };
        //.map_err(|_| anyhow::anyhow!("Config type mismatch"))?;

        Ok(ConfigWriteGuard {
            config: current_config,
            path,
            manager: self,
            _caches_lock: caches, // 保持锁直到 ConfigWriteGuard 被丢弃
        })
    }

    /// 保存配置到文件
    async fn save_category<C: ConfigCategory + 'static>(&self, category: &str) -> ConfigResult<()> {
        let caches = self.caches.read().await;
        let (data, path) = match caches.get(category) {
            Some(v) => v,
            None => return Err(NotInitErr {
                conf_category: category.into(),
                e: "NotInit".into(),
            }),
        };
        // 从内存中的配置生成JSON
        let config_ref = data.downcast_ref::<C>().ok_or_else(|e| CastErr { e })?;
        let json = serde_json::to_string_pretty(config_ref).map_err(|e| SerializeErr {
            path: path.clone(),
            e: e.to_string(),
        })?;
        //.context("Failed to serialize config")?;

        tokio::fs::write(path, json).await.map_err(|e| WriteErr {
            path: path.clone(),
            e: e.to_string(),
        })?;
        //.context("Failed to write config file")?;

        Ok(())
    }
}
