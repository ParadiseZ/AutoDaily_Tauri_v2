use crate::infra::logging::log_trait::Log;
use tauri::{WebviewWindow, command};

#[command]
pub async fn frontend_debug_log_cmd(
    level: String,
    message: String,
    details: Option<String>,
) -> Result<(), String> {
    let normalized_level = level.to_ascii_lowercase();
    let full_message = match details {
        Some(extra) if !extra.trim().is_empty() => {
            format!("[frontend:{}] {}\n{}", normalized_level, message, extra)
        }
        _ => format!("[frontend:{}] {}", normalized_level, message),
    };

    match normalized_level.as_str() {
        "error" => Log::error(&full_message),
        "warn" | "warning" => Log::warn(&full_message),
        "debug" => Log::debug(&full_message),
        _ => Log::info(&full_message),
    }

    Ok(())
}

#[command]
pub fn open_current_devtools_cmd(window: WebviewWindow) -> Result<(), String> {
    window.open_devtools();
    Ok(())
}
