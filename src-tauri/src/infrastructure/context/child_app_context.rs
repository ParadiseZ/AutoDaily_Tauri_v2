use crate::infrastructure::context::init_error::{InitError, InitResult};
use crate::infrastructure::devices::device_ctx::DeviceCtx;
use std::sync::Arc;
use tokio::sync::RwLock;
// 应用上下文

static CHILD_APP_CONTEXT: Arc<RwLock<Option<AppCtx>>> = Arc::new(RwLock::new( None));

pub fn init_child_app_ctx(app_ctx: AppCtx) -> InitResult<()> {
    match CHILD_APP_CONTEXT.write() {
        Ok(mut ctx) => {
            *ctx = Some(app_ctx);
            Ok(())
        },
        Err(e) => {
            Err(InitError::InitChildAppCtxFailed {e: e.to_string()})
        }
    }
}
pub struct AppCtx{
    //脚本上下文
    //pub script_ctx : Arc<ScriptCtx>,

    //规则上下文
    //pub rule_ctx : Arc<RuleCtx>,

    //调度引擎上下文
    //pub scheduler_ctx : Arc<SchedulerCtx>,

    //设备上下文
    pub device_ctx : Arc<DeviceCtx>,

    //视觉服务上下文
    //pub vision_ctx: Arc<Option<VisIonCtx>>,

    //通信上下文
    //pub communication: Arc<IpcClient>,

    //日志上下文
}