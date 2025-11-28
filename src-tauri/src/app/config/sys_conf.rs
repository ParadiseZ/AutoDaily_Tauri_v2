use crate::app::app_error::{AppError, AppResult};
use crate::app::config::short_cut::{register_short_cut_by_config, unregister_all};
use crate::constant::project::MAIN_WINDOW;
use crate::domain::config::sys_conf::SystemConfig;
use crate::infrastructure::logging::log_trait::Log;
use tauri::{AppHandle, Manager};

pub async fn set_system_settings_app(
    app_handle: &AppHandle,
    system_settings: SystemConfig,
) -> AppResult<()> {
    // 快捷键编辑
    unregister_all(app_handle)?;
    let shortcut = system_settings.shortcut.clone();
    register_short_cut_by_config(shortcut, app_handle)?;

    // 窗口状态记忆功能处理
    //handle_window_state_setting(app_handle, old_rem_size_position, config.rem_size_position)?;

    // 窗口置顶状态处理
    handle_always_on_top_setting(app_handle, system_settings.always_on_top)?;

    // 自启动设置处理
    handle_auto_start_setting(app_handle, system_settings.auto_start)?;

    Ok(())
}

/// 处理窗口置顶设置
fn handle_always_on_top_setting(app_handle: &AppHandle, always_on_top: bool) -> AppResult<()> {
    if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
        let _ = main_window.set_always_on_top(always_on_top);
        if always_on_top {
            Log::info("窗口置顶已启用");
        } else {
            Log::info("窗口置顶已禁用");
        }
    }
    Ok(())
}

/// 处理开机自启动设置
fn handle_auto_start_setting(app_handle: &AppHandle, auto_start: bool) -> AppResult<()> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let autostart_manager = app_handle.autolaunch();

        match autostart_manager.is_enabled() {
            Ok(current_enabled) => {
                if current_enabled != auto_start {
                    if auto_start {
                        if let Err(e) = autostart_manager.enable() {
                            Log::error(&format!("启用开机自启动失败: {}", e));
                            return Err(AppError::SetConfigFailed {
                                detail: "启用开机自启动失败".to_string(),
                                e: e.to_string(),
                            });
                        } else {
                            Log::info("开机自启动已启用");
                        }
                    } else {
                        if let Err(e) = autostart_manager.disable() {
                            Log::error(&format!("禁用开机自启动失败: {}", e));
                            return Err(AppError::SetConfigFailed {
                                detail: "禁用开机自启动失败".to_string(),
                                e: e.to_string(),
                            });
                        } else {
                            Log::info("开机自启动已禁用");
                        }
                    }
                }
            }
            Err(e) => {
                Log::error(&format!("检测开机自启动状态失败: {}", e));
                return Err(AppError::SetConfigFailed {
                    detail: "检测开机自启动状态失败".to_string(),
                    e: e.to_string(),
                });
            }
        }
    }

    #[cfg(not(desktop))]
    {
        Log::warn("当前平台不支持开机自启动功能");
    }

    Ok(())
}