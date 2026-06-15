use crate::infrastructure::adb_cli_local::adb_command::ADBCmdConv;
use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_executor::ADBExecutor;
use crate::infrastructure::logging::log_trait::Log;
//use core_affinity::CoreId;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::Mutex;

static ADB_CONTEXT: OnceLock<ADBCtx> = OnceLock::new();
const ADB_COMMAND_SEND_TIMEOUT_MS: u64 = 10000;
const ADB_COMMAND_RESULT_TIMEOUT_MS: u64 = 10000;
const ADB_REBOOT_COMMAND_RESULT_TIMEOUT_MS: u64 = 10000;

fn default_command_result_timeout_ms(adb_command: &ADBCommand) -> u64 {
    match adb_command {
        ADBCommand::Reboot => ADB_REBOOT_COMMAND_RESULT_TIMEOUT_MS,
        _ => ADB_COMMAND_RESULT_TIMEOUT_MS,
    }
}

pub fn get_adb_ctx() -> &'static ADBCtx {
    try_get_adb_ctx().expect("ADB context not initialized")
}

pub fn try_get_adb_ctx() -> Result<&'static ADBCtx, String> {
    ADB_CONTEXT
        .get()
        .ok_or_else(|| "ADB上下文未初始化，请先完成设备连接准备".to_string())
}

impl std::fmt::Display for ADBCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        tokio::runtime::Handle::current().block_on(async move {
            write!(
                f,
                "ADB_CONTEXT config:{:?},cmd_channel:{},cmd_loop_channel:{},cmd_cur_queue:{}",
                self.adb_config.clone().lock().await,
                self.cmd_sender.len(),
                self.cmd_loop_sender.len(),
                self.cmds_after_conversion.clone().lock().await.iter().len()
            )
        })
    }
}

pub struct ADBCtx {
    pub adb_config: Arc<Mutex<ADBConnectConfig>>,
    pub cmds_after_conversion: Arc<Mutex<VecDeque<ADBCmdConv>>>,
    //命令发送通道
    pub cmd_sender: crossbeam_channel::Sender<ADBCommand>,
    //循环命令发送通道
    pub cmd_loop_sender: crossbeam_channel::Sender<ADBCommand>,
    //截图命令发送通道（高优先级，避免被Loop热路径拖慢）
    pub capture_sender: crossbeam_channel::Sender<ADBCommand>,
}

impl ADBCtx {
    pub async fn new(runtime_connect_conf: ADBConnectConfig) -> Result<(), String> {
        if let Some(adb_ctx) = ADB_CONTEXT.get() {
            adb_ctx
                .send_adb_cmd_await(ADBCommand::ChangeConnectConfig(runtime_connect_conf))
                .await?;
            return Ok(());
        }

        let initial_config = runtime_connect_conf.clone();
        let adb_config = Arc::new(Mutex::new(runtime_connect_conf));
        let (err_tx, err_rx) = crossbeam_channel::bounded(5);
        let (executor, cmd_sender, cmd_loop_sender, capture_sender) =
            //ADBExecutor::new(Arc::new(Mutex::new(runtime_connect_conf)),core_id, err_tx);
            ADBExecutor::new(adb_config.clone(), err_tx);
        let cmds_after_conversion = executor.cmds_after_conversion.clone();

        let adb_ctx = ADBCtx {
            adb_config,
            cmds_after_conversion,
            cmd_sender,
            cmd_loop_sender,
            capture_sender,
        };

        if ADB_CONTEXT.set(adb_ctx).is_err() {
            let adb_ctx = ADB_CONTEXT
                .get()
                .ok_or_else(|| "[ ADBCtx ] ADB上下文初始化竞争后丢失".to_string())?;
            adb_ctx
                .send_adb_cmd_await(ADBCommand::ChangeConnectConfig(initial_config))
                .await?;
            return Ok(());
        }

        let runtime_handle = tokio::runtime::Handle::current();
        std::thread::spawn(move || {
            runtime_handle.block_on(executor.run());
        });
        std::thread::spawn(move || {
            while let Ok(cmd) = err_rx.recv() {
                Log::error(format!("[ ADBCtx ] ADB执行错误:{:?}", cmd).as_str());
            }
        });

        let adb_ctx = ADB_CONTEXT
            .get()
            .ok_or_else(|| "[ ADBCtx ] ADB上下文初始化后读取失败".to_string())?;
        adb_ctx
            .send_adb_cmd_await(ADBCommand::ChangeConnectConfig(initial_config))
            .await
    }

