use crate::constant::project::{HEARTBEAT_INTERVAL, SOCKET_NAME};
use crate::infrastructure::context::child_process_sec::{get_ipc_client, process_need_stop, set_running_status, RunningStatus};
use crate::infrastructure::core::DeviceId;
use crate::infrastructure::ipc::chanel_trait::ChannelTrait;
use crate::infrastructure::ipc::channel_error::{ChannelError, ChannelResult};
use crate::infrastructure::ipc::message::{HeartbeatMessage, IpcMessage, MessagePayload, MessageType};
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Stream;
use interprocess::local_socket::ToNsName;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};
use tokio::time;
use crate::infrastructure::logging::log_trait::Log;

// ===== 子进程逻辑 =====
pub struct IpcClient {
    pub(crate) device_id: Arc<DeviceId>,
    pub(crate) log_level: AtomicU8,
    log_sender: Arc<Mutex<Option<mpsc::Sender<IpcMessage>>>>,
    ensure_sender: Arc<Mutex<Option<mpsc::Sender<IpcMessage>>>>,
}

impl IpcClient{
    pub(crate) fn new(device_id: Arc<DeviceId>, log_level: AtomicU8) -> Self {
        Self {
            device_id,
            log_level,
            log_sender: Arc::new(Mutex::new(None)),
            ensure_sender: Arc::new(Mutex::new(None)),
        }
    }

    async fn connect_and_run(&self) -> ChannelResult<()> {
        // 1. 建立连接
        let stream = LocalSocketStream::connect(
            SOCKET_NAME
                .to_ns_name::<interprocess::local_socket::GenericNamespaced>()
                .map_err(async |e| ChannelError::ConnectErr { device_id: self.device_id.to_string(), e: e.to_string() })?,
        )
            .await
            .map_err(async |e| ChannelError::ConnectErr { device_id: self.device_id.to_string(), e: e.to_string() })?;

        let (reader, writer) = stream.split();

        // 2. 创建新的通道（旧通道自动丢弃）
        let (log_tx, log_rx) = mpsc::channel(30);
        let (cmd_tx, cmd_rx) = mpsc::channel(50); // 命令缓冲 100 条

        *self.log_sender.lock().await = Some(log_tx);
        *self.ensure_sender.lock().await = Some(cmd_tx);

        // 3. 启动读写任务
        let send_task = Self::send_loop(log_rx, cmd_rx, writer);
        let recv_task = Self::recv_loop(reader);

        // 4. 启动心跳任务
        let heart_task = self.send_heart();

        // 5. 等待任一任务结束（表示连接断开）
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
            _ = heart_task => {},
        }

        // 6. 清理发送端（触发后续重连时重建）
        *self.log_sender.lock().await = None;
        *self.ensure_sender.lock().await = None;

