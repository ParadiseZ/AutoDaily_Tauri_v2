#![feature(portable_simd)]

// DDD Architecture Modules
mod app;
mod domain;
mod infrastructure;

// Legacy modules (will be gradually phased out)
mod constant;
mod command;

use std::sync::Arc;
use tauri::{Manager, State};
use tauri::path::BaseDirectory;
use crate::app::config::shortcut::register_by_config;
use crate::command::{cancel_shutdown_cmd, get_cpu_cores_cmd, get_log_cmd, get_performance_cmd, get_system_settings_cmd, paddle_ocr_inference_test, save_captured_image, save_window_state_cmd, set_log_cmd, set_performance_cmd, set_system_settings_cmd, start_idle_monitoring_cmd, stop_idle_monitoring_cmd, update_activity_cmd, window_capture_test, yolo_inference_test, get_system_performance_info, start_test_process, terminate_process, get_active_processes_info, cleanup_finished_processes, get_process_output, start_simple_test_process, start_cpu_intensive_process, start_parallel_processes};
use crate::constant::project::MAIN_WINDOW;
use crate::constant::sys_conf_path::{PERFORMANCE_CONFIG_PATH, SYSTEM_SETTINGS_PATH};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::entities::config::performance::Performance;
use crate::domain::entities::config::sys_conf::{StartModel, SystemConfig};
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager};
use crate::infrastructure::entities::config::idle_monitor::IdleMonitor;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            tauri::async_runtime::spawn(async move {
                domain::app_handle::GLOBAL_APP_HANDLE.get_or_init(|| async { app.handle().clone() }).await;
            });
            let app_handle = app.handle().clone();
            let app_handle_2 = app_handle.clone();
            let config_manager = ConfigManager::new(app_handle.clone());
            let config_manager_2 = config_manager.clone();
            let idle_monitor = Arc::new(IdleMonitor::new());
            
            // 先注册到全局状态，确保 Tauri 命令可以访问
            app_handle.manage(config_manager.clone());
            app_handle.manage(idle_monitor.clone());
            tauri::async_runtime::block_on(async{
                if let Err(err) = Log::init(config_manager, "AutoDaily").await {
                    eprintln!("初始化日志系统失败: {}", err);
                    std::process::exit(1);
                }
            });

            // 其他配置在后台异步初始化
            tauri::async_runtime::spawn(async move {
                if let Err(e) = config_manager_2.init_category::<SystemConfig>(SYSTEM_SETTINGS_PATH,BaseDirectory::AppConfig).await {
                    Log::error(&format!("系统设置初始化失败：{}",e)); }

                let system_conf = match config_manager_2.get_conf::<SystemConfig>(SYSTEM_SETTINGS_PATH).await{
                    Ok( res ) => res,
                    Err( _ ) => panic!()
                };
                
                // 根据 rem_size_position 设置决定是否恢复窗口状态
                if system_conf.rem_size_position {
                    use tauri_plugin_window_state::{StateFlags};
                    // 恢复窗口状态
                    if let Some(main_window) = app_handle_2.get_webview_window(MAIN_WINDOW) {
                        use tauri_plugin_window_state::WindowExt;
                        let _ = main_window.restore_state(StateFlags::all());
                        Log::info("窗口状态已恢复");
                    }
                } else {
                    Log::info("窗口状态记忆功能已禁用");
                }

                // 应用启动模式处理
                if let Some(main_window) = app_handle_2.get_webview_window(MAIN_WINDOW) {
                    match system_conf.start_mode {
                        StartModel::Normal => {
                            // 普通模式：显示窗口
                            let _ = main_window.show();
                            Log::info("应用以普通模式启动");
                        },
                        StartModel::Minimized => {
                            // 最小化模式：最小化窗口
                            let _ = main_window.minimize();
                            Log::info("应用以最小化模式启动");
                        },
                        StartModel::Tray => {
                            // 托盘模式：隐藏窗口
                            let _ = main_window.hide();
                            Log::info("应用以托盘模式启动");
                        }
                    }

                    // 设置窗口置顶状态
                    if system_conf.always_on_top {
                        let _ = main_window.set_always_on_top(true);
                        Log::info("窗口置顶已启用");
                    }
                }

                // 处理开机自启动设置
                #[cfg(desktop)]
                {
                    use tauri_plugin_autostart::ManagerExt;
                    let autostart_manager = app_handle_2.autolaunch();
                    
                    match autostart_manager.is_enabled() {
                        Ok(current_enabled) => {
                            if current_enabled != system_conf.auto_start {
                                if system_conf.auto_start {
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
                                if system_conf.auto_start {
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
                //快捷键注册
                let _ = register_by_config(system_conf.shortcut, &app_handle_2.clone());

                //性能设置初始化
                if let Err(e) = config_manager_2.init_category::<Performance>(PERFORMANCE_CONFIG_PATH,BaseDirectory::AppConfig).await {
                    Log::error(&format!("性能设置初始化失败：{}",e));
                }
                Log::info("其他配置初始化完成");
            });

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

            // 自启动插件初始化
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::MacosLauncher;

                let _ = app
                    .handle()
                    .plugin(tauri_plugin_autostart::init(
                        MacosLauncher::LaunchAgent,
                        Some(vec!["--flag1", "--flag2"]),
                    ))
                    .expect("autostart config failed");
            }

            // 设置窗口关闭事件处理，在窗口关闭时保存状态
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let app_handle_for_close = app_handle.clone();
                window.on_window_event(move |event| {
                    // 在窗口关闭时
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let app_handle = app_handle_for_close.clone();
                        tauri::async_runtime::spawn(async move {
                            use crate::app::config::sys_conf::save_window_state_if_enabled;
                            let state: State<ConfigManager> = app_handle.state::<ConfigManager>();
                            if let Err(e) = save_window_state_if_enabled(state).await {
                                Log::error(&format!("保存窗口状态失败: {}", e));
                            }
                        });
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            //开发者相关
            window_capture_test,
            save_captured_image,
            yolo_inference_test,
            paddle_ocr_inference_test,
            //日志
            set_log_cmd,get_log_cmd,
            //性能设置
            get_performance_cmd,set_performance_cmd,get_cpu_cores_cmd,
            // 常规/系统设置
            get_system_settings_cmd,set_system_settings_cmd,save_window_state_cmd,
            // 空闲监控
            start_idle_monitoring_cmd,stop_idle_monitoring_cmd,update_activity_cmd,cancel_shutdown_cmd,
            // 进程管理
            get_system_performance_info,
            start_test_process,
            terminate_process,
            get_active_processes_info,
            cleanup_finished_processes,
            get_process_output,
            start_simple_test_process,
            start_cpu_intensive_process,
            start_parallel_processes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
