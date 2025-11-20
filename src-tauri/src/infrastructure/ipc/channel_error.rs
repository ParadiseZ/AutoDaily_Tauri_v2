use crate::infrastructure::context::init_error::InitError;
use crate::infrastructure::core::{Deserialize, Error, Serialize};

/// IPC通道错误类型
#[derive(Debug, Error,Deserialize, Serialize)]
pub enum ChannelError {
    #[error("初始化全局ipc客户端数据失败：{e}")]
    InitFailed{ e: String},

    #[error(transparent)]
    InitErr(#[from] InitError::InitMainIpcServerErr),

    #[error("消息长度过长：{detail}")]
    MessageTooLong{ detail: String},
    
    #[error("ipc通道已关闭，设备id: {device_id}")]
    ChannelClosed { device_id: String },

    #[error("ipc通道建立失败, 设备id:{device_id}, {e}")]
    ConnectErr { device_id:String, e: String },

    #[error("编码消息失败:{e}")]
    EncodeErr { e: String },

    #[error("写入失败:{detail}, {e}")]
    WriteErr { detail:String,e: String },

    #[error("读取失败:{detail}, {e}")]
    ReadErr { detail:String,e: String },

    #[error("解码消息失败:{e}")]
    DecodeErr { e: String },

    #[error("发送非日志消息失败:{e}")]
    SendErr { e: String },

    #[error("向设备[{device_id}]发送消息失败: {e}")]
    SendToChildErr { device_id: String, e: String },
}

pub type ChannelResult<T> = Result<T, ChannelError>;