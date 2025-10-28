use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;
use ahash::AHashMap;
use tauri::AppHandle;
use tauri::path::BaseDirectory;
use tokio::sync::{RwLock, RwLockWriteGuard};
use crate::domain::entities::app_result::AppResult;
use crate::domain::trait_ad::config_category::ConfigCategory;

/// 配置管理器 - 管理所有配置文件
pub struct ConfigManager {
    // 内存缓存: 配置类别 -> (配置数据, 文件路径)
    pub(crate) caches: Arc<RwLock<AHashMap<String, (Arc<dyn Any + Send + Sync>, PathBuf)>>>,
    pub(crate) app_handle: AppHandle,
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            caches: Arc::clone(&self.caches),
            app_handle: self.app_handle.clone(),
        }
    }
}

impl ConfigManager{
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            caches: Arc::new(RwLock::new(AHashMap::new())),
            app_handle,
        }
    }
}


pub trait ConfMgr{
    async fn init_category<C: ConfigCategory + 'static>(&self, category: &str,
                                                  dir : Option<String>,base_dir : BaseDirectory  ) -> AppResult<()>;

    async fn get_conf<C: ConfigCategory + 'static + Clone>(&self, category: &str) -> AppResult<C>;

    async fn get_conf_mut<C: ConfigCategory + 'static>(&self, category: &str) -> AppResult<ConfigWriteGuard<'_, C>>;

    async fn save_category<C: ConfigCategory + 'static>(
        &self,
        category: &str
    ) -> AppResult<()>;
}

/// 可写配置守卫（自动保存）
pub struct ConfigWriteGuard<'a, C: ConfigCategory> {
    pub(crate) config: C,
    pub(crate) path: PathBuf,
    pub(crate) manager: &'a ConfigManager,
    pub(crate) _caches_lock: RwLockWriteGuard<'a, AHashMap<String, (Arc<dyn Any + Send + Sync>, PathBuf)>>,
}

impl<'a, C: ConfigCategory> std::ops::Deref for ConfigWriteGuard<'a, C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<'a, C: ConfigCategory> std::ops::DerefMut for ConfigWriteGuard<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}