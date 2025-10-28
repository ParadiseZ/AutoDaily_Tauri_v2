use std::fs;
use std::sync::Arc;
use tauri::Manager;
use tauri::path::BaseDirectory;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager, ConfigWriteGuard};
use crate::domain::trait_ad::config_category::ConfigCategory;

impl ConfMgr for ConfigManager{
    async fn init_category<C: ConfigCategory + 'static>(&self, category: &str, dir : Option<String>,
                                                  base_dir : BaseDirectory ) -> AppResult<()> {
        // 使用新 API 获取配置文件路径
        let path = self.app_handle
            .path()
            .resolve(format!("{}", category), base_dir)?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::ConfigError(format!("目录{:?}创建失败:{}", path, e)))?;
        }

        // 尝试从文件加载
        let config = if path.exists() {
            let json = fs::read_to_string(&path);
            match json {
                Ok(json_content) => {
                    match serde_json::from_str(&json_content) {
                        Ok(config) => config,
                        Err(serde_err) => {
                            let err = format!("序列化{}的{}配置失败:{},请检查配置", path.display(), category, serde_err);
                            Log::error(&err);
                            return Err(AppError::ConfigError(err));
                        }
                    }
                }
                Err(read_err) => {
                    Log::error(&format!("从{}读取{}配置失败:{},将使用默认配置", path.display(), category, read_err));
                    return Err(AppError::ConfigError(read_err.to_string()));
                }
            }
        } else {
            // 创建默认配置
            C::default()
        };

        // 保存到内存缓存
        {
            let mut caches = self.caches.write().await;
            caches.insert(
                category.to_string(),
                (Arc::new(config), path)
            );
        } // 提前释放写锁，避免后续保存时读写锁相互等待

        // 立即持久化（确保文件存在）
        self.save_category::<C>(category).await?;
        Ok(())
    }

    async fn get_conf<C: ConfigCategory + 'static + Clone>(&self, category: &str) -> AppResult<C> {
        let caches = self.caches.read().await;
        let (data, _) = caches.get(category)
            .ok_or_else(|| AppError::ConfigError(format!("配置{}未初始化", category)))?;

        // 尝试转换为具体类型
        let config = data.downcast_ref::<C>()
            .ok_or_else(|| AppError::ConfigError(format!("配置{}类型不匹配", category)))?;

        Ok(config.clone())
    }

    async fn get_conf_mut<C: ConfigCategory + 'static>(&self, category: &str) -> AppResult<ConfigWriteGuard<'_, C>> {
        let caches = self.caches.write().await;
        let (data, path) = caches.get(category)
            .ok_or_else(|| AppError::ConfigError(format!("配置{}未初始化", category)))?
            .clone();

        // 先获取当前配置的拷贝作为初始值
        let current_config = {
            let data_ref = data.downcast_ref::<C>()
                .ok_or_else(|| AppError::ConfigError(format!("配置{}类型不匹配", category)))?;
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
    async fn save_category<C: ConfigCategory + 'static>(
        &self,
        category: &str
    ) -> AppResult<()> {
        let caches = self.caches.read().await;
        let (data, path) = caches.get(category)
            .ok_or_else(|| AppError::ConfigError(format!("配置{}未初始化", category)))?;

        // 从内存中的配置生成JSON
        let config_ref = data.downcast_ref::<C>()
            .ok_or_else(|| AppError::ConfigError(format!("配置{}类型不匹配", category)))?;

        let json = serde_json::to_string_pretty(config_ref)
            .map_err(|e| AppError::ConfigError(format!("配置{}序列化失败:{}", category, e)))?;
        //.context("Failed to serialize config")?;

        tokio::fs::write(path, json)
            .await
            .map_err(|e| AppError::ConfigError(format!("配置{}写入失败:{}", category, e)))?;
        //.context("Failed to write config file")?;

        Ok(())
    }
}