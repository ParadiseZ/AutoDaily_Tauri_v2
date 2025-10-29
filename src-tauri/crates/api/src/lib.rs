//! # AutoDaily API Module
//! 
//! API模块提供Tauri命令和前端接口

pub mod commands;
pub mod handlers;
pub mod responses;

// 重新导出主要类型
pub use commands::*;
pub use handlers::*;
pub use responses::*;

// Tauri应用入口
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 初始化逻辑
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 这里添加命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}