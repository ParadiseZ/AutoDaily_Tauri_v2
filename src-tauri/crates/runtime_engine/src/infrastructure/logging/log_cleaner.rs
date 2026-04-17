use chrono::Local;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing;

/// 日志自动清理器
/// 定时扫描日志目录，删除超过保留天数的日志文件
pub struct LogCleaner;

/// 保留天数（可动态修改）
static RETENTION_DAYS: AtomicU32 = AtomicU32::new(7);

impl LogCleaner {
    /// 更新保留天数
    pub fn set_retention_days(days: u32) {
        RETENTION_DAYS.store(days, Ordering::Relaxed);
        tracing::info!("日志保留天数更新为: {} 天", days);
    }

    /// 获取当前保留天数
    pub fn get_retention_days() -> u32 {
        RETENTION_DAYS.load(Ordering::Relaxed)
    }

    /// 启动定时清理任务
    /// - 启动时立即执行一次清理
    /// - 之后每 6 小时执行一次
    pub async fn start(log_dir: PathBuf, initial_days: u32) {
        RETENTION_DAYS.store(initial_days, Ordering::Relaxed);

        // 启动时立即清理一次
        Self::clean_once(&log_dir).await;

        // 定时清理循环
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 3600));
        interval.tick().await; // 消耗第一个立即触发的 tick
        loop {
            interval.tick().await;
            Self::clean_once(&log_dir).await;
        }
    }

    /// 执行一次清理
    pub async fn clean_once(log_dir: &PathBuf) {
        let retention_days = RETENTION_DAYS.load(Ordering::Relaxed);
        if retention_days == 0 {
            return; // 0 表示不自动清理
        }

        let now = Local::now();
        let threshold = chrono::Duration::days(retention_days as i64);

        tracing::debug!("开始扫描日志目录清理: {}", log_dir.display());

        let dir = match tokio::fs::read_dir(log_dir).await {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("读取日志目录失败: {}", e);
                return;
            }
        };

        let mut dir = dir;
        let mut cleaned_count = 0u32;

        while let Ok(Some(entry)) = dir.next_entry().await {
            let path = entry.path();

            // 只处理 .log 文件
            if !path.is_file() {
                continue;
            }
            match path.extension().and_then(|e| e.to_str()) {
                Some("log") => {}
                _ => continue,
            }

            // 检查文件修改时间
            let metadata = match tokio::fs::metadata(&path).await {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = match metadata.modified() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let modified_time: chrono::DateTime<Local> = modified.into();
            let age = now.signed_duration_since(modified_time);

            if age > threshold {
                match tokio::fs::remove_file(&path).await {
                    Ok(_) => {
                        cleaned_count += 1;
                        tracing::debug!("已清理过期日志: {}", path.display());
                    }
                    Err(e) => {
                        tracing::warn!("清理日志文件失败 {}: {}", path.display(), e);
                    }
                }
            }
        }

        if cleaned_count > 0 {
            tracing::info!("日志清理完成，共删除 {} 个过期文件", cleaned_count);
        }
    }

    /// 手动触发清理（由前端 API 调用）
    pub async fn clean_now(log_dir: &PathBuf) {
        tracing::info!("手动触发日志清理...");
        Self::clean_once(log_dir).await;
    }
}
