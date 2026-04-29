use tauri::{command, WebviewWindow};

#[command]
pub async fn frontend_debug_log_cmd(
    level: String,
    message: String,
    details: Option<String>,
) -> Result<(), String> {
    let normalized_level = level.to_ascii_lowercase();
    let prefix = format!("[frontend:{}] {}", normalized_level, message);

    match details {
        Some(extra) if !extra.trim().is_empty() => {
            if normalized_level == "error" || normalized_level == "warn" {
                eprintln!("{prefix}\n{extra}");
            } else {
                println!("{prefix}\n{extra}");
            }
        }
        _ => {
            if normalized_level == "error" || normalized_level == "warn" {
                eprintln!("{prefix}");
            } else {
                println!("{prefix}");
            }
        }
    }

    Ok(())
}

#[command]
pub fn open_current_devtools_cmd(window: WebviewWindow) -> Result<(), String> {
    window.open_devtools();
    Ok(())
}
