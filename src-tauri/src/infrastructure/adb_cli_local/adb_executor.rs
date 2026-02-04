use crate::infrastructure::adb_cli_local::adb_command::{click_cmd, input_text_cmd, long_click_and_swipe, long_click_cmd, sleep_cmd, stop_app_cmd, swipe_cmd, swipe_duration_cmd, ADBCmdConv, ADBCommand, ADBCommandResult, BACK, HOME};

use adb_client::{ADBDeviceExt, RebootType};
use crossbeam_channel::bounded;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use std::time::Duration;
use adb_client::server::ADBServer;
use adb_client::tcp::ADBTcpDevice;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_error::{AdbError, AdbResult};
use crate::infrastructure::logging::log_trait::Log;

pub struct ADBExecutor {
    device: Option<Box<dyn ADBDeviceExt + Send + Sync>>,
    pub adb_config: Arc<Mutex<ADBConnectConfig>>, //from RuntimeContext

    cmd_rx: crossbeam_channel::Receiver<ADBCommand>,
    cmd_loop_rx: crossbeam_channel::Receiver<ADBCommand>,
    error_tx: crossbeam_channel::Sender<ADBCommandResult>,

    //core : CoreId,
    executor_is_looping: bool,
    //paused: Arc<AtomicBool>,
    pub cmds_after_conversion: Arc<Mutex<VecDeque<ADBCmdConv>>>,
    duration: Duration,
    need_duration: Arc<AtomicBool>,
}

impl std::fmt::Debug for ADBExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"device_connected:{},executor_is_looping:{}",self.device.is_some(),self.executor_is_looping)
    }
}
impl ADBExecutor {
    pub fn new(
        adb_config: Arc<Mutex<ADBConnectConfig>>,
        //core: CoreId,
        error_tx: crossbeam_channel::Sender<ADBCommandResult>,
    ) -> (
        Self,
        crossbeam_channel::Sender<ADBCommand>,
        crossbeam_channel::Sender<ADBCommand>,
    ) {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (cmd_loop_tx, cmd_loop_rx) = bounded(10);

        (
            ADBExecutor {
                device: None,
                adb_config,
                cmd_rx,
                cmd_loop_rx,
                error_tx,
                //core,
                executor_is_looping: false,
                //paused : Arc::new(AtomicBool::new(true)),
                cmds_after_conversion: Arc::new(Mutex::new(VecDeque::new())),
                duration: Duration::from_millis(300),
                need_duration: Arc::new(AtomicBool::new(false)),
            },
            cmd_tx,
            cmd_loop_tx,
        )
    }
    pub fn clone_for_child(&self) -> Self {
        Self {
            device: None,
            adb_config: self.adb_config.clone(),
            cmd_rx: self.cmd_rx.clone(),
            cmd_loop_rx: self.cmd_loop_rx.clone(),
            error_tx: self.error_tx.clone(),
            executor_is_looping: false,
            cmds_after_conversion: self.cmds_after_conversion.clone(),
            duration: self.duration,
            need_duration: self.need_duration.clone(),
        }
    }
    pub fn validate_config(&self) -> bool {
        tokio::runtime::Handle::current().block_on(async {
            self.adb_config.clone().lock().await.valid()
        })
    }


