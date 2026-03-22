use crate::api::api_response::ApiResponse;
use crate::constant::project::{MAIN_WINDOW, SOCKET_NAME};
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::context::child_process_sec::RunningStatus;
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::core::time_format::LocalTimer;
use crate::infrastructure::core::{
    decode_from_slice, encode_to_vec, serialize_config, Deserialize, DeviceId, Serialize,
};
use crate::infrastructure::ipc::chanel_trait::ChannelTrait;
use crate::infrastructure::ipc::channel_error::{ChannelError, ChannelResult};
use crate::infrastructure::ipc::message::{IpcMessage, MessagePayload};
use crate::infrastructure::logging::log_trait::Log;
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Listener;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncWriteExt, BufWriter, WriteHalf};
use tokio::sync::RwLock as TokioRwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcClientState {
    pid: u32,
    device_id: Arc<DeviceId>,
    last_heartbeat: LocalTimer,
    #[serde(skip)]
    writer: Option<Arc<TokioRwLock<BufWriter<WriteHalf<LocalSocketStream>>>>>,
    running_status: RunningStatus,
}
pub struct IpcServer;

impl IpcServer {
    pub(crate) fn start() -> ChannelResult<()> {
        let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>().map_err(|e| {
            ChannelError::InitFailed {
                e: format!("初始化主进程 IPC 服务失败 [{}]: {}", SOCKET_NAME, e),
            }
        })?;
        let opts = ListenerOptions::new().name(name);
        let listener: interprocess::local_socket::tokio::Listener = match opts.create_tokio() {
            Err(e) => {
                Log::info(&format!("[ socket ] ❌ {}，启动失败", SOCKET_NAME));
                return Err(ChannelError::InitFailed {
                    e: format!("初始化主进程 IPC 服务失败 [{}]: {}", SOCKET_NAME, e),
                });
            }
            Ok(l) => l,
        };
        Log::info(&format!("[ socket ] 主进程监听于: {}", SOCKET_NAME));
        tokio::spawn(async move {
            loop {
                let stream = match listener.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        Log::error(&format!("[ socket ] ️⚠️ 接受连接失败: {}", e));
                        continue;
                    }
                };

                let (mut reader, writer) = tokio::io::split(stream);
                let mut writer = Some(writer);

                // 3. 启动读任务
                //let send_task = Self::send_loop(log_rx, cmd_rx, writer);
                tokio::spawn(async move {
                    //let mut reader = reader;
                    loop {
                        match Self::recv_message(&mut reader).await {
                            Ok(buffer) => {
                                // 分发
                                if let Ok((msg, _)) =
                                    decode_from_slice::<IpcMessage, _>(&buffer, serialize_config())
                                {
                                    match &msg.payload {
                                        MessagePayload::SocketRegistration(pid) => {
                                            let pid = *pid;
                                            let device_id = Arc::new(msg.source_or_target);
                                            Log::info(&format!(
                                                "[ socket ] [{}]ipc加入连接...",
                                                *device_id
                                            ));
                                            let childrens = get_app_handle()
                                                .state::<MainProcessCtx>()
                                                .ipc_servers
                                                .clone();
                                            match childrens.write() {
                                                Ok(mut childrens) => {
                                                    childrens.insert(
                                                        device_id.clone(),
                                                        Arc::new(IpcClientState {
                                                            pid,
                                                            device_id: device_id.clone(),
                                                            last_heartbeat:
                                                                LocalTimer::DayStamp,
                                                            writer: writer.take().map(|w| {
                                                                Arc::new(TokioRwLock::new(
                                                                    BufWriter::new(w),
                                                                ))
                                                            }),
                                                            running_status: RunningStatus::Idle,
                                                        }),
                                                    );
                                                    let msg = format!(
                                                        "[ socket ] [{}]加入ipc连接成功！",
                                                        device_id
                                                    );
                                                    Log::info(&msg);
                                                    Self::success_to_ui(None, Some(msg));
                                                }
                                                Err(_) => {
                                                    let msg = format!("[ socket ] [{}]加入ipc连接失败:获取锁失败！", device_id);
                                                    Log::error(&msg);
                                                    Self::error_to_ui(None, Some(msg));
                                                }
                                            };
                                        }
                                        _ => {
                                            Self::handle_msg(msg);
                                        }
                                    }
                                } else {
                                    let msg = "解码来自子进程的消息数据失败！".to_string();
                                    Log::error(&msg);
                                    Self::error_to_ui(None, Some(msg));
                                }
                            }
                            Err(_) => break, // 连接断开
                        }
                    }
                });
            }
        });
        Ok(())
    }
    pub async fn send_to_client(device_id: &DeviceId, msg: IpcMessage) {
        let device_id = *device_id;
        let ipc_client_state_opt = {
            match get_app_handle()
                .state::<MainProcessCtx>()
                .ipc_servers
                .read()
            {
                Ok(childrens) => childrens.get(&device_id).cloned(),
                Err(_) => {
                    let msg = format!(
                        "[ socket ] ️向设备[{}]发送消息失败：获取ipc通道数据锁失败！",
                        device_id
                    );
                    Log::warn(&msg);
                    Self::error_to_ui(None, Some(msg));
                    None
                }
            }
        };

        if let Some(ipc_client_state) = ipc_client_state_opt {
            if let Some(writer_lock) = &ipc_client_state.writer {
                let mut sender = writer_lock.write().await;

                let buffer = match encode_to_vec(msg, serialize_config()) {
                    Ok(b) => b,
                    Err(_) => {
                        let msg = format!(
                            "[ socket ] ️向设备[{}]发送消息失败：编码消息失败！",
                            device_id
                        );
                        Log::error(&msg);
                        Self::error_to_ui(None, Some(msg));
                        return;
                    }
                };

                let len = match u32::try_from(buffer.len()) {
                    Ok(l) => l,
                    Err(_) => {
                        let msg = format!(
                            "[ socket ] ️向设备[{}]发送消息失败：计算消息长度失败！",
                            device_id
                        );
                        Log::error(&msg);
                        Self::error_to_ui(None, Some(msg));
                        return;
                    }
                };
                if let Err(_) = sender.write_all(&len.to_le_bytes()).await {
                    let msg = format!(
                        "[ socket ] ️向设备[{}]发送消息失败：写入消息长度失败！",
                        device_id
                    );
                    Log::error(&msg);
                    Self::error_to_ui(None, Some(msg));
                    return;
                };
                if let Err(_) = sender.write_all(&buffer).await {
                    let msg = format!(
                        "[ socket ] ️向设备[{}]发送消息失败：写入消息失败！",
                        device_id
                    );
                    Log::error(&msg);
                    Self::error_to_ui(None, Some(msg));
                    return;
                };
                if let Err(_) = sender.flush().await {
                    let msg = format!(
                        "[ socket ] ️向设备[{}]发送消息失败：刷新缓存失败！",
                        device_id
                    );
                    Log::error(&msg);
                    Self::error_to_ui(None, Some(msg));
                };
            } else {
                let msg = format!(
                    "[ socket ] ️向设备[{}]发送消息失败：Writer不可用！",
                    device_id
                );
                Log::error(&msg);
                Self::error_to_ui(None, Some(msg));
            }
        } else {
            let msg = format!(
                "[ socket ] ️向设备[{}]发送消息失败：获取该设备状态信息失败！",
                device_id
            );
            Log::error(&msg);
            Self::error_to_ui(None, Some(msg));
        }
    }

    fn success_to_ui(data: Option<String>, msg: Option<String>) {
        if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
            let emit_msg = ApiResponse::success(data, msg);
            if let Err(e) = main_window.emit("send-event", emit_msg) {
                Log::error(&format!(
                    "向UI发送消息失败: 向前端提交send-event事件失败！{}",
                    e
                ));
            }
        } else {
            Log::warn(&format!("向UI发送消息失败: 未找到窗口[ {MAIN_WINDOW} ]！"));
        }
    }

    fn error_to_ui(data: Option<String>, msg: Option<String>) {
        if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
            let emit_msg = ApiResponse::failed(data, msg);
            if let Err(e) = main_window.emit("send-event", emit_msg) {
                Log::error(&format!(
                    "向UI发送消息失败: 向前端提交send-event事件失败！{}",
                    e
                ));
            }
        } else {
            Log::error(&format!("向UI发送消息失败: 未找到窗口[ {MAIN_WINDOW} ]！"));
        }
    }
}
impl ChannelTrait for IpcServer {
    fn handle_msg(msg: IpcMessage) {
        // 委托给消息处理器
        tokio::spawn(async move {
            crate::infrastructure::ipc::msg_handler_main::handle_child_message(msg).await;
        });
    }
}
