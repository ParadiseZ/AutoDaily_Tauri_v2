// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("安装 rustls ring CryptoProvider 失败");

    #[cfg(feature = "child-bin")]
    {
        if std::env::args().any(|arg| arg == "--child") {
            child_runner::bootstrap::run_child_process_entry();
            return;
        }
    }

    auto_daily_lib::run()
}
