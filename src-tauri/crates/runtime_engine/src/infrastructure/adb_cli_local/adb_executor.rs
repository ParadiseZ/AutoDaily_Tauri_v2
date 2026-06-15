use crate::infrastructure::adb_cli_local::adb_command::{
    click_cmd, input_text_cmd, long_click_and_swipe, long_click_cmd, sleep_cmd, stop_app_cmd,
    swipe_cmd, swipe_duration_cmd, ADBCmdConv, ADBCommand, ADBCommandResult, BACK, HOME,
};
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::adb_cli_local::adb_error::{AdbError, AdbResult};
use crate::infrastructure::logging::log_trait::Log;
use adb_client::server::ADBServer;
use adb_client::tcp::ADBTcpDevice;
use adb_client::{ADBDeviceExt, RebootType};
use crossbeam_channel::bounded;
use image::RgbaImage;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct ADBExecutor {
    device: Option<Box<dyn ADBDeviceExt + Send + Sync>>,
    pub adb_config: Arc<Mutex<ADBConnectConfig>>,
    cmd_rx: crossbeam_channel::Receiver<ADBCommand>,
    cmd_loop_rx: crossbeam_channel::Receiver<ADBCommand>,
    capture_rx: crossbeam_channel::Receiver<ADBCommand>,
    error_tx: crossbeam_channel::Sender<ADBCommandResult>,
    executor_is_looping: bool,
    pub cmds_after_conversion: Arc<Mutex<VecDeque<ADBCmdConv>>>,
    duration: Duration,
    need_duration: Arc<AtomicBool>,
}

impl std::fmt::Debug for ADBExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "device_connected:{},executor_is_looping:{}",
            self.device.is_some(),
            self.executor_is_looping
        )
    }
}

impl ADBExecutor {
    pub fn new(
        adb_config: Arc<Mutex<ADBConnectConfig>>,
        error_tx: crossbeam_channel::Sender<ADBCommandResult>,
    ) -> (
        Self,
        crossbeam_channel::Sender<ADBCommand>,
        crossbeam_channel::Sender<ADBCommand>,
        crossbeam_channel::Sender<ADBCommand>,
    ) {
        let (cmd_tx, cmd_rx) = bounded(3);
        let (cmd_loop_tx, cmd_loop_rx) = bounded(10);
        let (capture_tx, capture_rx) = bounded(2);
        let cmds_after_conversion = Arc::new(Mutex::new(VecDeque::new()));

        (
            Self {
                device: None,
                adb_config,
                cmd_rx,
                cmd_loop_rx,
                capture_rx,
                error_tx,
                executor_is_looping: false,
                cmds_after_conversion,
                duration: Duration::from_millis(300),
                need_duration: Arc::new(AtomicBool::new(false)),
            },
            cmd_tx,
            cmd_loop_tx,
            capture_tx,
        )
    }

    pub fn validate_config(&self) -> bool {
        tokio::runtime::Handle::current()
            .block_on(async { self.adb_config.clone().lock().await.valid() })
    }

    pub async fn run(mut self) {
        loop {
            if self.executor_is_looping {
                self.run_loop_cycle().await;
                continue;
            }

            crossbeam_channel::select! {
                recv(self.capture_rx) -> msg => {
                    if let Ok(cmd_capture) = msg {
                        self.capture_cmd(&cmd_capture).await;
                    }
                },
                recv(self.cmd_rx) -> msg => {
                    if let Ok(cmd) = msg {
                        self.low_speed_cmd(&cmd).await;
                    }
                },
                recv(self.cmd_loop_rx) -> msg => {
                    if let Ok(cmd_high) = msg {
                        self.replace_loop_commands(&cmd_high).await;
                    }
                },
            }
        }
    }

    async fn run_loop_cycle(&mut self) {
        let cmds = { self.cmds_after_conversion.lock().await.clone() };
        if cmds.is_empty() {
            self.executor_is_looping = false;
            return;
        }

        for cmd in cmds.iter() {
            self.flush_capture_queue().await;
            self.process_pending_low_speed().await;
            if !self.executor_is_looping {
                break;
            }

            if let Err(error) = self.execute_single(cmd).await {
                let _ = self.error_tx.send(ADBCommandResult::Failed(format!(
                    "[ ADBExecutor ] 执行循环操作[{}]失败: {}",
                    cmd, error
                )));
            }

            self.flush_capture_queue().await;
            self.process_pending_low_speed().await;
            self.refresh_loop_commands_from_queue().await;
            if !self.executor_is_looping {
                break;
            }
        }

        if self.executor_is_looping && self.need_duration.load(Ordering::Acquire) {
            sleep(self.duration).await;
        }
    }

