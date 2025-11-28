use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RunningError {
    #[error("程初始化主线程上下文数据失败: {e}")]
    InitMainCtxErr { e: String },
    #[error("注册日志接收器失败: {e}")]
    RegisterLogReceiverErr { e: String },
}

pub type RunningResult<T> = Result<T, RunningError>;

pub async fn run() -> RunningResult<()> {
    /*let main_ctx = MainProcessCtx::init().await
    .map_err(|e| InitMainCtxErr {e : e.to_string()})?;*/

    let _app_handle = get_app_handle();
    //app_handle.manage( Arc::new(Mutex::new(main_ctx)) );

    Ok(())
}
