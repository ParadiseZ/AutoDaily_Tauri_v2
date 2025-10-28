use crate::app::app_error::{AppError, AppResult};
use crate::app::config::short_cut::{register_by_config, unregister_all};
use crate::constant::project::MAIN_WINDOW;
use crate::constant::sys_conf_path::SYSTEM_SETTINGS_PATH;
use crate::domain::config::sys_conf::SystemConfig;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::logging::log_trait::Log;
use tauri::{AppHandle, Manager};

pub async fn get_system_settings(
    manager: tauri::State<'_, ConfigManager>
) -> AppResult<String> {
    let system_settings = manager.get_conf::<SystemConfig>(SYSTEM_SETTINGS_PATH).await?;
    let res = serde_json::to_string(&system_settings)
        .map_err(|e| AppError::SetConfigFailed{detail: "序列化失败".to_string(), e: e.to_string()})?;
    Ok(res)
}

pub async  fn set_system_settings(
    manager: tauri::State<'_, ConfigManager>,
    config: SystemConfig
) -> AppResult<()> {
    let mut system_settings = manager.get_conf_mut::<SystemConfig>(SYSTEM_SETTINGS_PATH).await?;
    let old_rem_size_position = system_settings.rem_size_position;

    // 两种方式都可以访问和修改 SystemSettings：

    // 方式1: 通过 Deref 机制（推荐）
    *system_settings = config.clone();

    // 方式2: 直接访问 config 字段（等价但不常用）  
    // system_settings.config = config;

    // 快捷键编辑
    let app_handle = get_app_handle().await;

    unregister_all(app_handle)?;
    let shortcut = system_settings.shortcut.clone();
    register_by_config(shortcut, app_handle)?;

    // 窗口状态记忆功能处理
    handle_window_state_setting(app_handle, old_rem_size_position, config.rem_size_position)?;

    // 窗口置顶状态处理
    handle_always_on_top_setting(app_handle, config.always_on_top)?;

    // 自启动设置处理
    handle_auto_start_setting(app_handle, config.auto_start)?;

    Ok(())
}

/// 处理窗口状态记忆功能的开关
fn handle_window_state_setting(
    app_handle: &AppHandle,
    old_enabled: bool,
    new_enabled: bool,
) -> AppResult<()> {
    // 如果状态发生变化
    if old_enabled != new_enabled {
        if new_enabled {
            // 启用：立即恢复窗口状态
            if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
                use tauri_plugin_window_state::{StateFlags, WindowExt};
                let _ = main_window.restore_state(StateFlags::all());
                Log::info("窗口状态已恢复")
            }
        } else {
            // 禁用：保存一次当前状态然后清除缓存文件（可选）
            use tauri_plugin_window_state::{AppHandleExt, StateFlags};
            let _ = app_handle.save_window_state(StateFlags::all());
            Log::info("窗口状态记忆功能已禁用，当前状态已保存");
        }
    }
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
                            return Err(AppError::SetConfigFailed{detail: "启用开机自启动失败".to_string(), e: e.to_string()});
                        } else {
                            Log::info("开机自启动已启用");
                        }
                    } else {
                        if let Err(e) = autostart_manager.disable() {
                            Log::error(&format!("禁用开机自启动失败: {}", e));
                            return Err(AppError::SetConfigFailed{detail: "禁用开机自启动失败".to_string(), e: e.to_string()});
                        } else {
                            Log::info("开机自启动已禁用");
                        }
                    }
                }
            }
            Err(e) => {
                Log::error(&format!("检测开机自启动状态失败: {}", e));
                return Err(AppError::SetConfigFailed{detail: "检测开机自启动状态失败".to_string(), e: e.to_string()});
            }
        }
    }

    #[cfg(not(desktop))]
    {
        Log::warn("当前平台不支持开机自启动功能");
    }

    Ok(())
}

/// 保存窗口状态（在应用退出时调用）
pub fn save_window_state_if_enabled(
    app_handle: &AppHandle,
    sys_conf: &SystemConfig
){
    if sys_conf.rem_size_position {
        use tauri_plugin_window_state::{AppHandleExt, StateFlags};
        let _ = app_handle.save_window_state(StateFlags::all());
        Log::info("窗口状态已保存");
    }
}