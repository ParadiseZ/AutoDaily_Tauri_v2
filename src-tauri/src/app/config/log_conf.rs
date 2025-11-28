use crate::app::app_error::AppResult;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::LogLevel;

pub async fn update_log_level_app(
    log_level: &LogLevel,
) -> AppResult<()> {
    LogMain::update_level(&log_level).await?;
    Ok(())
}