    async fn flush_capture_queue(&mut self) {
        while let Ok(cmd_capture) = self.capture_rx.try_recv() {
            self.capture_cmd(&cmd_capture).await;
        }
    }

    async fn process_pending_low_speed(&mut self) {
        while let Ok(cmd_low) = self.cmd_rx.try_recv() {
            self.low_speed_cmd(&cmd_low).await;
            if !self.executor_is_looping {
                break;
            }
        }
    }

    async fn refresh_loop_commands_from_queue(&mut self) {
        let mut latest = None;
        while let Ok(cmd_high) = self.cmd_loop_rx.try_recv() {
            latest = Some(cmd_high);
        }

        if let Some(cmd_high) = latest {
            self.replace_loop_commands(&cmd_high).await;
        }
    }

    async fn replace_loop_commands(&mut self, cmd_high: &ADBCommand) {
        let uses_explicit_sleep = self.high_speed_cmd(cmd_high).await;
        self.need_duration
            .store(!uses_explicit_sleep, Ordering::Release);

        let has_commands = !self.cmds_after_conversion.lock().await.is_empty();
        self.executor_is_looping = has_commands;
    }

    async fn high_speed_cmd(&mut self, cmd_high: &ADBCommand) -> bool {
        let mut sleep_count = 0;
        let mut not_sleep_count = 0;
        let mut new_cmds = VecDeque::new();

        if let ADBCommand::Loop(commands) = cmd_high {
            for cmd in commands.iter() {
                if Self::translate_cmd(cmd, &mut new_cmds) {
                    sleep_count += 1;
                } else {
                    not_sleep_count += 1;
                }
            }
        } else {
            Log::warn(&format!(
                "[ ADBExecutor ] 高频命令[{}]不支持，请将步骤设置在非序列步骤内",
                cmd_high
            ));
            return false;
        }

        let mut queue = self.cmds_after_conversion.lock().await;
        queue.clear();

        let mut seen: HashSet<String> = HashSet::new();
        for cmd in new_cmds {
            match &cmd {
                ADBCmdConv::ADBShellCommand(shell_cmd) => {
                    if seen.insert(shell_cmd.clone()) {
                        queue.push_back(cmd);
                    }
                }
                ADBCmdConv::ADBClientCommand(_) | ADBCmdConv::ADBSleepCommand(_) => {
                    queue.push_back(cmd);
                }
            }
        }

        sleep_count == not_sleep_count
    }

