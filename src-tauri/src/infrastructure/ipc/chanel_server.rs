use crate::api::api_response::ApiResponse;
use crate::constant::project::{MAIN_WINDOW, SOCKET_NAME};
use crate::infrastructure::app_handle::get_app_handle;
use crate::infrastructure::context::child_process_sec::RunningStatus;
use crate::infrastructure::context::init_error::InitError;
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::core::{Deserialize, DeviceId, Serialize};
use crate::infrastructure::ipc::chanel_trait::ChannelTrait;
use crate::infrastructure::ipc::channel_error::ChannelResult;
use crate::infrastructure::ipc::message::{IpcMessage, MessagePayload, MessageType};
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Listener;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use std::sync::{Arc, RwLock};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufWriter};
use tokio::time::Instant;
use crate::infrastructure::logging::log_trait::Log;

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcClientState {
    pid: u32,
    device_id : Arc<DeviceId>,
    last_heartbeat: Instant,
    writer: Arc<RwLock<BufWriter<LocalSocketStream>>>,
    running_status: RunningStatus,
}
pub struct IpcServer;

impl IpcServer {
    pub(crate) fn start() -> ChannelResult<()> {
        let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>().map_err(|e| InitError::InitMainIpcServerErr { name : SOCKET_NAME.to_string(),e: e.to_string() })?;
        let opts = ListenerOptions::new().name(name);
        let listener: interprocess::local_socket::tokio::Listener = match opts.create_tokio() {
            Err(e) => {
                Log::info(&format!("[ socket ] ❌ {}，启动失败", SOCKET_NAME));
                return Err(InitError::InitMainIpcServerErr { name : SOCKET_NAME.to_string(),e: e.to_string() })?;
            }
            Ok(l) => l,
        };
        Log::info(&format!("[ socket ] 主进程监听于: {}", SOCKET_NAME));
        tokio::spawn(async {
            loop {
                let stream= match listener.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        Log::error(&format!("[ socket ] ️⚠️ 接受连接失败: {}", e));
                        continue;
                    }
                };

                let (reader, writer) = stream.split();

                // 3. 启动读任务
                //let send_task = Self::send_loop(log_rx, cmd_rx, writer);
                tokio::spawn(async move {
                    let mut reader = reader;
                    loop {
                        match Self::recv_message(&mut reader).await {
                            Ok(buffer) => {
                                // 分发
                                if let Ok((msg, _)) = bincode::decode_from_slice::<IpcMessage, _>(&buffer, bincode::config::standard()) {
                                    match msg.payload {
                                        MessageType::Event => {
                                            match msg.payload {
                                                MessagePayload::SocketRegistration(pid)=>{
                                                    let device_id = Arc::new(msg.source_or_target);
                                                    Log::info(&format!("[ socket ] [{}]ipc加入连接...", *device_id));
                                                    let childrens = get_app_handle().state::<MainProcessCtx>().ipc_servers.clone();
                                                    match childrens.write() {
                                                        Ok(mut childrens) => {
                                                            childrens.insert(device_id.clone(), IpcClientState {
                                                                pid,
                                                                device_id,
                                                                last_heartbeat: Instant::now(),
                                                                writer,
                                                                running_status: RunningStatus::Idle,
                                                            });
                                                            let msg = format!("[ socket ] [{}]加入ipc连接成功！", device_id);
                                                            Log::info(&msg);
                                                            Self::success_to_ui(  None, Some(msg));
                                                        }
                                                        Err(_) => {
                                                            let msg = format!("[ socket ] [{}]加入ipc连接失败:获取锁失败！", device_id);
                                                            Log::error(&msg);
                                                            Self::error_to_ui(  None, Some(msg));
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    Self::handle_msg(msg).await;
                                                }
                                            }
                                        }
                                        _ =>{
                                            Self::handle_msg(msg).await;
                                        }
                                    }
                                }else{
                                    let msg = "解码来自子进程的消息数据失败！";
                                    Log::error(&msg);
                                    Self::error_to_ui(  None, Some(msg));
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
    fn send_msg(msg: IpcMessage, device_id: DeviceId) {
        tokio::spawn(async move {
            match get_app_handle().state::<MainProcessCtx>().ipc_servers.read() {
                Ok(childrens) => {
                    match childrens.get(&device_id){
                        Some(ipc_client_state) => {
                            let mut sender = match ipc_client_state.writer.write() {
                                Ok(s) => s,
                                Err(_) => {
                                    let msg = format!("[ socket ] ️向设备[{}]发送消息失败：无法获取该设备锁！", device_id);
                                    Log::error(&msg);
                                    Self::error_to_ui(  None, Some(msg));
                                    return;
                                }
                            };

                            let mut buffer = match bincode::encode_to_vec(msg, bincode::config::standard()){
                                Ok(b) => b,
                                Err(_) => {
                                    let msg = format!("[ socket ] ️向设备[{}]发送消息失败：编码消息失败！", device_id);
                                    Log::error(&msg);
                                    Self::error_to_ui(  None, Some(msg));
                                    return;
                                }
                            };

                            let len = match u32::try_from(buffer.len()){
                                Ok( l)  => l,
                                Err(_) => {
                                    let msg = format!("[ socket ] ️向设备[{}]发送消息失败：计算消息长度失败！", device_id);
                                    Log::error(&msg);
                                    Self::error_to_ui(  None, Some(msg));
                                    return;
                                }
                            };
                            if let Err(_) = sender.write_all(&len.to_le_bytes()){
                                let msg = format!("[ socket ] ️向设备[{}]发送消息失败：写入消息长度失败！", device_id);
                                Log::error(&msg);
                                Self::error_to_ui(  None, Some(msg));
                                return;
                            };
                            if let Err(_) = sender.write_all(&buffer).await{
                                let msg = format!("[ socket ] ️向设备[{}]发送消息失败：写入消息失败！", device_id);
                                Log::error(&msg);
                                Self::error_to_ui(  None, Some(msg));
                                return;
                            };
                            if let Err(_) = sender.flush().await{
                                let msg = format!("[ socket ] ️向设备[{}]发送消息失败：刷新缓存失败！！", device_id);
                                Log::error(&msg);
                                Self::error_to_ui(  None, Some(msg));
                            };
                        }
                        _ => {
                            let msg = format!("[ socket ] ️向设备[{}]发送消息失败：刷新缓存失败！", device_id);
                            Log::warn(&msg);
                            Self::error_to_ui(  None, Some(msg));
                        }
                    }
                }
                Err(_) => {
                    let msg = format!("[ socket ] ️向设备[{}]发送消息失败：获取ipc通道数据锁失败！", device_id);
                    Log::warn(&msg);
                    Self::error_to_ui( None, Some(msg));
                }
            }
        });
    }

    fn success_to_ui(data: Option<String>,msg : Option<String>){
        if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
            let emit_msg = ApiResponse::success(data,msg);
            if let Err( e) =  main_window.emit("send-event", emit_msg){
                Log::error(&format!("向UI发送消息失败: 向前端提交send-event事件失败！{}",e));
            }
        }else {
            Log::warn(&format!("向UI发送消息失败: 未找到窗口[ {MAIN_WINDOW} ]！"));
        }
    }

    fn error_to_ui(data: Option<String>,msg : Option<String>){
        if let Some(main_window) = get_app_handle().get_webview_window(MAIN_WINDOW) {
            let emit_msg = ApiResponse::failed(data,msg);
            if let Err( e) =  main_window.emit("send-event", emit_msg){
                Log::error(&format!("向UI发送消息失败: 向前端提交send-event事件失败！{}",e));
            }
        }else {
            Log::error(&format!("向UI发送消息失败: 未找到窗口[ {MAIN_WINDOW} ]！"));
        }
    }
}
impl ChannelTrait for IpcServer {
    fn handle_msg(msg: IpcMessage) {
        match msg.message_type {
            MessageType::Command => {

             }
            _=>{

            }
        }
    }
}
