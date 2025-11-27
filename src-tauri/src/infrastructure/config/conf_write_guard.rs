use crate::infrastructure::config::conf_mgr::ConfigManager;
use ahash::AHashMap;
use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLockWriteGuard;
use crate::infrastructure::core::Serialize;

// 配置存储
pub trait ConfigCategory {
    fn default() -> Self;
}
/// 可写配置守卫（自动保存）
pub struct ConfigWriteGuard<'a, C: ConfigCategory> {
    pub(crate) config: C,
    pub(crate) path: PathBuf,
    pub(crate) manager: &'a ConfigManager,
    pub(crate) _caches_lock:
        RwLockWriteGuard<'a, AHashMap<String, (Arc<dyn Any + Send + Sync>, PathBuf)>>,
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
impl<'a, C: ConfigCategory + 'static + Clone + Serialize + Send + Sync> Drop for ConfigWriteGuard<'a, C> {
    fn drop(&mut self) {
        // 在后台异步保存（不阻塞主线程）
        let config = self.config.clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            let json = match serde_json::to_string_pretty(&config) {
                Ok(j) => j,
                Err(e) => {
                    //Log::error(&format!("配置序列化失败：{}",e));
                    eprintln!("Config serialization error: {}", e);
                    return;
                }
            };

            if let Err(e) = tokio::fs::write(&path, json).await {
                //Log::error(&format!("配置保存失败：{}",e));
                eprintln!("Config save error ({}): {}", path.display(), e);
            }
        });
    }
}
