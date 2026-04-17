use crate::app::config::short_cut::register_short_cut_by_config;
use crate::constant::project::MAIN_WINDOW;
use crate::constant::sys_conf_path::{
    APP_STORE, EMAIL_CONFIG_KEY, LOG_CONFIG_KEY, SCRIPTS_CONFIG_KEY, SYSTEM_SETTINGS_KEY,
    VISION_TEXT_CACHE_CONFIG_KEY,
};
use crate::domain::config::notice_conf::EmailConfig;
use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::domain::config::sys_conf::{StartMode, SystemConfig};
use crate::domain::config::vision_cache_conf::VisionTextCacheConfig;
use crate::infrastructure::app_handle::init_app_handle;
use crate::infrastructure::logging::config::LogMain;
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::store_local::config_store::get_or_init_config;
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{Store, StoreExt};

pub async fn init_at_start(app_handle: &AppHandle) {
    //初始化app_handle
    init_app_handle(app_handle);
    //初始化数据库
    if let Err(e) = crate::infrastructure::db::init_db(app_handle).await {
        panic!("初始化数据库失败: {}", e);
    }
    //初始化store
    let store = match app_handle.store(APP_STORE) {
        Ok(store) => store,
        Err(e) => {
            panic!("初始化store失败: {}", e);
        }
    };
    // 初始化日志设置
    let log_conf: LogMain = get_or_init_config(store.clone(), LOG_CONFIG_KEY);
    match LogMain::init(log_conf, "AutoDaily").await {
        Ok(conf) => {
            // 注册主进程 Logger 到全局 LOGGER，使 Log::info() 等方法可用
            if let Err(e) = Log::init_logger(Box::new(conf)) {
                eprintln!("注册主进程 Logger 失败: {}", e);
            }
        }
        Err(e) => eprintln!("初始化日志系统失败: {}", e),
    }
    // 初始化子进程日志接收器
    crate::infrastructure::logging::main_process_log_handler::init_child_log_receiver();
    // 启动 IPC Server
    if let Err(e) = crate::infrastructure::ipc::chanel_server::IpcServer::start() {
        Log::error(&format!("启动 IPC Server 失败: {}", e));
    }
    // 初始化子进程管理器
    crate::infrastructure::context::child_process_manager::init_process_manager();
    // 初始化系统设置
    let sys_conf: SystemConfig = get_or_init_config(store.clone(), SYSTEM_SETTINGS_KEY);
    // 处理开机自启动
    init_autostart(app_handle, &sys_conf);
    // 初始化快捷键设置
    init_short_cut_by_config(app_handle, &sys_conf);
    // 窗口位置初始化
    //init_window_position(app_handle, &sys_conf);
    // 窗口关闭事件(可参考windows-state插件里的事件拦截)
    //init_close_window_event(app_handle.clone());
    // 初始化资源路径
    init_resources_path(app_handle);
    // 初始化启动方式
    init_start_model(app_handle, &sys_conf);

    // 异步初始化配置，设备设置、脚本设置
    init_conf_async(store)
}

pub fn init_autostart(app_handle: &AppHandle, sys_conf: &SystemConfig) {
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

pub fn init_short_cut_by_config(app_handle: &AppHandle, sys_conf: &SystemConfig) {
    if let Err(e) = register_short_cut_by_config(sys_conf.shortcut.clone(), app_handle) {
        Log::error(&format!("初始化快捷键设置失败: {}", e));
    };
}

pub fn init_resources_path(app: &AppHandle) {
    // 静态资源路径
    #[cfg(debug_assertions)]
    {
        // 开发模式：记录资源路径
        if let Ok(resource_path) = app.path().resolve("models", BaseDirectory::Resource) {
            tracing::info!("开发模式资源路径: {}", resource_path.display());
        } else {
            tracing::warn!("无法解析开发模式资源路径");
        }
    }
    #[cfg(not(debug_assertions))]
    {
        // 生产模式：记录资源路径
        if let Ok(resource_path) = app.path().resolve("models", BaseDirectory::Resource) {
            tracing::info!("生产模式资源路径: {}", resource_path.display());
        } else {
            tracing::warn!("无法解析生产模式资源路径");
        }
    }
}

pub fn init_start_model(app_handel: &AppHandle, sys_conf: &SystemConfig) {
    // 应用启动模式处理
    if let Some(main_window) = app_handel.get_webview_window(MAIN_WINDOW) {
        match sys_conf.start_mode {
            StartMode::Normal => {
                // 普通模式：显示窗口
                let _ = main_window.show();
                Log::info("应用以普通模式启动");
            }
            StartMode::Minimized => {
                // 最小化模式：最小化窗口
                let _ = main_window.minimize();
                Log::info("应用以最小化模式启动");
            }
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

pub fn init_conf_async(store: Arc<Store<Wry>>) {
    tokio::spawn(async move {
        let mut changed = false;
        // 设备设置
        /*if !store.has(DEVICES_CONFIG_KEY){
            store.set(DEVICES_CONFIG_KEY, serde_json::to_value(&HashMap::<DeviceId, DeviceConfig>::default()).unwrap_or_default());
        };*/
        // 脚本设置
        if !store.has(SCRIPTS_CONFIG_KEY) {
            store.set(
                SCRIPTS_CONFIG_KEY,
                serde_json::to_value(&ScriptsConfig::default()).unwrap_or_default(),
            );
            changed = true;
        };
        // 通知设置
        if !store.has(EMAIL_CONFIG_KEY) {
            store.set(
                EMAIL_CONFIG_KEY,
                serde_json::to_value(&EmailConfig::default()).unwrap_or_default(),
            );
            changed = true;
        };
        // OCR 文字缓存设置
        if !store.has(VISION_TEXT_CACHE_CONFIG_KEY) {
            store.set(
                VISION_TEXT_CACHE_CONFIG_KEY,
                serde_json::to_value(&VisionTextCacheConfig::default()).unwrap_or_default(),
            );
            changed = true;
        };

        if changed {
            if let Err(e) = store.save() {
                Log::error(&format!("初始化配置文件失败: {}", e));
            }
        };
    });
}