        Err(ChannelError::ChannelClosed{device_id: self.device_id.to_string()})
    }
    
    pub(crate) fn spawn_reconnect_task(&'static self) {
        let self_arc = Arc::new(self.clone());
        let mut connect_num = 0u8;
        tokio::spawn(async move {
            loop {
                // 只会返回错误，以自动重连
                if let Err(_) = self_arc.clone().connect_and_run().await{
                    // 连接失败，等待后重试
                    connect_num += 1;
                    tokio::time::sleep(time::Duration::from_secs(1)).await;
                    if connect_num > 30 {
                        Log::error("子进程socket重连次数达到上限30，将放弃重连");
                        set_running_status(RunningStatus::Error);
                        break
                    }
                }
            }
        });
    }
    

    /// 发送消息循环函数
    ///
    /// 该函数负责从两个消息通道接收消息并发送到指定的写入器。
    /// 命令消息具有更高的优先级，以避免饥饿问题。
    /// 当任意一个通道关闭时，循环将终止。
    ///
    /// # 参数
    ///
    /// * `log_rx` - 日志消息接收器，无界通道，用于接收日志类型的消息
    /// * `cmd_rx` - 命令消息接收器，有界通道，用于接收命令类型的消息
    /// * `writer` - 消息写入器，实现 AsyncWriteExt 和 Unpin trait 的类型
    ///
    /// # 说明
    ///
    /// 循环使用 tokio::select! 宏监听两个通道的消息：
    /// 1. 优先处理命令消息通道，防止重要命令因日志消息过多而被延迟
    /// 2. 当两个通道都关闭时，循环结束
    async fn send_loop<W: AsyncWriteExt + Unpin + Send>(
        mut sure_rx: mpsc::Receiver<IpcMessage>,
        mut uncertain_rx: mpsc::Receiver<IpcMessage>,
        mut writer: W,
    ) {
        loop {
            tokio::select! {
                // 优先处理命令（避免饥饿）
                Some(cmd) = sure_rx.recv() => {
                    if let Err(_) = Self::send_message(&mut writer, &cmd).await {
                        break;
                    }
                },
                Some(log) = uncertain_rx.recv() => {
                    if let Err(_) = Self::send_message(&mut writer, &log).await {
                        break;
                    }
                },
                else => break, // 两个通道都关闭
            }
        }
    }

    pub async fn send_heart(&self){
        // 简单协议：[len: u32][data...]
        let dev_id = self.device_id.clone();
        let msg = IpcMessage::new(*dev_id,MessageType::Heartbeat,MessagePayload::Heartbeat(HeartbeatMessage { cpu_usage: 0.5, memory_usage: 0 }));
        loop {
            time::sleep(HEARTBEAT_INTERVAL).await;
            if let Err(_) = self.send_ensure(msg.clone()).await {
                if process_need_stop() {
                    break
                }
            }
        }
    }

    async fn send_message<W: AsyncWriteExt + Unpin + Send>(
        writer: &mut W,
        msg: &IpcMessage,
    ) -> ChannelResult<()> {
        // 简单协议：[len: u32][data...]
        let encoded = bincode::encode_to_vec(msg, bincode::config::standard())
            .map_err(|e| ChannelError::EncodeErr { e: e.to_string() })?;
        let len = u32::try_from(encoded.len())
            .map_err(|e| ChannelError::MessageTooLong {detail:"发送失败！".to_string()})?;
        writer.write_all(&len.to_le_bytes()).await
            .map_err(|e| ChannelError::WriteErr { detail: "写入数据长度失败！".to_string(), e: e.to_string() })?;
        writer.write_all(&encoded).await
            .map_err(|e| ChannelError::WriteErr { detail: "写入数据失败！".to_string(), e: e.to_string() })?;
        writer.flush().await
            .map_err(|e| ChannelError::WriteErr { detail: "刷新缓存失败！".to_string(), e: e.to_string() })?;
        Ok(())
    }

    fn send_uncertain(&self, log: IpcMessage) {
        if let Ok(tx) = self.log_sender.try_lock() {
            if let Some(sender) = tx.as_ref(){
                let _ = sender.send(log);// 失败就丢弃
            }
        }
    }

    async fn send_ensure(&self, msg: IpcMessage) -> ChannelResult<()> {
        let tx = self.ensure_sender.lock().await;
        if let Some(sender) = tx.as_ref(){
            sender.send(msg)
                .await
                .map_err(|_| ChannelError::SendErr)
        } else {
            Err(ChannelError::ChannelClosed { device_id: self.device_id.to_string() })
        }
    }
    
    async fn recv_loop<R: AsyncReadExt + Unpin + Send>(mut reader: R) {
        loop {
            match Self::recv_message(&mut reader).await {
                Ok(buffer) => {
                    // 分发
                    if let Ok((msg, _)) = bincode::decode_from_slice::<IpcMessage, _>(&buffer, bincode::config::standard()) {
                        Self::handle_msg(msg).await;
                    }
                }
                Err(_) => break, // 连接断开
            }
        }
    }
}

/// 日志相关方法在日志模块里
impl ChannelTrait for IpcClient {
    fn handle_msg(msg: IpcMessage) {
        match msg.message_type {
            MessageType::Logger => {
                // 日志级别切换
                if let MessagePayload::Logger(log) = msg.payload {
                    let client = get_ipc_client();
                    client.log_level.store(log.level as u8, Ordering::Release)
                }
            }
            MessageType::Command => {

            }
            _ => {}
        }
    }
}
