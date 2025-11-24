use crate::command::ADBCommand;
use crate::config::ADBConnectConfig;
use crate::executor::ADBExecutor;
//use core_affinity::CoreId;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

pub struct ADBCtx {
    pub adb_executor: Arc<ADBExecutor>,
    //命令发送通道
    pub cmd_sender: crossbeam_channel::Sender<ADBCommand>,
    //循环命令发送通道
    pub cmd_loop_sender: crossbeam_channel::Sender<ADBCommand>,
}

impl ADBCtx {
    pub fn new(adb_connect_conf: ADBConnectConfig) -> Self {
        let (err_tx, err_rx) = crossbeam_channel::bounded(5);
        let (executor, cmd_sender,cmd_loop_sender) =
            //ADBExecutor::new(Arc::new(Mutex::new(adb_connect_conf)),core_id, err_tx);
            ADBExecutor::new(Arc::new(Mutex::new(adb_connect_conf)), err_tx);
        tokio::spawn(async move {
            while let Ok(cmd) = err_rx.recv() {
                error!("ADB执行错误:{:?}", cmd)
            }
        });
        ADBCtx {
            adb_executor: Arc::new(executor),
            cmd_sender,
            cmd_loop_sender,
        }
    }

    pub fn send_adb_cmd(&self, adb_command: &ADBCommand) {
        if let Err(e) = self.cmd_sender.send(adb_command.clone()) {
            error!("发送ADB命令[{:?}]失败:{:?}", adb_command, e);
        };
    }

    pub fn send_adb_loop_cmd(&self, adb_command: &ADBCommand) {
        if let Err(e) = self.cmd_loop_sender.send(adb_command.clone()) {
            error!("发送ADB循环命令[{:?}]失败:{:?}", adb_command, e);
        };
    }
}
