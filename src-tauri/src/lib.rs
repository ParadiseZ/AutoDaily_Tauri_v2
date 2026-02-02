#![feature(portable_simd)]

// 日志系统

extern crate core;

// Legacy modules (will be gradually phased out)
mod api;
mod app;
mod constant;
mod domain;
mod infrastructure;


use crate::api::dev_test::{
    dev_capture_test, paddle_ocr_inference_test, save_captured_image, yolo_inference_test,
};
use crate::api::infrastructure::config::log_api::{update_log_level_cmd};
use crate::api::infrastructure::config::sys_conf::{set_system_settings_cmd};
use crate::api::infrastructure::img::convert_img_to_base64_cmd;
use crate::api::infrastructure::get_uuid_v7;
use crate::api::domain::devices::{get_all_devices_cmd, get_device_by_id_cmd, save_device_cmd, delete_device_cmd, get_cpu_count_cmd};
use crate::api::domain::scripts::{get_all_scripts_cmd, get_script_by_id_cmd, save_script_cmd, delete_script_cmd,
                                  get_script_tasks_cmd,save_script_tasks_cmd};
use crate::api::domain::policy::*;
use crate::app::init_start::init_at_start;
use crate::infrastructure::ipc::chanel_server::IpcServer;
use tauri::{App, Emitter, Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_single_instance::init(|app, argv, cwd| {
                println!("{}, {argv:?}, {cwd}", app.package_info().name);
                let _ = app.emit("single-instance", ());
            })
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|app: &mut App| {
            let app_handle = app.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                // 启动时初始化
                init_at_start(&app_handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            //开发者相关
            dev_capture_test,
            save_captured_image,
            yolo_inference_test,
            paddle_ocr_inference_test,
            //日志更新级别
            update_log_level_cmd,
            //性能设置
            //get_performance_cmd,set_performance_cmd,get_cpu_cores_cmd,
            //uuid
            get_uuid_v7,
            // 常规/系统设置
            set_system_settings_cmd,
            // 设备配置
            get_all_devices_cmd,
            get_device_by_id_cmd,
            save_device_cmd,
            delete_device_cmd,
            get_cpu_count_cmd,
            // 脚本配置
            get_all_scripts_cmd,
            get_script_by_id_cmd,
            save_script_cmd,
            delete_script_cmd,
            // 图像转换
            convert_img_to_base64_cmd,
            // 脚本任务
            get_script_tasks_cmd,
            save_script_tasks_cmd,
            // 策略管理
            get_all_policies_cmd,
            save_policy_cmd,
            delete_policy_cmd,
            get_all_policy_groups_cmd,
            save_policy_group_cmd,
            delete_policy_group_cmd,
            get_group_policies_cmd,
            update_group_policies_cmd,
            get_all_policy_sets_cmd,
            save_policy_set_cmd,
            delete_policy_set_cmd,
            get_set_groups_cmd,
            update_set_groups_cmd,
            // 空闲监控
                                  //start_idle_monitoring_cmd,stop_idle_monitoring_cmd,update_activity_cmd,cancel_shutdown_cmd,
                                  // 进程管理
            //退出前函数

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    IpcServer::start().expect("Failed to start to IPC server");
    ort::init()
        .with_telemetry(false)
        .commit()
        .expect("ort 关闭遥测失败！");
}