    pub fn validate_config(&self) -> bool {
        tokio::runtime::Handle::current()
            .block_on(async { self.adb_config.clone().lock().await.valid() })
    }

    pub fn send_adb_cmd(&self, adb_command: &ADBCommand) -> Result<(), String> {
        self.cmd_sender
            .send_timeout(
                adb_command.clone(),
                Duration::from_millis(ADB_COMMAND_SEND_TIMEOUT_MS),
            )
            .map_err(|error| {
                let message = match error {
                    crossbeam_channel::SendTimeoutError::Timeout(_) => format!(
                        "[ ADBCtx ] ADB命令队列繁忙，发送命令[{}]超时{}ms，可能存在卡住的设备操作",
                        adb_command, ADB_COMMAND_SEND_TIMEOUT_MS
                    ),
                    crossbeam_channel::SendTimeoutError::Disconnected(_) => {
                        format!(
                            "[ ADBCtx ] ADB命令通道已关闭，无法发送命令[{}]",
                            adb_command
                        )
                    }
                };
                Log::error(&message);
                message
            })
    }

    pub async fn send_adb_cmd_await(&self, adb_command: ADBCommand) -> Result<(), String> {
        let result_timeout_ms = default_command_result_timeout_ms(&adb_command);
        self.send_adb_cmd_await_timeout(adb_command, result_timeout_ms)
            .await
    }

    /// 这里只等待执行器回传命令结果，不负责设备启动或 EnsureReady 之类的连接准备超时。
    pub async fn send_adb_cmd_await_timeout(
        &self,
        adb_command: ADBCommand,
        result_timeout_ms: u64,
    ) -> Result<(), String> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.send_adb_cmd(&ADBCommand::AwaitResult(Box::new(adb_command), tx))?;

        tokio::task::spawn_blocking(move || {
            rx.recv_timeout(Duration::from_millis(result_timeout_ms))
        })
        .await
        .map_err(|error| format!("[ ADBCtx ] 等待ADB命令执行结果任务异常: {}", error))?
        .map_err(|_| {
            format!(
                "[ ADBCtx ] 等待ADB命令执行结果超时{}ms，可能存在卡住的设备操作",
                result_timeout_ms
            )
        })?
    }

    pub fn send_adb_loop_cmd(&self, adb_command: &ADBCommand) -> Result<(), String> {
        self.cmd_loop_sender
            .send_timeout(
                adb_command.clone(),
                Duration::from_millis(ADB_COMMAND_SEND_TIMEOUT_MS),
            )
            .map_err(|error| {
                let message = match error {
                    crossbeam_channel::SendTimeoutError::Timeout(_) => format!(
                        "[ ADBCtx ] ADB循环命令队列繁忙，发送命令[{}]超时，可能存在卡住的设备操作",
                        adb_command
                    ),
                    crossbeam_channel::SendTimeoutError::Disconnected(_) => {
                        format!(
                            "[ ADBCtx ] ADB循环命令通道已关闭，无法发送命令[{}]",
                            adb_command
                        )
                    }
                };
                Log::error(&message);
                message
            })
    }

    pub fn send_capture_cmd(&self, adb_command: &ADBCommand) -> Result<(), String> {
        self.capture_sender
            .send_timeout(
                adb_command.clone(),
                Duration::from_millis(ADB_COMMAND_SEND_TIMEOUT_MS),
            )
            .map_err(|error| {
                let message = match error {
                    crossbeam_channel::SendTimeoutError::Timeout(_) => format!(
                        "[ ADBCtx ] ADB截图命令队列繁忙，发送命令[{}]超时{}ms，可能存在卡住的设备截图操作",
                        adb_command,
                        ADB_COMMAND_SEND_TIMEOUT_MS
                    ),
                    crossbeam_channel::SendTimeoutError::Disconnected(_) => {
                        format!("[ ADBCtx ] ADB截图命令通道已关闭，无法发送命令[{}]", adb_command)
                    }
                };
                Log::error(&message);
                message
            })
    }
}