    pub async fn run(mut self) {
        let (tx, rx) = bounded(1);

        loop {
            crossbeam_channel::select! {
                recv(self.cmd_rx) ->msg => {
                    if let Ok(cmd) = msg.as_ref() {
                        // 非循环状态下直接执行
                        self.low_speed_cmd(cmd).await;
                    }
                },
                recv(self.cmd_loop_rx) ->msg => {
                    if let Ok(cmd_high) = msg.as_ref() {
                        // 收到循环命令，先清空之前的转换队列
                        self.cmds_after_conversion.lock().await.clear();
                        
                        // 命令中睡眠命令和其他命令数量不一致，则启动默认睡眠
                        if !self.high_speed_cmd(cmd_high).await{
                            self.need_duration.store(true, Ordering::Release);
                        }
                        
                        // 如果当前没有在循环，且转换后队列为空（可能是空Loop命令？），发送启动信号可能没意义，但原逻辑似乎是想触发循环
                        // 原逻辑：if !self.executor_is_looping.load(Ordering::Acquire) && self.cmds_after_conversion.lock().await.is_empty()
                        // 这里简化逻辑：只要收到Loop命令，就尝试启动循环逻辑（通过发送到rx）
                        
                        if let Err(_) = tx.send(true){
                            self.error_tx.send(ADBCommandResult::Failed("发送启动循环信号失败！".to_string())).unwrap();
                        }
                    }
                },
                recv(rx) ->msg => {
                    if let Ok(_) = msg.as_ref() {
                        self.executor_is_looping = true;

                        while self.executor_is_looping {
                            let cmds : VecDeque<ADBCmdConv>= {
                                self.cmds_after_conversion.lock().await.clone()
                            };
                            for cmd in cmds.iter() {
                                // 检查是否停止循环
                                if !self.executor_is_looping {
                                    break;
                                }

                                if let Err(e) = self.execute_single(cmd).await{
                                    self.error_tx.send(ADBCommandResult::Failed(format!("执行操作{}失败：{}", cmd,e))).unwrap();
                                }

                                // 检查低速命令插队
                                if let Ok( cmd_low) = self.cmd_rx.try_recv().as_ref(){
                                    self.low_speed_cmd(cmd_low).await
                                }

                                // 检查新的循环命令（更新循环内容）
                                if let Ok( cmd_high) = self.cmd_loop_rx.try_recv().as_ref(){
                                    // 命令中睡眠命令和其他命令梳理不一致，则启动默认睡眠
                                    if !self.high_speed_cmd(cmd_high).await{
                                        self.need_duration.store(true, Ordering::Release);
                                    }
                                }
                            }
                            // 默认睡眠
                            if self.executor_is_looping && self.need_duration.load(Ordering::Acquire){
                                sleep(self.duration).await;
                            }
                        }
                    }
                },
            }
        }
    }

