#![feature(portable_simd)]

// 日志系统

extern crate core;

// Legacy modules (will be gradually phased out)
mod api;
mod app;
mod constant;
mod domain;
mod infrastructure;
pub mod main_child;

use crate::api::dev_test::{
    dev_capture_test, paddle_ocr_inference_test, save_captured_image, yolo_inference_test,
};
use crate::api::infrastructure::config::log_api::{get_log_cmd, set_log_cmd};
use crate::api::infrastructure::config::sys_conf::{
    get_system_settings_cmd, save_window_state_cmd, set_system_settings_cmd,
};
use crate::app::init_start::init_at_start;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use crate::infrastructure::ipc::chanel_trait::ChannelTrait;
use tauri::{App, Manager};

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
        .setup(|app: &mut App| {
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn(async move {
                // 启动时初始化
                init_at_start(app_handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            //开发者相关
            dev_capture_test,
            save_captured_image,
            yolo_inference_test,
            paddle_ocr_inference_test,
            //日志
            set_log_cmd,
            get_log_cmd,
            //性能设置
            //get_performance_cmd,set_performance_cmd,get_cpu_cores_cmd,
            // 常规/系统设置
            get_system_settings_cmd,
            set_system_settings_cmd,
            save_window_state_cmd // 空闲监控
                                  //start_idle_monitoring_cmd,stop_idle_monitoring_cmd,update_activity_cmd,cancel_shutdown_cmd,
                                  // 进程管理
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    IpcServer::start().expect("Failed to start to IPC server");
    ort::init()
        .with_telemetry(false)
        .commit()
        .expect("ort 关闭遥测失败！");
}
