pub mod core_error;
pub mod cores_affinity;
pub mod time_format;
// Rayon线程池集成

// 重新导出主要类型供外部使用
pub use ahash::AHashMap as HashMap;
//解决版本不一致的问题
pub use bincode::config::standard as serialize_config;
pub use bincode::decode_from_slice;
pub use bincode::encode_to_vec;
use bincode::{Decode, Encode};
//pub use ahash::AHashSet as HashSet;
pub use serde::{Deserialize, Serialize};
use sqlx::error::BoxDynError;
use sqlx::{Database, Sqlite};
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
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash
)]
//#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UuidV7(pub u128);

impl serde::Serialize for UuidV7 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for UuidV7 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
        Ok(UuidV7::from(uuid))
    }
}

impl sqlx::Decode<'_, Sqlite> for UuidV7 {
    fn decode(value: <Sqlite as Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        // 先从 SQLite 提取字符串
        let s: &str = <&str as sqlx::Decode<Sqlite>>::decode(value)?;
        // 解析 UUID 字符串
        let uuid = Uuid::parse_str(s)?;
        Ok(UuidV7::from(uuid))
    }
}

impl sqlx::Type<Sqlite> for UuidV7 {
    fn type_info() ->  <Sqlite as Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &<Sqlite as Database>::TypeInfo) -> bool {
        <String as sqlx::Type<Sqlite>>::compatible(ty)
    }
}

impl UuidV7 {
    /// 生成一个新的 UUIDv7（时间有序）
    #[inline(always)]
    pub fn new_v7() -> Self {
        // uuid::Uuid::now_v7() 是时间递增的，完美适合做 ID
        UuidV7(u128::from_be_bytes(Uuid::now_v7().into_bytes()))
    }

    /// 转成标准字符串（如 "67e5504410b1426f9247bb680e5fe0c8."）
    #[inline(always)]
    pub fn to_string(&self) -> String {

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
