use child_support::infrastructure::context::child_process::{init_environment, ChildProcessInitData};
use child_support::infrastructure::context::child_process_sec::{
    get_running_status, process_need_stop, set_running_status, RunningStatus,
};
use child_support::infrastructure::context::init_error::InitError;
use child_support::infrastructure::core::{Deserialize, Error, Serialize};
use child_support::infrastructure::ipc::runtime_reporter::emit_lifecycle_event;
use child_support::infrastructure::ipc::message::RuntimeLifecyclePhase;
use child_support::infrastructure::logging::log_trait::Log;
use child_support::infrastructure::scripts::scheduler::{get_scheduler, init_scheduler};
use tokio_util::sync::CancellationToken;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ChildProcessError {
    #[error(transparent)]
    InitErr(#[from] InitError),

    #[error("child-初始化全局上下文失败: {e}")]
    FailedToInitialize { e: String },

    #[error("child-运行主循环失败:{e}")]
    FailedToRunMainLoop { e: String },

    #[error("child-关闭失败")]
    FailedToShutdown { e: String },
}
pub type ChildProcessResult<T> = Result<T, ChildProcessError>;

/// 子进程的入口点
#[tokio::main]
async fn main() {
    if let Err(e) = run_child_process().await {
        set_running_status(RunningStatus::Error);
        eprintln!("Child process failed: {}", e);
        std::process::exit(1);
    }
}

/// 子进程运行函数
async fn run_child_process() -> ChildProcessResult<()> {
    // 1. 从环境变量获取序列化的上下文数据
    let context_data = std::env::var("CHILD_CONTEXT_DATA").map_err(|_| {
        ChildProcessError::FailedToInitialize {
            e: "缺少子进程上下文数据环境变量".to_string(),
        }
    })?;

    let init_data: ChildProcessInitData = serde_json::from_str(&context_data).map_err(|e| {
        ChildProcessError::FailedToInitialize {
            e: format!("反序列化上下文数据失败: {}", e),
        }
    })?;

    // 2. 初始化子进程环境（CPU亲和性、日志、数据库、IPC、ADB、运行时上下文）
    init_environment(&init_data)
        .await
        .map_err(ChildProcessError::InitErr)?;

    Log::info("[ child ] 子进程环境初始化完成");

    // 3. 创建 CancellationToken 用于优雅停止，并注册到全局
    let cancel_token = CancellationToken::new();
    child_support::infrastructure::context::child_process_sec::init_cancel_token(cancel_token.clone());

    // 4. 启动信号处理器
    {
        let token = cancel_token.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl+c");
            Log::info("[ child ] 收到 Ctrl+C 信号，准备关闭子进程");
            set_running_status(RunningStatus::Stopping);
            token.cancel();
        });
    }

    // 5. 初始化脚本调度器
    init_scheduler(cancel_token.clone());
    Log::info("[ child ] 脚本调度器已初始化");

    // 6. 设置运行状态为 Idle，等待主进程指令
    set_running_status(RunningStatus::Idle);
    Log::info("[ child ] 进入主循环（Idle），等待主进程指令");
    emit_lifecycle_event(
        RuntimeLifecyclePhase::Idle,
        Some("子进程已就绪，等待运行会话".to_string()),
    );

    // 7. 主循环
    run_main_loop(cancel_token.clone()).await;

    // 8. 清理
    Log::info("[ child ] 子进程主循环结束，执行清理");
    set_running_status(RunningStatus::Stopped);

    Ok(())
}

/// 子进程主循环
/// - Idle 状态：等待主进程发来的消息（通过 IPC 接收，由 chanel_client 的 recv_loop 处理）
/// - Running 状态：执行脚本调度
/// - Stopping 状态：退出循环
async fn run_main_loop(cancel_token: CancellationToken) {
    loop {
        // 检查是否需要停止
        if cancel_token.is_cancelled() || process_need_stop() {
            Log::info("[ child ] 检测到停止信号，退出主循环");
            break;
        }

        let status = get_running_status();
        match status {
            RunningStatus::Idle | RunningStatus::Paused => {
                // 空闲/暂停状态：短暂休眠，等待主进程发来的命令
                // 消息已由 IPC recv_loop → msg_handler_child 自动处理
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        Log::info("[ child ] Idle 中收到取消信号");
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {}
                }
            }
            RunningStatus::Running => {
                // 运行状态：执行脚本调度
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        Log::info("[ child ] Running 中收到取消信号");
                        break;
                    }
                    _ = async {
                        if let Some(scheduler) = get_scheduler() {
                            let has_more = scheduler.tick().await;
                            if !has_more {
                                // 队列执行完毕，回到 Idle 等待新任务
                                Log::info("[ child ] 脚本队列已空，回到 Idle 状态");
                                set_running_status(RunningStatus::Idle);
                                emit_lifecycle_event(
                                    RuntimeLifecyclePhase::Idle,
                                    Some("脚本队列已空".to_string()),
                                );
                            }
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }
                    } => {}
                }
            }
            RunningStatus::Stopping | RunningStatus::Stopped | RunningStatus::Error => {
                Log::info(&format!("[ child ] 状态为 {:?}，退出主循环", status));
                break;
            }
            RunningStatus::Unknown => {
                Log::warn("[ child ] 未知状态，设为 Idle");
                set_running_status(RunningStatus::Idle);
            }
        }
    }
}
