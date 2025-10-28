use crate::domain::config::scripts_conf::ScriptsConfig;
use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::core::{DeviceId, HashMap};
use crate::infrastructure::devices::device_conf::DeviceConfMap;
use crate::infrastructure::ipc::chanel_server::IpcClientState;
use crate::infrastructure::logging::config::Log;
use crate::infrastructure::scripts::script_info_model::ScriptManager;
use memmap2::Mmap;
use std::sync::{Arc, RwLock};
use tauri::Manager;

pub type MemoryMap = Arc<RwLock<Vec<(String, Mmap)>>>;

/// 主进程上下文 - 优化的数据存储策略
pub struct MainProcessCtx{
    /// 脚本管理器（使用分页+缓存，不全量加载）
    pub script_manager: Arc<RwLock<ScriptManager>>,

    /// 设备配置（通常数量有限，可以全量加载到内存）
    pub devices_config: Arc<RwLock<DeviceConfMap>>,

    /// IPC通道映射（运行时数据，必须在内存中）
    pub ipc_servers: Arc<RwLock<HashMap< Arc<DeviceId>, Arc<IpcClientState> >>>,

    /// 模型存储: hash-内存映射
    pub model_data : Arc<RwLock<HashMap<Arc<DeviceId>, MemoryMap >>>,

}

impl MainProcessCtx {
    pub async fn init_scripts_mgr(script_cache_size: usize, ) -> InitResult<()> {
        Log::info("初始化脚本管理器...");
        // 创建脚本管理器并加载索引
        let mut script_manager = ScriptManager::new(script_cache_size);

        // 加载所有脚本数据
        script_manager.load_from_directory(ScriptsConfig::get_dir()).await
            .map_err(|e| InitError::InitMainScriptMgrErr { e: e.to_string()})?;
        Ok(())
    }
    
    /// 搜索脚本数据
    pub async fn init_(
        &mut self, 
        request: crate::infrastructure::scripts::script_info_model::ScriptPageReq
    ) -> Result<crate::infrastructure::scripts::script_info_model::ScriptPageResp, Box<dyn std::error::Error>> {
        Log::info("初始化脚本管理器...");
        self.script_manager.read().await.get_scripts_page(request)
    }

/*    /// 添加设备配置（小量数据，直接存储在内存）
    pub fn add_device_config(&mut self, config: DeviceConfig) {
        self.device_config.push(config);
    }

    /// 获取设备配置（从内存中快速获取）
    pub fn get_device_config(&self, device_id: &DeviceId) -> Option<&DeviceConfig> {
        self.device_config.iter().find(|config| config.device_id == *device_id)
    }*/

    pub fn run(&self){

    }
}