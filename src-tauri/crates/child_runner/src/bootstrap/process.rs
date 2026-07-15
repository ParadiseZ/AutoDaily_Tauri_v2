use crate::infra::{
    context::{
        ChildRuntimeInitError, RunningStatus,
        child_process::init_environment,
        runtime_control::{
            clear_stop_request, get_running_status, get_scheduler, init_cancel_token,
            init_scheduler, process_need_stop, set_running_status,
        },
    },
    ipc::runtime_reporter::{
        emit_connection_event_now, emit_dispatch_event, emit_lifecycle_event,
        emit_lifecycle_event_now, emit_progress_event,
    },
};
use infra_logging::Log;
use runner_protocol::{
    ChildProcessInitData,
    message::{
        ConnectionStatusKind, RuntimeDispatchPhase, RuntimeLifecyclePhase, RuntimeProgressPhase,
    },
};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Error, Debug)]
enum ChildProcessError {
    #[error(transparent)]
    Init(#[from] ChildRuntimeInitError),

    #[error("child-初始化全局上下文失败: {0}")]
    Initialize(String),
}

/// 在独立 Tokio runtime 中运行 child；组合根只负责选择此入口。
pub fn run_child_process_entry() {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Child process failed: 初始化 Tokio runtime 失败: {error}");
            std::process::exit(1);
        }
    };
    runtime.block_on(run_child_process_or_exit());
}

async fn run_child_process_or_exit() {
    if let Err(error) = run_child_process().await {
        set_running_status(RunningStatus::Error);
        let message = error.to_string();
        let _ = emit_lifecycle_event_now(RuntimeLifecyclePhase::Error, Some(message.clone())).await;
        let _ = emit_connection_event_now(
            ConnectionStatusKind::DeviceDisconnected,
            Some(message.clone()),
        )
        .await;
        eprintln!("Child process failed: {message}");
        std::process::exit(1);
    }
}

async fn run_child_process() -> Result<(), ChildProcessError> {
    let context_data = std::env::var("CHILD_CONTEXT_DATA")
        .map_err(|_| ChildProcessError::Initialize("缺少子进程上下文数据环境变量".to_string()))?;
    let init_data: ChildProcessInitData = serde_json::from_str(&context_data).map_err(|error| {
        ChildProcessError::Initialize(format!("反序列化上下文数据失败: {error}"))
    })?;

    init_environment(&init_data).await?;
    Log::info("[ child ] 子进程环境初始化完成");

    let cancel_token = CancellationToken::new();
    init_cancel_token(cancel_token.clone());
    spawn_signal_handler(cancel_token.clone());
    init_scheduler(cancel_token.clone())?;

    Log::info("[ child ] 脚本调度器已初始化");
    set_running_status(RunningStatus::Idle);
    emit_lifecycle_event(
        RuntimeLifecyclePhase::Idle,
        Some("子进程已就绪，等待运行会话".to_string()),
    );

    run_main_loop(cancel_token).await;

    Log::info("[ child ] 子进程主循环结束，执行清理");
    set_running_status(RunningStatus::Stopped);
    let _ = emit_connection_event_now(
        ConnectionStatusKind::DeviceDisconnected,
        Some("子进程已结束".to_string()),
    )
    .await;
    Ok(())
}

fn spawn_signal_handler(cancel_token: CancellationToken) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        Log::info("[ child ] 收到 Ctrl+C 信号，准备关闭子进程");
        set_running_status(RunningStatus::Stopping);
        cancel_token.cancel();
    });
}

async fn run_main_loop(cancel_token: CancellationToken) {
    loop {
        if cancel_token.is_cancelled() || process_need_stop() {
            Log::info("[ child ] 检测到停止信号，退出主循环");
            break;
        }

        match get_running_status() {
            RunningStatus::Idle | RunningStatus::Paused => {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        Log::info("[ child ] Idle 中收到取消信号");
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {}
                }
            }
            RunningStatus::Running => run_scheduler_tick(&cancel_token).await,
            RunningStatus::Stopping => finish_stop().await,
            status @ (RunningStatus::Stopped | RunningStatus::Error) => {
                Log::info(&format!("[ child ] 状态为 {status:?}，退出主循环"));
                break;
            }
            RunningStatus::Unknown => {
                Log::warn("[ child ] 未知状态，设为 Idle");
                set_running_status(RunningStatus::Idle);
            }
        }
    }
}

async fn run_scheduler_tick(cancel_token: &CancellationToken) {
    tokio::select! {
        _ = cancel_token.cancelled() => Log::info("[ child ] Running 中收到取消信号"),
        _ = async {
            let Some(scheduler) = get_scheduler() else {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                return;
            };
            if scheduler.tick().await {
                return;
            }

            Log::info("[ child ] 脚本队列已空，回到 Idle 状态");
            set_running_status(RunningStatus::Idle);
            emit_lifecycle_event(
                RuntimeLifecyclePhase::Idle,
                Some("脚本队列已空".to_string()),
            );
            emit_dispatch_event(
                None,
                None,
                None,
                RuntimeDispatchPhase::RequestNext,
                Some("当前 dispatch 已结束，请求下一个 dispatch".to_string()),
            );
        } => {}
    }
}

async fn finish_stop() {
    let should_finalize = get_scheduler()
        .map(|scheduler| scheduler.current_script_snapshot().is_none())
        .unwrap_or(true);
    if !should_finalize {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        return;
    }

    if let Some(scheduler) = get_scheduler() {
        scheduler.clear_queue().await;
    }
    clear_stop_request();
    set_running_status(RunningStatus::Idle);
    emit_progress_event(
        RuntimeProgressPhase::Idle,
        None,
        None,
        None,
        None,
        Some("停止完成，已回到待命状态".to_string()),
    );
    emit_lifecycle_event(
        RuntimeLifecyclePhase::Idle,
        Some("停止完成，已回到待命状态".to_string()),
    );
}
