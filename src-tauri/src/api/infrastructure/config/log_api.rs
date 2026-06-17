use crate::app::config::log_conf::{
    clean_logs_now_app, get_log_config_app, update_child_log_level_app, update_log_dir_app,
    update_log_level_app, update_retention_days_app,
};
use crate::constant::table_name::DEVICE_TABLE;
use crate::domain::devices::device_conf::DeviceTable;
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::db::DbRepo;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::LogLevel;
use chrono::Local;
use serde::Serialize;
use std::path::Path;
use tauri::command;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLogHistoryEntry {
    pub device_id: DeviceId,
    pub level: LogLevel,
    pub message: String,
    pub time: String,
}

fn parse_log_level(value: &str) -> Option<LogLevel> {
    match value.trim() {
        "Debug" => Some(LogLevel::Debug),
        "Info" => Some(LogLevel::Info),
        "Warn" => Some(LogLevel::Warn),
        "Error" => Some(LogLevel::Error),
        "Off" => Some(LogLevel::Off),
        _ => None,
    }
}

fn parse_log_line(device_id: DeviceId, line: &str) -> Option<DeviceLogHistoryEntry> {
    let trimmed = line.trim();
    let (timestamp, rest) = trimmed.split_once(" [")?;
    let (_, time) = timestamp.split_once(' ')?;
    let (level_text, message) = rest.split_once("] ")?;
    let level = parse_log_level(level_text)?;

    Some(DeviceLogHistoryEntry {
        device_id,
        level,
        message: message.to_string(),
        time: time.to_string(),
    })
}

fn read_device_log_file(
    log_path: &Path,
    device_id: DeviceId,
) -> Result<Vec<DeviceLogHistoryEntry>, String> {
    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(log_path)
        .map_err(|error| format!("读取日志文件失败 {}: {}", log_path.display(), error))?;

    Ok(content
        .lines()
        .filter_map(|line| parse_log_line(device_id, line))
        .collect())
}

fn clear_device_log_file(log_path: &Path) -> Result<(), String> {
    if !log_path.exists() {
        return Ok(());
    }

    std::fs::write(log_path, "")
        .map_err(|error| format!("清空日志文件失败 {}: {}", log_path.display(), error))
}

/// 更新主进程日志级别
#[command]
pub async fn update_log_level_cmd(log_level: LogLevel) -> Result<String, String> {
    match update_log_level_app(&log_level).await {
        Ok(_) => Ok("设置成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}

/// 更新日志目录
#[command]
pub async fn update_log_dir_cmd(log_dir: String) -> Result<String, String> {
    match update_log_dir_app(&log_dir).await {
        Ok(_) => Ok("日志目录更新成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}

/// 更新日志保留天数
#[command]
pub async fn update_retention_days_cmd(days: u32) -> Result<String, String> {
    match update_retention_days_app(days).await {
        Ok(_) => Ok("保留天数更新成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}

/// 获取当前日志配置
#[command]
pub async fn get_log_config_cmd() -> Result<LogMain, String> {
    match get_log_config_app().await {
        Ok(config) => Ok(config),
        Err(err) => Err(format!("获取失败：{}", err)),
    }
}

/// 手动触发日志清理
#[command]
pub async fn clean_logs_now_cmd() -> Result<String, String> {
    match clean_logs_now_app().await {
        Ok(_) => Ok("清理完成！".to_string()),
        Err(err) => Err(format!("清理失败：{}", err)),
    }
}

/// 读取当日设备日志文件
#[command]
pub async fn read_today_device_logs_cmd(
    device_id: Option<DeviceId>,
) -> Result<Vec<DeviceLogHistoryEntry>, String> {
    let today = Local::now().format("%y%m%d").to_string();
    let log_dir = LogMain::get_log_dir().await;

    let devices = if let Some(target_device_id) = device_id {
        DbRepo::get_by_id::<DeviceTable>(DEVICE_TABLE, &target_device_id.to_string())
            .await?
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await?
    };

    let mut entries = Vec::new();
    for device in devices {
        let device_name = device.data.0.device_name.trim();
        if device_name.is_empty() {
            continue;
        }

        let file_path = log_dir.join(format!("{}_{}.log", device_name, today));
        match read_device_log_file(&file_path, device.id) {
            Ok(mut file_entries) => entries.append(&mut file_entries),
            Err(error) => tracing::warn!("{}", error),
        }
    }

    entries.sort_by(|left, right| {
        left.time
            .cmp(&right.time)
            .then_with(|| left.device_id.to_string().cmp(&right.device_id.to_string()))
    });
    Ok(entries)
}

/// 清空当日设备日志文件
#[command]
pub async fn clear_today_device_logs_cmd(device_id: Option<DeviceId>) -> Result<String, String> {
    let today = Local::now().format("%y%m%d").to_string();
    let log_dir = LogMain::get_log_dir().await;

    let devices = if let Some(target_device_id) = device_id {
        DbRepo::get_by_id::<DeviceTable>(DEVICE_TABLE, &target_device_id.to_string())
            .await?
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        DbRepo::get_all::<DeviceTable>(DEVICE_TABLE).await?
    };

    for device in devices {
        let device_name = device.data.0.device_name.trim();
        if device_name.is_empty() {
            continue;
        }

        let file_path = log_dir.join(format!("{}_{}.log", device_name, today));
        clear_device_log_file(&file_path)?;
    }

    Ok("清空成功！".to_string())
}

/// 更新子进程日志级别
#[command]
pub async fn update_child_log_level_cmd(
    device_id: DeviceId,
    log_level: LogLevel,
) -> Result<String, String> {
    match update_child_log_level_app(device_id, &log_level).await {
        Ok(_) => Ok("子进程日志级别更新成功！".to_string()),
        Err(err) => Err(format!("设置失败：{}", err)),
    }
}
