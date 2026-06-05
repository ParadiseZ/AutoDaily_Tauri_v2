// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "child-bin")]
mod main_child;

fn main() {
    #[cfg(feature = "child-bin")]
    {
        if std::env::args().any(|arg| arg == "--child") {
            main_child::run_child_process_entry();
            return;
        }
    }

    auto_daily_lib::run()
}
