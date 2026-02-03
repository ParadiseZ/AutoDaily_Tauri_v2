use crate::infrastructure::context::init_error::InitResult;
use crate::infrastructure::context::runtime_context::SharedRuntimeContext;
use crate::infrastructure::devices::device_ctx::DeviceCtx;
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::sync::RwLock;
// 应用上下文

lazy_static!{
    pub static ref  CHILD_APP_CONTEXT: Arc<RwLock<Option<AppCtx>>> = Arc::new(RwLock::new(None));
}


pub async fn init_child_app_ctx(app_ctx: AppCtx) -> InitResult<()> {
    let mut guard = CHILD_APP_CONTEXT.write().await;
    *guard = Some(app_ctx);
    Ok(())
}
pub struct AppCtx {
    //设备上下文
    pub device_ctx: Arc<DeviceCtx>,
    
    //运行时上下文
    pub runtime_ctx: SharedRuntimeContext,

    //通信上下文 (已在 IpcClient 中管理，通过 get_ipc_client 获取)

    //adb上下文(已在adb_context管理，通过get_adb_ctx获取).设备截图需要adb_ctx
    //pub adb_ctx: Arc<AdbCtx>,
}
