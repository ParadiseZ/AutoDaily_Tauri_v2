use crate::app::app_error::AppResult;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::log_cleaner::LogCleaner;
use crate::infrastructure::logging::logger::LOG_DIR;
use crate::infrastructure::logging::LogLevel;

/// 更新主进程日志级别
pub async fn update_log_level_app(log_level: &LogLevel) -> AppResult<()> {
    LogMain::update_level(log_level).await?;
    Ok(())
}

/// 更新日志目录
pub async fn update_log_dir_app(log_dir: &str) -> AppResult<()> {
    LogMain::update_log_dir(log_dir).await?;
    Ok(())
}

/// 更新日志保留天数
pub async fn update_retention_days_app(days: u32) -> AppResult<()> {
    LogCleaner::set_retention_days(days);
    Ok(())
}

/// 获取当前日志配置
pub async fn get_log_config_app() -> AppResult<LogMain> {
    let log_dir = LOG_DIR.read().await;
    let retention_days = LogCleaner::get_retention_days();
    Ok(LogMain {
        log_level: LogLevel::Info, // 当前级别无法从 reload handle 反查，返回默认值
        log_dir: log_dir.to_string_lossy().to_string(),
        retention_days,
    })
}

/// 手动触发日志清理
pub async fn clean_logs_now_app() -> AppResult<()> {
    let log_dir = LOG_DIR.read().await.clone();
    LogCleaner::clean_now(&log_dir).await;
    Ok(())
}

/// 更新子进程日志级别（通过 IPC 发送到子进程）
pub async fn update_child_log_level_app(device_id: DeviceId, log_level: &LogLevel) -> AppResult<()> {
    // TODO: 第二阶段实现 - 通过 IPC 发送 ConfigUpdate 消息到子进程
    // IpcServer::send_msg(IpcMessage::new(device_id, MessageType::Command,
    //     MessagePayload::ConfigUpdate(ConfigUpdateMessage { log_level })), device_id);
    crate::infrastructure::logging::log_trait::Log::info(
        &format!("[ log ] 子进程[{}]日志级别更新为: {} (IPC发送待第二阶段实现)", device_id, log_level)
    );
    Ok(())
}
