use crate::api::local::execution::send_device_config_update;
use crate::app::app_error::AppResult;
use crate::app::constants::{APP_STORE, LOG_CONFIG_KEY};
use crate::infra::app_handle::get_app_handle;
use crate::infra::context::child_process_manager::get_process_manager;
use crate::infra::logging::LogLevel;
use crate::infra::logging::config::LogMain;
use crate::infra::logging::log_cleaner::LogCleaner;
use crate::infra::logging::log_trait::Log;
use crate::infra::logging::logger::LOG_DIR;
use ad_kernel::ids::DeviceId;
use infra_sqlite::{get_device, save_device};
use tauri_plugin_store::StoreExt;

/// 持久化日志配置到 store
fn persist_log_config_to_store(
    log_level: Option<&LogLevel>,
    log_dir: Option<&str>,
    retention_days: Option<u32>,
) {
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
pub async fn update_child_log_level_app(
    device_id: DeviceId,
    log_level: &LogLevel,
) -> AppResult<()> {
    let app_handle = get_app_handle();
    let Some(mut current) = get_device(device_id).await.map_err(|error| {
        crate::app::app_error::AppError::SetConfigFailed {
            detail: "读取设备日志级别配置".to_string(),
            e: error,
        }
    })?
    else {
        return Err(crate::app::app_error::AppError::SetConfigFailed {
            detail: "读取设备日志级别配置".to_string(),
            e: "目标设备不存在".to_string(),
        });
    };
    let device_name = current.config.device_name.clone();
    current.config.log_level = log_level.clone();
    save_device(&current).await.map_err(|error| {
        crate::app::app_error::AppError::SetConfigFailed {
            detail: "写入设备日志级别配置".to_string(),
            e: error,
        }
    })?;
    if let Some(manager) = get_process_manager() {
        if manager.is_running(&device_id).await {
            send_device_config_update(&app_handle, device_id, &current.config)
                .await
                .map_err(|error| crate::app::app_error::AppError::SetConfigFailed {
                    detail: "同步设备日志级别到子进程".to_string(),
                    e: error,
                })?;
        }
    }
    Log::info(&format!(
        "[ log ] 设备[{}]日志级别已更新为: {}",
        device_name, log_level
    ));
    Ok(())
}