    async fn low_speed_cmd(&mut self, cmd_low: &ADBCommand) {
        match cmd_low {
            ADBCommand::StopLoop(is_stop) => {
                if *is_stop {
                    self.executor_is_looping = false;
                }
            }
            ADBCommand::Duration(_)
            | ADBCommand::Loop(_)
            | ADBCommand::Pause
            | ADBCommand::Resume => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 低频命令[{}]不支持，请将步骤设置在序列步骤内",
                    cmd_low
                ));
            }
            ADBCommand::AwaitResult(command, sender) => {
                let result = match self.execute_adb_command_with_retry(command.as_ref()).await {
                    Ok(ADBCommandResult::Success)
                    | Ok(ADBCommandResult::Output(_))
                    | Ok(ADBCommandResult::Image(_)) => Ok(()),
                    Ok(ADBCommandResult::Failed(message)) => {
                        Log::error(&format!("[ ADBExecutor ] ADB命令执行失败: {}", message));
                        Err(message)
                    }
                    Err(error) => {
                        Log::error(&format!("[ ADBExecutor ] ADB命令执行异常: {}", error));
                        Err(error.to_string())
                    }
                };
                let _ = sender.send(result);
            }
            _ => {
                if let Err(error) = self.execute_adb_command_with_retry(cmd_low).await {
                    let _ = self.error_tx.send(ADBCommandResult::Failed(format!(
                        "[ ADBExecutor ] 执行操作[{}]失败: {}",
                        cmd_low, error
                    )));
                }
            }
        }
    }

    async fn capture_cmd(&mut self, cmd_capture: &ADBCommand) {
        if let ADBCommand::Capture(sender) = cmd_capture {
            let result = self.capture_with_retry().await;
            let _ = sender.send(result);
            return;
        }

        let _ = self.error_tx.send(ADBCommandResult::Failed(format!(
            "[ ADBExecutor ] 无效的截图命令: {}",
            cmd_capture
        )));
    }

    fn translate_cmd(cmd: &ADBCommand, cmds_str: &mut VecDeque<ADBCmdConv>) -> bool {
        match cmd {
            ADBCommand::Reboot => cmds_str.push_back(ADBCmdConv::ADBClientCommand(cmd.clone())),
            ADBCommand::StartActivity(_, _) => {
                cmds_str.push_back(ADBCmdConv::ADBClientCommand(cmd.clone()))
            }
            ADBCommand::Capture(_) => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 循环命令中包含截图[{}]，该命令将被忽略，请将截图步骤设置在非序列步骤内",
                    cmd
                ));
            }
            ADBCommand::Duration(_) => {
                cmds_str.push_back(ADBCmdConv::ADBSleepCommand(cmd.clone()));
                return true;
            }
            ADBCommand::Click(point) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(click_cmd(point)))
            }
            ADBCommand::LongClick(point) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(long_click_cmd(point)))
            }
            ADBCommand::LongClickAndSwipe(point1, point2, duration) => cmds_str.push_back(
                ADBCmdConv::ADBShellCommand(long_click_and_swipe(point1, point2, duration)),
            ),
            ADBCommand::Swipe(point1, point2) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(swipe_cmd(point1, point2)))
            }
            ADBCommand::SwipeWithDuration(point1, point2, duration) => cmds_str.push_back(
                ADBCmdConv::ADBShellCommand(swipe_duration_cmd(point1, point2, duration)),
            ),
            ADBCommand::StopApp(pkg_name) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(stop_app_cmd(pkg_name)))
            }
            ADBCommand::InputText(text) => {
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(input_text_cmd(text)))
            }
            ADBCommand::Back => cmds_str.push_back(ADBCmdConv::ADBShellCommand(BACK.to_string())),
            ADBCommand::Home => cmds_str.push_back(ADBCmdConv::ADBShellCommand(HOME.to_string())),
            ADBCommand::Sequence(cmds) | ADBCommand::ReliableSequence(cmds) => {
                let cmd_string = Self::translate_sequence_cmd(cmds);
                cmds_str.push_back(ADBCmdConv::ADBShellCommand(cmd_string));
            }
            _ => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 循环命令中包含不支持的命令[{}]，该命令将被忽略，请将该步骤设置在非序列步骤内",
                    cmd
                ));
            }
        }
        false
    }

    fn translate_sequence_cmd(cmds: &[ADBCommand]) -> String {
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
                ADBCommand::SwipeWithDuration(point1, point2, duration) => {
                    cmd_string.push_str(&swipe_duration_cmd(point1, point2, duration));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::StopApp(pkg_name) => {
                    cmd_string.push_str(&stop_app_cmd(pkg_name));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::StartActivity(package_name, activity_name) => {
                    cmd_string.push_str(&format!("am start -n {}/{}", package_name, activity_name));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::InputText(text) => {
                    cmd_string.push_str(&input_text_cmd(text));
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Back => {
                    cmd_string.push_str(BACK);
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Home => {
                    cmd_string.push_str(HOME);
                    cmd_string.push_str(" &&");
                }
                ADBCommand::Duration(interval) => {
                    cmd_string.push_str(&sleep_cmd(*interval));
                    cmd_string.push_str(" &&");
                }
                _ => {
                    Log::warn("[ ADBExecutor ] 在序列命令中输入了不支持的shell命令");
                }
            }
        }
        if cmd_string.ends_with("&&") {
            cmd_string.truncate(cmd_string.len() - 2);
        }
        Log::debug(&format!("[ ADBExecutor ] 合并命令队列: {}", cmd_string));
        cmd_string
    }

    async fn execute_single(&mut self, cmd: &ADBCmdConv) -> AdbResult<ADBCommandResult> {
        match cmd {
            ADBCmdConv::ADBClientCommand(command) => {
                self.execute_adb_command_with_retry(command).await
            }
            ADBCmdConv::ADBShellCommand(shell_cmd) => {
                self.execute_shell_with_retry(shell_cmd).await
            }
            ADBCmdConv::ADBSleepCommand(ADBCommand::Duration(interval)) => {
                sleep(Duration::from_millis(*interval)).await;
                Ok(ADBCommandResult::Success)
            }
            _ => {
                Log::warn("[ ADBExecutor ] 转换后的循环指令包含了不支持的命令类型");
                Ok(ADBCommandResult::Success)
            }
        }
    }

    fn is_successful_result(result: &AdbResult<ADBCommandResult>) -> bool {
        matches!(
            result,
            Ok(ADBCommandResult::Success
                | ADBCommandResult::Output(_)
                | ADBCommandResult::Image(_))
        )
    }

    fn should_retry_after_failure(cmd: &ADBCommand) -> bool {
        if let ADBCommand::Capture(_) = cmd {
            Log::warn("[ ADBExecutor ] 错误的调用链路：截图指令应在单独的指令通道里处理");
            return false;
        }

        let should_retry = !matches!(
            cmd,
            ADBCommand::Loop(_)
                | ADBCommand::StopLoop(_)
                | ADBCommand::Duration(_)
                | ADBCommand::Pause
                | ADBCommand::Resume
                | ADBCommand::AwaitResult(_, _)
        );
        if !should_retry {
            Log::warn(&format!(
                "[ ADBExecutor ] ADB命令[{}]执行失败，且将不会重试",
                cmd
            ));
        }
        should_retry
    }

    fn describe_failure(result: &AdbResult<ADBCommandResult>) -> String {
        match result {
            Ok(ADBCommandResult::Failed(message)) => message.clone(),
            Err(error) => error.to_string(),
            Ok(other) => format!("{:?}", other),
        }
    }

    fn retry_delay_for(cmd: &ADBCommand) -> Duration {
        match cmd {
            ADBCommand::Reboot => Duration::from_secs(1),
            _ => Duration::from_secs(3),
        }
    }

    async fn execute_adb_command_with_retry(
        &mut self,
        cmd: &ADBCommand,
    ) -> AdbResult<ADBCommandResult> {
        let first_result = self.execute_adb_command(cmd).await;
        if Self::is_successful_result(&first_result) || !Self::should_retry_after_failure(cmd) {
            return first_result;
        }

        let failure_detail = Self::describe_failure(&first_result);
        Log::warn(&format!(
            "[ ADBExecutor ] 命令[{}]首次执行失败，准备重连后重试: {}",
            cmd, failure_detail
        ));

        sleep(Self::retry_delay_for(cmd)).await;
        if !self.reconnect().await {
            Log::warn(&format!(
                "[ ADBExecutor ] 命令[{}]首次执行失败后重连失败，停止重试",
                cmd
            ));
            return first_result;
        }

        Log::info(&format!("[ ADBExecutor ] 命令[{}]重连成功，开始重试", cmd));
        let retry_result = self.execute_adb_command(cmd).await;
        if Self::is_successful_result(&retry_result) {
            Log::info(&format!("[ ADBExecutor ] 命令[{}]重试成功", cmd));
        } else {
            Log::warn(&format!(
                "[ ADBExecutor ] 命令[{}]重试后仍失败: {}",
                cmd,
                Self::describe_failure(&retry_result)
            ));
        }
        retry_result
    }

    async fn execute_shell_with_retry(&mut self, shell_cmd: &str) -> AdbResult<ADBCommandResult> {
        let first_result = self.execute_shell(shell_cmd).await;
        if Self::is_successful_result(&first_result) {
            return first_result;
        }

        Log::warn(&format!(
            "[ ADBExecutor ] shell命令首次执行失败，准备重连后重试: {}",
            shell_cmd
        ));
        sleep(Duration::from_secs(3)).await;
        if !self.reconnect().await {
            Log::warn(&format!(
                "[ ADBExecutor ] shell命令首次执行失败后重连失败，停止重试: {}",
                shell_cmd
            ));
            return first_result;
        }

        Log::info(&format!(
            "[ ADBExecutor ] shell命令重连成功，开始重试: {}",
            shell_cmd
        ));
        let retry_result = self.execute_shell(shell_cmd).await;
        if Self::is_successful_result(&retry_result) {
            Log::info(&format!("[ ADBExecutor ] shell命令重试成功: {}", shell_cmd));
        } else {
            Log::warn(&format!(
                "[ ADBExecutor ] shell命令重试后仍失败: {}",
                shell_cmd
            ));
        }
        retry_result
    }

    async fn execute_capture_once(&mut self) -> Result<RgbaImage, String> {
        match self.device.as_mut() {
            Some(device) => device
                .framebuffer_inner()
                .map_err(|error| format!("[ ADBExecutor ] framebuffer_inner执行失败: {}", error)),
            None => Err("[ ADBExecutor ] 设备未连接：ADBDeviceExt为空".to_string()),
        }
    }

    async fn capture_with_retry(&mut self) -> Result<RgbaImage, String> {
        match self.execute_capture_once().await {
            Ok(image) => Ok(image),
            Err(first_error) => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 截图首次执行失败，准备重连后重试: {}",
                    first_error
                ));
                sleep(Duration::from_millis(200)).await;
                if !self.reconnect().await {
                    Log::warn("[ ADBExecutor ] 截图首次执行失败后重连失败，停止重试");
                    return Err(first_error);
                }

                Log::info("[ ADBExecutor ] 截图重连成功，开始重试");
                match self.execute_capture_once().await {
                    Ok(image) => {
                        Log::info("[ ADBExecutor ] 截图重试成功");
                        Ok(image)
                    }
                    Err(retry_error) => {
                        Log::warn(&format!(
                            "[ ADBExecutor ] 截图重试后仍失败: {}",
                            retry_error
                        ));
                        Err(retry_error)
                    }
                }
            }
        }
    }

    async fn execute_adb_command(&mut self, cmd: &ADBCommand) -> AdbResult<ADBCommandResult> {
        match cmd {
            ADBCommand::Reboot => {
                if let Some(device) = self.device.as_mut() {
                    let res = device
                        .reboot(RebootType::System)
                        .map(|_| ADBCommandResult::Success)
                        .unwrap_or_else(|error| ADBCommandResult::Failed(error.to_string()));
                    Ok(res)
                } else {
                    Err(AdbError::ConfigErr {
                        detail: "[ ADBExecutor ] 设备未连接：ADBDeviceExt为空".to_string(),
                    })
                }
            }
            ADBCommand::Click(point) => self.execute_shell(&click_cmd(point)).await,
            ADBCommand::LongClick(point) => self.execute_shell(&long_click_cmd(point)).await,
            ADBCommand::LongClickAndSwipe(point1, point2, duration) => {
                self.execute_shell(&long_click_and_swipe(point1, point2, duration))
                    .await
            }
            ADBCommand::Swipe(point1, point2) => {
                self.execute_shell(&swipe_cmd(point1, point2)).await
            }
            ADBCommand::SwipeWithDuration(point1, point2, duration) => {
                self.execute_shell(&swipe_duration_cmd(point1, point2, duration))
                    .await
            }
            ADBCommand::StartActivity(package_name, activity_name) => {
                if let Some(device) = self.device.as_mut() {
                    let res = device
                        .run_activity(package_name, activity_name)
                        .map(|_| ADBCommandResult::Success)
                        .unwrap_or_else(|error| ADBCommandResult::Failed(error.to_string()));
                    Ok(res)
                } else {
                    Err(AdbError::ConfigErr {
                        detail: "[ ADBExecutor ] 设备未连接：ADBDeviceExt为空".to_string(),
                    })
                }
            }
            ADBCommand::Capture(_) => Err(AdbError::ConfigErr {
                detail: "[ ADBExecutor ] 错误路径：截图指令应在单独命令通道中处理".to_string(),
            }),
            ADBCommand::StopApp(pkg_name) => self.execute_shell(&stop_app_cmd(pkg_name)).await,
            ADBCommand::InputText(text) => self.execute_shell(&input_text_cmd(text)).await,
            ADBCommand::Back => self.execute_shell(BACK).await,
            ADBCommand::Home => self.execute_shell(HOME).await,
            ADBCommand::Sequence(cmds) | ADBCommand::ReliableSequence(cmds) => {
                let cmd_string = Self::translate_sequence_cmd(cmds);
                self.execute_shell(&cmd_string).await
            }
            ADBCommand::Loop(_) | ADBCommand::StopLoop(_) => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 命令[{}]路线错误，应在父级高频/低频命令处理函数中处理",
                    cmd
                ));
                Ok(ADBCommandResult::Success)
            }
            ADBCommand::Duration(interval) => {
                sleep(Duration::from_millis(*interval)).await;
                Ok(ADBCommandResult::Success)
            }
            ADBCommand::Pause | ADBCommand::Resume => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 命令[{}]路线错误，应在父级高频/低频命令处理函数中处理",
                    cmd
                ));
                Ok(ADBCommandResult::Success)
            }
            ADBCommand::ChangeConnectConfig(config) => {
                self.device = None;
                {
                    let mut old_conf = self.adb_config.lock().await;
                    *old_conf = config.clone();
                }
                if self.reconnect().await {
                    Ok(ADBCommandResult::Success)
                } else {
                    Err(AdbError::ConfigErr {
                        detail: format!("[ ADBExecutor ] 更新ADB连接配置后重连失败: {}", config),
                    })
                }
            }
            ADBCommand::AwaitResult(_, _) => {
                Log::warn(&format!(
                    "[ ADBExecutor ] 命令[{}]路线错误，AwaitResult应在父级低频命令处理函数中处理",
                    cmd
                ));
                Ok(ADBCommandResult::Success)
            }
        }
    }

    async fn execute_shell(&mut self, cmd: &str) -> AdbResult<ADBCommandResult> {
        if let Some(device) = self.device.as_mut() {
            let res = device
                .shell_command(&cmd, None, None)
                .map(|_| ADBCommandResult::Success)
                .unwrap_or_else(|error| ADBCommandResult::Failed(error.to_string()));
            Ok(res)
        } else {
            Err(AdbError::ConnectFailed {
                ipv4: "unknown".to_string(),
                e: "ADBDeviceExt为空".to_string(),
            })
        }
    }

    async fn reconnect(&mut self) -> bool {
        self.try_to_connect().await
    }

    async fn try_to_connect(&mut self) -> bool {
        let cfg = self.adb_config.lock().await;
        let device: Option<Box<dyn ADBDeviceExt + Send + Sync>> = match &*cfg {
            ADBConnectConfig::ServeByIdentifier(dev) => {
                if !dev.valid() {
                    Log::warn(
                        "[ ADBExecutor ] ServeByIdentifier 配置无效（缺少 adb_path/server_connect/identifier）",
                    );
                    None
                } else {
                    let device = ADBServer::new_from_path(
                        dev.adb_config.server_connect.unwrap(),
                        dev.adb_config.adb_path.clone(),
                    )
                    .get_device_by_name(dev.identifier.as_ref().unwrap().as_str());
                    match device {
                        Ok(device) => Some(Box::new(device)),
                        Err(error) => {
                            Log::warn(&format!(
                                "[ ADBExecutor ] ServeByIdentifier 获取设备失败: {}",
                                error
                            ));
                            None
                        }
                    }
                }
            }
            ADBConnectConfig::DirectTcp(dev) => {
                if dev.is_none() {
                    Log::warn("[ ADBExecutor ] DirectTcp 配置无效：未设置连接地址");
                    None
                } else {
                    match ADBTcpDevice::new(SocketAddr::V4(dev.unwrap())) {
                        Ok(device) => Some(Box::new(device)),
                        Err(error) => {
                            Log::warn(&format!(
                                "[ ADBExecutor ] DirectTcp 连接失败 ({}): {}",
                                dev.unwrap(),
                                error
                            ));
                            None
                        }
                    }
                }
            }
        };
        drop(cfg);

        self.device = device;
        let connected = self.device.is_some();
        if connected {
            Log::info("[ ADBExecutor ] 设备连接成功");
        } else {
            Log::warn("[ ADBExecutor ] 设备连接失败");
        }
        connected
    }
}
