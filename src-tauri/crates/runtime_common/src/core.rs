pub mod core_error;
pub mod cores_affinity;
pub mod time_format;

pub use ahash::AHashMap as HashMap;
pub use bincode::config::standard as serialize_config;
pub use bincode::decode_from_slice;
pub use bincode::encode_to_vec;
use bincode::{Decode, Encode};
pub use serde::{Deserialize, Serialize};
use sqlx::error::BoxDynError;
use sqlx::{Database, Sqlite};
pub use thiserror::Error;
use uuid::Uuid;

pub type DeviceId = UuidV7;
pub type ScriptId = UuidV7;
pub type UserId = UuidV7;
pub type TaskId = UuidV7;
pub type GuardId = UuidV7;
pub type PolicyId = UuidV7;
pub type PolicyGroupId = UuidV7;
pub type PolicySetId = UuidV7;
pub type SubFlowId = UuidV7;
pub type StepId = UuidV7;
pub type MessageId = UuidV7;
pub type SessionId = UuidV7;
pub type ExecutionId = UuidV7;
pub type ScheduleId = UuidV7;
pub type TemplateId = UuidV7;
pub type ScriptTemplateValueId = UuidV7;
pub type AccountId = String;

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        let s: &str = <&str as sqlx::Decode<Sqlite>>::decode(value)?;
        let uuid = Uuid::parse_str(s)?;
        Ok(UuidV7::from(uuid))
    }
}

impl sqlx::Type<Sqlite> for UuidV7 {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &<Sqlite as Database>::TypeInfo) -> bool {
        <String as sqlx::Type<Sqlite>>::compatible(ty)
    }
}

impl UuidV7 {
    #[inline(always)]
    pub fn new_v7() -> Self {
        UuidV7(u128::from_be_bytes(Uuid::now_v7().into_bytes()))
    }

    #[inline(always)]
    pub fn to_string(&self) -> String {
        Uuid::from_bytes(self.0.to_be_bytes()).simple().to_string()
    }

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

impl ts_rs::TS for UuidV7 {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;

    fn name(_cfg: &ts_rs::Config) -> String {
        "string".to_owned()
    }

    fn inline(_cfg: &ts_rs::Config) -> String {
        "string".to_owned()
    }

    fn visit_dependencies(_: &mut impl ts_rs::TypeVisitor) {}
    fn visit_generics(_: &mut impl ts_rs::TypeVisitor) {}

    fn output_path() -> Option<std::path::PathBuf> {
        None
    }
}
