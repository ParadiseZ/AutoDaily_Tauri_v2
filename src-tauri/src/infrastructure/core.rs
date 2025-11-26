pub mod core_error;
pub mod cores_affinity;
pub mod time_format;
// Rayon线程池集成

// 重新导出主要类型供外部使用
pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;
pub use serde::{Deserialize, Serialize};
//解决版本不一致的问题
pub use bincode::config::standard as serialize_config;
pub use bincode::decode_from_slice;
use bincode::{Decode, Encode};
pub use bincode::encode_to_vec;
pub use thiserror::Error;

use uuid::Uuid;

pub type DeviceId = UuidV7; // UUID v7
pub type ScriptId = UuidV7; // UUID v7
pub type TaskId = UuidV7;
pub type GuardId = UuidV7;
pub type PolicyId = UuidV7;
pub type SubFlowId = UuidV7;

pub type MessageId = UuidV7;
/// 统一的新类型：所有 ID 都用这个
#[derive(
    Encode, Decode,
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
    serde::Serialize, serde::Deserialize
)]
//#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UuidV7(pub u128);

impl UuidV7 {
    /// 生成一个新的 UUIDv7（时间有序）
    #[inline(always)]
    pub fn new_v7() -> Self {
        // uuid::Uuid::now_v7() 是时间递增的，完美适合做 ID
        UuidV7(u128::from_be_bytes(Uuid::now_v7().into_bytes()))
    }

    #[inline(always)]
    pub fn to_string(&self) -> String {
        /// 转成标准字符串（如 "67e5504410b1426f9247bb680e5fe0c8."）
        Uuid::from_bytes(self.0.to_be_bytes()).simple().to_string()
    }

    /// 方便 Display（println!、format! 等直接用）
    #[inline(always)]
    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0.to_be_bytes())
    }
}

impl std::fmt::Display for UuidV7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_uuid())
    }
}

impl From<Uuid> for UuidV7 {
    fn from(uuid: Uuid) -> Self {
        UuidV7(u128::from_be_bytes(uuid.into_bytes()))
    }
}

impl From<UuidV7> for Uuid {
    fn from(id: UuidV7) -> Self {
        Uuid::from_bytes(id.0.to_be_bytes())
    }
}
