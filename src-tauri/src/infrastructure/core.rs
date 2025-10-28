pub mod rayon_pool;
pub mod cores_affinity;
pub mod core_error;
pub mod time_format;
// Rayon线程池集成

// 重新导出主要类型供外部使用
pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;
pub use serde::{Deserialize, Serialize};
pub use thiserror::Error;

use uuid::Uuid;

pub type DeviceId = Uuid;  // UUID v7
pub type ScriptId = Uuid;  // UUID v7
pub type TaskId = Uuid;
pub type GuardId = Uuid;
pub type PolicyId = Uuid;
pub type SubFlowId = Uuid;