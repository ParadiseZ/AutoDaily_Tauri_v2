use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::manager::conf_mgr::ConfigWriteGuard;
use crate::domain::trait_ad::config_category::ConfigCategory;

impl<'a, C: ConfigCategory + 'static> Drop for ConfigWriteGuard<'a, C> {
    fn drop(&mut self) {
        // 在后台异步保存（不阻塞主线程）
        let config = self.config.clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            let json = match serde_json::to_string_pretty(&config) {
                Ok(j) => j,
                Err(e) => {
                    Log::error(&format!("配置序列化失败：{}",e));
                    eprintln!("Config serialization error: {}", e);
                    return;
                }
            };

            if let Err(e) = tokio::fs::write(&path, json).await {
                Log::error(&format!("配置保存失败：{}",e));
                eprintln!("Config save error ({}): {}", path.display(), e);
            }
        });
    }
}