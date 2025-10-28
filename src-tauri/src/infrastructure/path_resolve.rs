use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri::path::BaseDirectory;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::core::{Deserialize, Error, Serialize};

pub mod model_path;
pub mod path_error;
pub mod other_path;