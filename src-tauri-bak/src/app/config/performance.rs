use crate::constant::sys_conf_path::PERFORMANCE_CONFIG_PATH;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::performance::Performance;
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager};

pub async fn get_performance_config(
    config_manager: tauri::State<'_, ConfigManager>
) -> AppResult<String>{
    let performance= config_manager.get_conf::<Performance>(PERFORMANCE_CONFIG_PATH).await?;
    let res = serde_json::to_string(&performance)
        .map_err(|e| AppError::ConfigError(format!("序列化配置失败：{}",e)))?;
    Ok(res)
}

pub async fn set_performance_config(
    config_manager: tauri::State<'_, ConfigManager>,
    max_devices : usize,
    cores_per_device : u32
)-> AppResult<()>{
    let mut performance= config_manager.get_conf_mut::<Performance>(PERFORMANCE_CONFIG_PATH).await?;
    performance.max_devices = max_devices;
    performance.cores_per_device = cores_per_device;
    Ok(())
}