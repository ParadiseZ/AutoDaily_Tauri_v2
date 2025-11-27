use crate::infrastructure::adb_cli_local::adb_command::ADBCommand;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_executor::ADBExecutor;
use crate::infrastructure::logging::log_trait::Log;
use std::cell::OnceCell;
//use core_affinity::CoreId;
use std::sync::Arc;
use tokio::sync::Mutex;

static ADB_CONTEXT: OnceCell<ADBCtx> = OnceCell::new();

pub fn get_adb_ctx() -> &'static ADBCtx {
    ADB_CONTEXT.get().unwrap()
}

impl std::fmt::Display for ADBCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        tokio::runtime::Handle::current().block_on(async move {
            write!(f,
                   "ADB_CONTEXT config:{:?},cmd_channel:{},cmd_loop_channel:{},cmd_cur_queue:{:?}",
                   self.adb_executor.adb_config.clone().lock().await,
                   self.cmd_sender.len(),
                   self.cmd_loop_sender.len(),
                   self.adb_executor.cmds_after_conversion.clone().lock().await
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
    pub async fn new(adb_connect_conf: ADBConnectConfig){
        if let Some(adb_ctx) = ADB_CONTEXT.get() {
            adb_ctx.send_adb_cmd(&ADBCommand::ChangeConnectConfig(adb_connect_conf));
            return;
        }
        let (err_tx, err_rx) = crossbeam_channel::bounded(5);
        let (executor, cmd_sender,cmd_loop_sender) =
            //ADBExecutor::new(Arc::new(Mutex::new(adb_connect_conf)),core_id, err_tx);
            ADBExecutor::new(Arc::new(Mutex::new(adb_connect_conf)), err_tx);
        tokio::spawn(async move {
            while let Ok(cmd) = err_rx.recv() {
                Log::error(format!("ADB执行错误:{:?}", cmd).as_str())
            }
        });
        ADB_CONTEXT.get_or_init( ||
            ADBCtx {
                adb_executor: executor,
                cmd_sender,
                cmd_loop_sender,
            }
        );
    }

    pub fn send_adb_cmd(&self, adb_command: &ADBCommand) {
        if let Err(e) = self.cmd_sender.send(adb_command.clone()) {
            Log::error(format!("发送ADB命令[{:?}]失败:{:?}", adb_command, e).as_str());
        };
    }

    pub fn send_adb_loop_cmd(&self, adb_command: &ADBCommand) {
        if let Err(e) = self.cmd_loop_sender.send(adb_command.clone()) {
            Log::error(format!("发送ADB循环命令[{:?}]失败:{:?}", adb_command, e).as_str());
        };
    }
}
