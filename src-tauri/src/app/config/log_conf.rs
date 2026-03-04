use crate::app::app_error::AppResult;
use crate::constant::sys_conf_path::{APP_STORE, LOG_CONFIG_KEY};
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::log_cleaner::LogCleaner;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::logging::logger::LOG_DIR;
use crate::infrastructure::logging::LogLevel;
use tauri_plugin_store::StoreExt;

/// 持久化日志配置到 store
fn persist_log_config_to_store(log_level: Option<&LogLevel>, log_dir: Option<&str>, retention_days: Option<u32>) {
    let app = get_app_handle();
    if let Ok(store) = app.store(APP_STORE) {
        // 读取当前配置
        let mut conf: LogMain = store
            .get(LOG_CONFIG_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // 按需更新
        if let Some(level) = log_level {
            conf.log_level = level.clone();
        }
        if let Some(dir) = log_dir {
            conf.log_dir = dir.to_string();
        }
        if let Some(days) = retention_days {
            conf.retention_days = days;
        }

        // 写回 store
        if let Ok(value) = serde_json::to_value(&conf) {
            store.set(LOG_CONFIG_KEY, value);
        }
    }
}

/// 更新主进程日志级别（立即生效 + 持久化）
pub async fn update_log_level_app(log_level: &LogLevel) -> AppResult<()> {
    LogMain::update_level(log_level).await?;
    persist_log_config_to_store(Some(log_level), None, None);
    Ok(())
}

/// 更新日志目录（立即生效 + 持久化）
pub async fn update_log_dir_app(log_dir: &str) -> AppResult<()> {
    LogMain::update_log_dir(log_dir).await?;
    persist_log_config_to_store(None, Some(log_dir), None);
    Ok(())
}

/// 更新日志保留天数（立即生效 + 持久化）
pub async fn update_retention_days_app(days: u32) -> AppResult<()> {
    LogCleaner::set_retention_days(days);
    persist_log_config_to_store(None, None, Some(days));
    Ok(())
}

/// 获取当前日志配置
pub async fn get_log_config_app() -> AppResult<LogMain> {
    let app = get_app_handle();
    if let Ok(store) = app.store(APP_STORE) {
        if let Some(value) = store.get(LOG_CONFIG_KEY) {
            if let Ok(config) = serde_json::from_value::<LogMain>(value.clone()) {
                return Ok(config);
            }
        }
    }
    // 回退到默认值
    Ok(LogMain::default())
}

/// 手动触发日志清理
pub async fn clean_logs_now_app() -> AppResult<()> {
    let log_dir = LOG_DIR.read().await.clone();
    LogCleaner::clean_now(&log_dir).await;
    Ok(())
}

/// 更新子进程日志级别（通过 IPC 发送 + 持久化到数据库 device_config）
pub async fn update_child_log_level_app(device_id: DeviceId, log_level: &LogLevel) -> AppResult<()> {
    // TODO: 第二阶段 - 通过 IPC 发送 ConfigUpdate 消息到子进程
    // 当前先记录日志，IPC 实现将在子进程功能完成时补充
    Log::info(
        &format!("[ log ] 子进程[{}]日志级别更新为: {} (IPC发送待第二阶段实现)", device_id, log_level)
    );
    Ok(())
}