    async fn high_speed_cmd(&mut self, cmd_high: &ADBCommand) -> bool {
        let mut sleep_count = 0;
        let mut not_sleep_count = 0;
        if let ADBCommand::Loop(commands) = cmd_high {
            //先全部转换
            let mut new_cmds = VecDeque::new();
            for cmd in commands.iter() {
                if Self::translate_cmd(cmd, &mut new_cmds) {
                    sleep_count += 1;
                } else {
                    not_sleep_count += 1;
                }
            }

            //获取写锁并更新队列（带去重）
            let mut queue = self.cmds_after_conversion.lock().await;

            //将shell类型的命令筛选出来
            let mut seen: HashSet<String> = queue
                .iter()
                .filter_map(|c| match c {
                    ADBCmdConv::ADBShellCommand(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();

            // 只添加不重复的shell命令，client、睡眠命令总是添加
            for cmd in new_cmds {
                match &cmd {
                    ADBCmdConv::ADBShellCommand(s) => {
                        if seen.insert(s.clone()) {
                            queue.push_back(cmd);
                        }
                    }
                    ADBCmdConv::ADBClientCommand(_) | ADBCmdConv::ADBSleepCommand(_) => {
                        queue.push_back(cmd);
                    }
                }
            }
        }
        if sleep_count == not_sleep_count {
            true
        } else {
            false
        }
    }

    async fn low_speed_cmd(&mut self, cmd_low: &ADBCommand) {
        match cmd_low {
            ADBCommand::Loop { .. } => {}
            ADBCommand::Duration(_) => {}
            ADBCommand::StopLoop(is_stop) => {
                if *is_stop {
                    self.executor_is_looping = false;
                }
            }
            ADBCommand::Pause => {}
            ADBCommand::Resume => {}
            _ => {
                if let Err(e) = self.execute_adb_command(cmd_low).await {
                    let _ = self.error_tx
                        .send(ADBCommandResult::Failed(format!(
                            "执行操作{}失败：{}",
                            cmd_low, e
                        )));
                }
            }
        }
    }

    /// 转换循环命令
    /// 返回是否是休眠命令，以供上层high_speed_cmd计数，最终以此判断是否启用默认休眠
    /// 注意： 有忽略其他命令的行为，新增时记得是否更新代码
    fn translate_cmd(cmd: &ADBCommand, cmds_str: &mut VecDeque<ADBCmdConv>) -> bool {
        match cmd {
            ADBCommand::Reboot => cmds_str.push_back(ADBCmdConv::ADBClientCommand(cmd.clone())),
            ADBCommand::StartActivity(_, _) => {
                cmds_str.push_back(ADBCmdConv::ADBClientCommand(cmd.clone()))
            }
            //忽略截图命令，此处为发过来的循环命令
            ADBCommand::Capture(_) => {}

            ADBCommand::Duration(_) => {
                cmds_str.push_back(ADBCmdConv::ADBSleepCommand(cmd.clone()));
                return true;
            }

            ADBCommand::Click(point) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(click_cmd(point).to_string()))
            }
            ADBCommand::LongClick(point) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(long_click_cmd(point).to_string()))
            }
            ADBCommand::LongClickAndSwipe(point1, point2, duration) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(
                    long_click_and_swipe(point1, point2, duration).to_string(),
                ))
            }
            ADBCommand::Swipe(point1, point2) => cmds_str.push_back(ADBCmdConv::ADBShellCommand(
                swipe_cmd(point1, point2).to_string(),
            )),
            ADBCommand::SwipeWithDuration(point1, point2, duration) => cmds_str.push_back(ADBCmdConv::ADBShellCommand(
                swipe_duration_cmd(point1, point2, duration).to_string(),
            )),
            ADBCommand::StopApp(pkg_name) => cmds_str.push_back(ADBCmdConv::ADBShellCommand(
                stop_app_cmd(&pkg_name).to_string(),
            )),
            ADBCommand::InputText(text) => cmds_str.push_back(ADBCmdConv::ADBShellCommand(
                input_text_cmd(&text).to_string(),
            )),
            ADBCommand::Back => cmds_str.push_back(ADBCmdConv::ADBShellCommand(BACK.to_string())),
            ADBCommand::Home => cmds_str.push_back(ADBCmdConv::ADBShellCommand(HOME.to_string())),
            ADBCommand::Sequence(cmds) => {
                let cmd_string = Self::translate_sequence_cmd(cmds);
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(cmd_string))
            }
            // 忽略其他命令
            _ => {}
        }
        false
    }

    /// 转换命令序列为单条命令
    /// 注意： 有忽略其他命令的行为，新增时记得是否更新代码
    fn translate_sequence_cmd(cmds: &[ADBCommand])-> String {
        let mut cmd_string = String::new();
        for sub_cmd in cmds.iter() {
            match sub_cmd {
                ADBCommand::Click(point) => {
                    cmd_string.push_str(&click_cmd(point));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::LongClick(point) => {
                    cmd_string.push_str(&long_click_cmd(point));
                    cmd_string.push_str(" &&");
                }

                ADBCommand::LongClickAndSwipe(point1, point2, duration) => {
                    cmd_string.push_str(&long_click_and_swipe(point1, point2, duration));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Swipe(point1, point2) => {
                    cmd_string.push_str(&swipe_cmd(point1, point2));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::SwipeWithDuration(point1, point2,duration) => {
                    cmd_string.push_str(&swipe_duration_cmd(point1, point2, duration));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::StopApp(pkg_name) => {
                    cmd_string.push_str(&stop_app_cmd(pkg_name));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::InputText(text) => {
                    cmd_string.push_str(&input_text_cmd(text));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Back => {
                    cmd_string.push_str(&BACK);
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Home => {
                    cmd_string.push_str(&HOME);
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Duration(interval) => {
                    cmd_string.push_str(&sleep_cmd(*interval));
                    cmd_string.push_str(" &&");
                }
                _ => {}
            }
        }
        if cmd_string.ends_with("&&") {
            cmd_string.truncate(cmd_string.len() - 2)
        }
        Log::debug(format!("命令队列：{}", cmd_string.as_str()).as_str());
        cmd_string
    }

    /// 循环中执行转换后的命令
    async fn execute_single(&mut self, cmd: &ADBCmdConv) -> AdbResult<ADBCommandResult> {
        match cmd {
            ADBCmdConv::ADBClientCommand(cmd) => {
                if let Err(_) = self.execute_adb_command(&cmd).await {
                    // 自动重连
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    self.reconnect().await;
                    // 重试一次
                    let _ = self.execute_adb_command(&cmd).await;
                }
            }
            ADBCmdConv::ADBShellCommand(cmd) => {
                if let Err(_) = self.execute_shell(cmd).await {
                    // 自动重连
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    self.reconnect().await;
                    // 重试一次
                    let _ = self.execute_shell(cmd).await;
                };
            }
            ADBCmdConv::ADBSleepCommand(ADBCommand::Duration(interval)) => {
                tokio::time::sleep(Duration::from_secs(*interval)).await;
            }
            _ => { /* Ok(ADBCommandResult::Success) */ }
        }
        Ok(ADBCommandResult::Success)
    }

    /// 最终执行调用的函数 -> 调用三方包执行命令
    async fn execute_adb_command(&mut self, cmd: &ADBCommand) -> AdbResult<ADBCommandResult> {
        match cmd {
            ADBCommand::Reboot => {
                if let Some(device) = self.device.as_mut(){
                    let res = device
                        .reboot(RebootType::System)
                        .map(|_| ADBCommandResult::Success)
                        .unwrap_or_else(|e| ADBCommandResult::Failed(e.to_string()));
                    return Ok(res);
                }
                Err(AdbError::ConfigErr {
                    detail: "Device not connected".to_string(),
                })
            }
            ADBCommand::Click(point) => {
                let _ = self.execute_shell(&click_cmd(point)).await;
                Ok(ADBCommandResult::Success)
            }

            ADBCommand::LongClick( point)=>{
                self.execute_shell(&long_click_cmd(point)).await
            }

            ADBCommand::LongClickAndSwipe(point1, point2, duration) => {
                self.execute_shell(&long_click_and_swipe(point1, point2, duration))
                    .await
            }

            ADBCommand::Swipe(point1, point2) => {
                self.execute_shell(&swipe_cmd(point1, point2)).await
            }
            ADBCommand::SwipeWithDuration(point1, point2, duration) => {
                self.execute_shell(&swipe_duration_cmd(point1, point2,duration)).await
            }

            ADBCommand::StartActivity(package_name, activity_name) => {
                if let Some(device) =  self.device.as_mut(){
                    let res = device
                        .run_activity(package_name, activity_name)
                        .map(|_| ADBCommandResult::Success)
                        .unwrap_or_else(|e| ADBCommandResult::Failed(e.to_string()));
                    Ok(res)
                }else {
                    Err(AdbError::ConfigErr {
                        detail: "Device not connected".to_string(),
                    })
                }
            }

            ADBCommand::Capture(sender) => {
                if let Some(device) =  self.device.as_mut(){
                    if let Ok(data) = device.framebuffer_inner() {
                        if let Ok(_) = sender.send(data) {
                            Ok(ADBCommandResult::Success)
                        } else {
                            Err(AdbError::SendPictureFailed)
                        }
                    } else {
                        Err(AdbError::ExecuteShellFailed {
                            cmd: "framebuffer_inner".to_string(),
                            e: "".to_string(),
                        })
                    }
                }else {
                    Err(AdbError::ConfigErr {
                        detail: "Device not connected".to_string(),
                    })
                }
            }

            ADBCommand::StopApp(pkg_name) => self.execute_shell(&stop_app_cmd(&pkg_name)).await,

            ADBCommand::InputText(text) => self.execute_shell(&input_text_cmd(&text)).await,

            ADBCommand::Back => self.execute_shell(BACK).await,

            ADBCommand::Home => self.execute_shell(HOME).await,

            ADBCommand::Sequence(cmds) => {
                let cmd_string = Self::translate_sequence_cmd(cmds);
                self.execute_shell(&cmd_string).await
            }
            //此类命令在此不执行
            ADBCommand::Loop { .. } | ADBCommand::StopLoop(_) => Ok(ADBCommandResult::Success),

            ADBCommand::Duration(interval) => {
                tokio::time::sleep(Duration::from_millis(*interval)).await;
                Ok(ADBCommandResult::Success)
            }
            //此类命令在此不执行
            ADBCommand::Pause | ADBCommand::Resume => Ok(ADBCommandResult::Success),

            ADBCommand::ChangeConnectConfig( config)=>{
                {
                    self.device = None;
                    let mut old_conf = self.adb_config.lock().await;
                    *old_conf = config.clone();
                }
                self.reconnect().await;
                Ok(ADBCommandResult::Success)
            }
        }
    }

    /// 封装的三方的shell命令
    async fn execute_shell(&mut self, cmd: &str) -> AdbResult<ADBCommandResult> {
        //let mut buffer = Vec::new();
        if let Some(device) =  self.device.as_mut(){
            let res = device
                .shell_command(&cmd,None,None)
                .map(|_| ADBCommandResult::Success)
                .unwrap_or_else(|e| ADBCommandResult::Failed(e.to_string()));
            //let _ = self.resp_tx.send(res);
            Ok(res)
        } else {
            Err(AdbError::ConnectFailed {
                ipv4: "unknown".to_string(),
                e: "".to_string(),
            })
        }
    }

    async fn reconnect(&mut self) {
        self.try_to_connect().await;
    }

    /// 尝试重连
    async fn try_to_connect(&mut self) -> bool {
        //self.device = None;
        let cfg = self.adb_config.lock().await;
        let device: Option<Box<dyn ADBDeviceExt + Send + Sync>> = match &*cfg {
            ADBConnectConfig::ServerConnectByName(dev) => {
                // 检查服务器连接地址是否配置
                if !dev.valid() {
                    None
                } else {
                    let device = ADBServer::new_from_path(
                        dev.adb_config.server_connect.unwrap(),
                        dev.adb_config.adb_path.clone(),
                    )
                        .get_device_by_name(dev.device_name.as_ref().unwrap().as_str());
                    if let Ok(device) = device {
                        Some(Box::new(device))
                    } else {
                        None
                    }
                }
            }
            ADBConnectConfig::ServerConnectByIp(dev) => {
                // 检查服务器连接地址是否配置
                if !dev.valid() {
                    None
                } else {
                    // 初始化 ADB 服务器+连接到设备
                    let mut adb_server = ADBServer::new_from_path(
                        dev.adb_config.server_connect.unwrap(),
                        dev.adb_config.adb_path.clone(),
                    );
                    //连接设备
                    let server = adb_server.connect_device(dev.client_connect.unwrap());
                    if let Ok(_) = server {
                        let device =
                            adb_server.get_device_by_name(&dev.client_connect.unwrap().to_string());
                        if let Ok(device) = device {
                            Some(Box::new(device))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
            ADBConnectConfig::DirectTcp(dev) => {
                // 检查设备连接地址是否配置
                if dev.is_none() {
                    None
                } else {
                    if let Ok(device) = ADBTcpDevice::new(SocketAddr::V4(dev.unwrap())) {
                        Some(Box::new(device))
                    } else {
                        None
                    }
                }
            }
            ADBConnectConfig::DirectUsb(_) => None,
        };
        self.device =  device;
        if self.device.is_some(){
            true
        }else { false }
    }
}
