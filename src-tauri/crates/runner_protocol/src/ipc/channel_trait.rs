//! Shared IPC channel framing contract.
use crate::channel_error::{ChannelError, ChannelResult};
use crate::message::IpcMessage;
use tokio::io::AsyncReadExt;

pub const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

#[allow(async_fn_in_trait)]
pub trait ChannelTrait {
    async fn recv_message<R: AsyncReadExt + Unpin + Send>(
        reader: &mut R,
    ) -> ChannelResult<Vec<u8>> {
        let mut len_bytes = [0u8; 4];
        reader
            .read_exact(&mut len_bytes)
            .await
            .map_err(|e| ChannelError::ReadErr {
                detail: "读取数据长度失败！".to_string(),
                e: e.to_string(),
            })?;

        let len = u32::from_le_bytes(len_bytes) as usize;

        // 安全限制：防止 OOM
        if len > MAX_MESSAGE_SIZE {
            //len_bytes = 0u8;
            return Err(ChannelError::MessageTooLong {
                detail: "读取失败！".to_string(),
            });
        }
        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut buffer = vec![0; len];
        reader
            .read_exact(&mut buffer)
            .await
            .map_err(|e| ChannelError::ReadErr {
                detail: "读取数据失败！".to_string(),
                e: e.to_string(),
            })?;

        Ok(buffer)
    }

    fn handle_msg(msg: IpcMessage);
}
