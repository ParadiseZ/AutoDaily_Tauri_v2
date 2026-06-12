use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_executor::ADBExecutor;
use crate::infrastructure::logging::log_trait::Log;
//use core_affinity::CoreId;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::Mutex;

static ADB_CONTEXT: OnceLock<ADBCtx> = OnceLock::new();
const ADB_COMMAND_SEND_TIMEOUT_MS: u64 = 100;

pub fn get_adb_ctx() -> &'static ADBCtx {
    ADB_CONTEXT.get().unwrap()
}

impl std::fmt::Display for ADBCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        tokio::runtime::Handle::current().block_on(async move {
            write!(
                f,
                "ADB_CONTEXT config:{:?},cmd_channel:{},cmd_loop_channel:{},cmd_cur_queue:{}",
                self.adb_executor.adb_config.clone().lock().await,
                self.cmd_sender.len(),
                self.cmd_loop_sender.len(),
                self.adb_executor
                    .cmds_after_conversion
                    .clone()
                    .lock()
                    .await
                    .iter()
                    .len()
            )
        })
    }
}

pub struct ADBCtx {
    pub adb_executor: ADBExecutor,
    //命令发送通道
    pub cmd_sender: crossbeam_channel::Sender<ADBCommand>,
    //循环命令发送通道
    pub cmd_loop_sender: crossbeam_channel::Sender<ADBCommand>,
}

impl ADBCtx {
    pub async fn new(runtime_connect_conf: ADBConnectConfig) {
        if let Some(adb_ctx) = ADB_CONTEXT.get() {
            let _ = adb_ctx.send_adb_cmd(&ADBCommand::ChangeConnectConfig(runtime_connect_conf));
            return;
        }
        let (err_tx, err_rx) = crossbeam_channel::bounded(5);
        let (executor, cmd_sender,cmd_loop_sender) =
            //ADBExecutor::new(Arc::new(Mutex::new(runtime_connect_conf)),core_id, err_tx);
            ADBExecutor::new(Arc::new(Mutex::new(runtime_connect_conf)), err_tx);
        tokio::spawn(async move {
            while let Ok(cmd) = err_rx.recv() {
                Log::error(format!("ADB执行错误:{:?}", cmd).as_str())
            }
        });
        ADB_CONTEXT.get_or_init(|| ADBCtx {
            adb_executor: executor,
            cmd_sender,
            cmd_loop_sender,
        });
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
                        "ADB命令队列繁忙，发送命令[{}]超时，可能存在卡住的设备操作",
                        adb_command
                    ),
                    crossbeam_channel::SendTimeoutError::Disconnected(_) => {
                        format!("ADB命令通道已关闭，无法发送命令[{}]", adb_command)
                    }
                };
                Log::error(&message);
                message
            })
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
                        "ADB循环命令队列繁忙，发送命令[{}]超时，可能存在卡住的设备操作",
                        adb_command
                    ),
                    crossbeam_channel::SendTimeoutError::Disconnected(_) => {
                        format!("ADB循环命令通道已关闭，无法发送命令[{}]", adb_command)
                    }
                };
                Log::error(&message);
                message
            })
    }
}
