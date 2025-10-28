use crate::infrastructure::context::child_process_sec::{set_running_status, RunningStatus};
use crate::infrastructure::core::{Deserialize, Error, Serialize};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ChildProcessError{
    #[error("child-初始化全局上下文失败: {e}")]
    FailedToInitialize { e: String },

    #[error("child-设置cpu亲和性失败:{e}")]
    FailedToSetCoreAffinity{e: String},

    #[error("child-运行主循环失败:{e}")]
    FailedToRunMainLoop{e: String},

    #[error("child-关闭失败")]
    FailedToShutdown{e:String},

    #[error("child-初始化日志失败")]
    FailedToInitializeLogging{e: String},

    #[error("child-初始化 Rayon 线程池失败")]
    FailedToInitializeRayonPool,

    #[error("child-初始化共享日志环形缓冲区失败:{e}")]
    FailedToInitializeSharedLogRingBuffer{e:String},
}
pub type ChildProcessResult<T> = Result<T, ChildProcessError>;
pub mod child_process {
    use crate::infrastructure::context::child_process::{init_data_from_main, ChildProcessCtx, ChildProcessInitData};
    use crate::infrastructure::context::child_process_sec::{get_child_process_ctx, init_child_process_ctx, set_running_status, RunningStatus};
    use crate::main_child::{ChildProcessError, ChildProcessResult};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// 子进程运行函数
    pub async fn run() -> ChildProcessResult<()> {

        // 从环境变量获取序列化的上下文数据
        let context_data = std::env::var("CHILD_CONTEXT_DATA")
            .map_err(|_| ChildProcessError::FailedToInitialize {e:"缺少子进程上下文数据环境变量".into_string()})?;
        
        let init_data: ChildProcessInitData = serde_json::from_str(&context_data)
            .map_err(|e| ChildProcessError::FailedToInitialize {e:"解析上下文数据失败".into_string()})?;

        // 根据 InitData 构建运行时上下文
        //let mut ctx = ChildProcessCtx::from_init(init_data)?;
        //ctx.initialize().await?;
        
        // 将上下文存储到全局静态变量
        //init_child_process_ctx(Arc::new(RwLock::new(ctx))).map_err(|_| ChildProcessError::FailedToInitialize {e:"初始化全局上下文失败".into_string()})?;
        // 获取上下文引用
        //let context = get_child_process_ctx().read().await;
        
        // 初始化上下文数据
        init_data.init_data_from_main()?;
        
        // 设置运行状态
        set_running_status(RunningStatus::Idle);
        // 设置信号处理器，优雅关闭
        let context_for_signal = ctx.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
            println!("Received shutdown signal, shutting down child process...");
            if let Err(e) = context_for_signal.shutdown().await {
                eprintln!("Error during shutdown: {}", e);
            }
        });
        Ok(())
    }
}

// 子进程的入口点
#[tokio::main]
async fn main() {
    if let Err(e) = child_process::run().await {
        set_running_status(RunningStatus::Error);
        eprintln!("Child process failed: {}", e);
        std::process::exit(1);
    }
}