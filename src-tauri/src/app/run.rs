use crate::app::run::RunningError::InitMainCtxErr;
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::core::{Deserialize, Error, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use crate::infrastructure::logging::main_process_log_handler::SharedLogReceiver;
use crate::infrastructure::logging::child_log::ChildLogResult;
use crate::app::decision::ReactiveExecutor;

#[derive(Error, Debug,Serialize,Deserialize)]
pub enum RunningError {
    #[error("程初始化主线程上下文数据失败: {e}")]
    InitMainCtxErr { e: String },
    #[error("注册日志接收器失败: {e}")]
    RegisterLogReceiverErr{e: String}
}

pub type RunningResult<T> = Result<T, RunningError>;


pub async fn run() -> RunningResult<()>{
    let main_ctx = MainProcessCtx::init().await
        .map_err(|e| InitMainCtxErr {e : e.to_string()})?;


    let app_handle = get_app_handle();
    app_handle.manage( Arc::new(Mutex::new(main_ctx)) );

    Ok(())
}

pub async fn register_log_buffers(handle: &AppHandle) -> RunningResult<()> {
    // 使用主进程上下文中的设备配置与日志接收器
    let main_ctx = handle.state::<Arc<Mutex<MainProcessCtx>>>();
    let mut ctx = main_ctx.lock().await;
    let devices = &ctx.device_config;
    let mut receiver = ctx.log_receiver.lock().await;
    receiver
        .refresh_from_config(devices)
        .map_err(|e| RunningError::RegisterLogReceiverErr { e: e.to_string() })?;
    Ok(())
}

// 执行器定义已迁移到 app/decision