// 日志系统

extern crate core;

// Legacy modules (will be gradually phased out).
mod api;
mod app;
mod constant;
mod domain;
pub mod infrastructure;


use crate::api::dev_test::{
    dev_capture_test, paddle_ocr_inference_test, save_captured_image, yolo_inference_test,
};
use crate::api::infrastructure::config::log_api::{
    update_log_level_cmd, update_log_dir_cmd, update_retention_days_cmd,
    get_log_config_cmd, clean_logs_now_cmd, update_child_log_level_cmd,
};
use crate::api::infrastructure::config::sys_conf::{set_system_settings_cmd};
use crate::api::infrastructure::config::vision_cache::{
    get_vision_text_cache_config_cmd, set_vision_text_cache_config_cmd,
};
use crate::api::infrastructure::img::convert_img_to_base64_cmd;
use crate::api::infrastructure::frontend_debug::frontend_debug_log_cmd;
use crate::api::infrastructure::get_uuid_v7;
use crate::api::domain::devices::{get_all_devices_cmd, get_device_by_id_cmd, save_device_cmd, delete_device_cmd, get_cpu_count_cmd};
use crate::api::domain::scripts::{get_all_scripts_cmd, get_script_by_id_cmd, save_script_cmd, delete_script_cmd,
                                  get_script_tasks_cmd,save_script_tasks_cmd, get_yolo_labels_cmd, clone_local_script_cmd};
use crate::api::domain::policy::*;
use crate::api::infrastructure::process_api::{
    cmd_device_start, cmd_device_stop, cmd_device_pause,
    cmd_sync_device_runtime_session, cmd_run_script_target,
    cmd_device_shutdown, cmd_get_running_devices, cmd_prepare_device_checkpoint,
    cmd_spawn_device, cmd_is_device_running,
};
use crate::api::domain::schedule::{
    get_assignments_by_device_cmd, save_assignment_cmd, delete_assignment_cmd, reorder_assignments_cmd,
    get_schedules_by_device_cmd, clear_schedules_cmd, clear_schedules_by_script_cmd,
    get_recovery_checkpoint_by_device_cmd,
    get_all_time_templates_cmd, save_time_template_cmd, delete_time_template_cmd,
    get_script_time_template_values_cmd, save_script_time_template_values_cmd, delete_script_time_template_values_cmd,
};
use crate::app::init_start::init_at_start;
use tauri::{App, Emitter, Manager};
use crate::api::backend_cmd::{
    backend_send_verification_code, backend_register, backend_login, backend_logout,
    backend_get_auth_session, backend_get_profile, backend_search_scripts, backend_redeem_sponsor_code,
    backend_check_update, backend_download_script, backend_upload_model, backend_download_model,
    backend_reset_password, backend_update_username, backend_upload_script,
};

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
            update_log_dir_cmd,
            update_retention_days_cmd,
            get_log_config_cmd,
            clean_logs_now_cmd,
            update_child_log_level_cmd,
            get_vision_text_cache_config_cmd,
            set_vision_text_cache_config_cmd,
            //性能设置
            //get_performance_cmd,set_performance_cmd,get_cpu_cores_cmd,
            //uuid
            get_uuid_v7,
            frontend_debug_log_cmd,
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
            get_yolo_labels_cmd,
            clone_local_script_cmd,
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
            cmd_device_start,
            cmd_device_stop,
            cmd_device_pause,
            cmd_sync_device_runtime_session,
            cmd_run_script_target,
            cmd_device_shutdown,
            cmd_prepare_device_checkpoint,
            cmd_get_running_devices,
            cmd_spawn_device,
            cmd_is_device_running,
            // 调度管理
            get_assignments_by_device_cmd,
            save_assignment_cmd,
            delete_assignment_cmd,
            reorder_assignments_cmd,
            get_schedules_by_device_cmd,
            clear_schedules_cmd,
            clear_schedules_by_script_cmd,
            get_recovery_checkpoint_by_device_cmd,
            get_all_time_templates_cmd,
            save_time_template_cmd,
            delete_time_template_cmd,
            get_script_time_template_values_cmd,
            save_script_time_template_values_cmd,
            delete_script_time_template_values_cmd,
            // 远端服务器相关
            backend_send_verification_code,
            backend_register,
            backend_login,
            backend_get_auth_session,
            backend_logout,
            backend_get_profile,
            backend_search_scripts,
            backend_redeem_sponsor_code,
            backend_check_update,
            backend_download_script,
            backend_upload_script,
            backend_upload_model,
            backend_download_model,
            backend_reset_password,
            backend_update_username,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    ort::init()
        .with_telemetry(false)
        .commit();
}
