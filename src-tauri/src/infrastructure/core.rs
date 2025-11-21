pub mod cores_affinity;
pub mod core_error;
pub mod time_format;
// Rayon线程池集成

// 重新导出主要类型供外部使用
pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;
pub use serde::{Deserialize, Serialize};
//解决版本不一致的问题
pub use bincode_another::encode_to_vec as encode_to_vec;
pub use bincode_another::decode_from_slice as decode_from_slice;
pub use bincode_another::config::standard as serialize_config;
pub use thiserror::Error;

use uuid::Uuid;

pub type DeviceId = Uuid;  // UUID v7
pub type ScriptId = Uuid;  // UUID v7
pub type TaskId = Uuid;
pub type GuardId = Uuid;
pub type PolicyId = Uuid;
pub type SubFlowId = Uuid;