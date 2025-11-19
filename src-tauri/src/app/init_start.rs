use sysinfo::Signal::Sys;
use tauri::{App, AppHandle, Manager, State};
use tauri::path::BaseDirectory;
use tracing::trace;
use crate::app::config::short_cut::register_short_cut_by_config;
use crate::constant::project::MAIN_WINDOW;
use crate::constant::sys_conf_path::SYSTEM_SETTINGS_PATH;
use crate::domain::config::sys_conf::{StartMode, SystemConfig};
use crate::infrastructure::app_handle::init_app_handle;
use crate::infrastructure::config::conf_init::{init_conf_async, init_conf_sync};
use crate::infrastructure::config::conf_mgr::ConfigManager;
use crate::infrastructure::logging::log_trait::Log;

pub async fn init_at_start(app_handle : &AppHandle){
    //初始化app_handle
    init_app_handle(app_handle);
    // 初始化配置管理器
    init_config_manager(app_handle);
    // 获取配置管理器
    let state = app_handle.state::<ConfigManager>();
    // 同步初始化配置，系统设置、日志设置
    if let Err(e) = init_conf_sync(&state).await{
        tracing::error!("同步配置初始化失败：{}",e);
    };
    // 获取系统配置
    let sys_conf = &state.get_conf::<SystemConfig>(SYSTEM_SETTINGS_PATH).await.unwrap();
    // 处理开机自启动
    init_autostart(app_handle, &sys_conf);
    // 初始化快捷键设置
    init_short_cut_by_config(app_handle, &sys_conf);
    // 窗口位置初始化
    init_window_position(app_handle, &sys_conf);
    // 初始化窗口关闭事件
    init_close_window_event(app_handle.clone());
    // 初始化资源路径
    init_resources_path(app_handle);
    // 初始化启动方式
    init_start_model(app_handle, &sys_conf);

    // 异步初始化配置，设备设置、脚本设置
    init_conf_async(state)
}

pub fn init_config_manager(app_handle : &AppHandle) {
    let config_manager = ConfigManager::new();
    app_handle.manage(config_manager);
}

pub fn init_autostart(app_handle : &AppHandle, sys_conf: &SystemConfig){
    // 处理开机自启动设置
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let autostart_manager = app_handle.autolaunch();

        match autostart_manager.is_enabled() {
            Ok(current_enabled) => {
                if current_enabled != sys_conf.auto_start {
                    if sys_conf.auto_start {
                        if let Err(e) = autostart_manager.enable() {
                            Log::error(&format!("启用开机自启动失败: {}", e));
                        } else {
                            Log::info("开机自启动已启用");
                        }
                    } else {
                        if let Err(e) = autostart_manager.disable() {
                            Log::error(&format!("禁用开机自启动失败: {}", e));
                        } else {
                            Log::info("开机自启动已禁用");
                        }
                    }
                } else {
                    if sys_conf.auto_start {
                        Log::info("开机自启动已启用（配置同步）");
                    } else {
                        Log::info("开机自启动已禁用（配置同步）");
                    }
                }
            }
            Err(e) => {
                Log::error(&format!("无法检查开机自启动状态: {}", e));
            }
        }
    }
    // 自启动插件初始化
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::MacosLauncher;

        let _ = app_handle
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                Some(vec!["--flag1", "--flag2"]),
            ))
            .expect("autostart config failed");
    }
}

pub fn init_short_cut_by_config(app_handle : &AppHandle, sys_conf : &SystemConfig){
    if let Err(e) = register_short_cut_by_config(sys_conf.shortcut.clone(), app_handle){
        Log::error(&format!("初始化快捷键设置失败: {}", e));
    };
}

pub fn init_close_window_event(app_handle: AppHandle){
    // 设置窗口关闭事件处理，在窗口关闭时保存状态
    if let Some(window) = app_handle.get_webview_window(MAIN_WINDOW) {
        window.on_window_event(move |event| {
            // 在窗口关闭时
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app_handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    // 临时注释掉窗口状态保存，等待重构完成
                    use crate::app::config::sys_conf::save_window_state_if_enabled;
                    save_window_state_if_enabled(&app_handle);
                    Log::info("窗口关闭事件处理");
                });
            }
        });
    }
}

pub fn init_resources_path(app: &AppHandle){
    // 静态资源路径
    #[cfg(debug_assertions)]
    {
        // 开发模式：记录资源路径
        if let Ok(resource_path) = app
            .path()
            .resolve("models", BaseDirectory::Resource)
        {
            tracing::info!("开发模式资源路径: {}", resource_path.display());
        } else {
            tracing::warn!("无法解析开发模式资源路径");
        }
    }
    #[cfg(not(debug_assertions))]
    {
        // 生产模式：记录资源路径
        if let Ok(resource_path) = app
            .path()
            .resolve("models", BaseDirectory::Resource)
        {
            tracing::info!("生产模式资源路径: {}", resource_path.display());
        } else {
            tracing::warn!("无法解析生产模式资源路径");
        }
    }
}

pub fn init_window_position(app_handle: &AppHandle, sys_conf: &SystemConfig){
    // 根据 rem_size_position 设置决定是否恢复窗口状态
    if sys_conf.rem_size_position {
        use tauri_plugin_window_state::{StateFlags};
        // 恢复窗口状态
        if let Some(main_window) = app_handle.get_webview_window(MAIN_WINDOW) {
            use tauri_plugin_window_state::WindowExt;
            let _ = main_window.restore_state(StateFlags::all());
            Log::info("窗口状态已恢复");
        }
    } else {
        Log::info("窗口状态记忆功能已禁用");
    }
}

pub fn init_start_model(app_handel: &AppHandle, sys_conf: &SystemConfig){
    // 应用启动模式处理
    if let Some(main_window) = app_handel.get_webview_window(MAIN_WINDOW) {
        match sys_conf.start_mode {
            StartMode::Normal => {
                // 普通模式：显示窗口
                let _ = main_window.show();
                Log::info("应用以普通模式启动");
            },
            StartMode::Minimized => {
                // 最小化模式：最小化窗口
                let _ = main_window.minimize();
                Log::info("应用以最小化模式启动");
            },
            StartMode::Tray => {
                // 托盘模式：隐藏窗口
                let _ = main_window.hide();
                Log::info("应用以托盘模式启动");
            }
        }

        // 设置窗口置顶状态
        if sys_conf.always_on_top {
            let _ = main_window.set_always_on_top(true);
            Log::info("窗口置顶已启用");
        }
    }
}