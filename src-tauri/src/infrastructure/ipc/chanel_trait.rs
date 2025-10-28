use crate::constant::project::MAX_MESSAGE_SIZE;
use crate::infrastructure::context::child_process_sec::{set_running_status, RunningStatus};
use crate::infrastructure::ipc::channel_error::{ChannelError, ChannelResult};
use crate::infrastructure::ipc::message::IpcMessage;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait ChannelTrait {
    async fn recv_message<R: AsyncReadExt + Unpin>(
        reader: &mut R,
    ) -> ChannelResult<Vec<u8>> {
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).await
            .map_err(|e| ChannelError::ReadErr { detail: "读取数据长度失败！".to_string(), e: e.to_string() })?;

        let len = u32::from_le_bytes(len_bytes) as usize;

        // 安全限制：防止 OOM
        if len > MAX_MESSAGE_SIZE {
            //len_bytes = 0u8;
            return Err(ChannelError::MessageTooLong {detail:"读取失败！".to_string()});
        }
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut buffer = vec![0; len];
        reader.read_exact(&mut buffer).await
            .map_err(|e| ChannelError::ReadErr { detail: "读取数据失败！".to_string(), e: e.to_string() })?;

        Ok(buffer)
    }

    fn handle_msg(msg: IpcMessage);
}