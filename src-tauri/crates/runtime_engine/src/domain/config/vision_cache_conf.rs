use crate::infrastructure::core::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct VisionTextCacheConfig {
    pub enabled: bool,
    pub dir: String,
}

impl VisionTextCacheConfig {
    pub fn to_runtime_config(&self, fallback_dir: PathBuf) -> VisionTextCacheRuntimeConfig {
        let trimmed_dir = self.dir.trim();

        VisionTextCacheRuntimeConfig {
            enabled: self.enabled,
            dir: if self.enabled {
                Some(if trimmed_dir.is_empty() {
                    fallback_dir
                } else {
                    PathBuf::from(trimmed_dir)
                })
            } else {
                None
            },
        }
    }
}

impl Default for VisionTextCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dir: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct VisionTextCacheRuntimeConfig {
    pub enabled: bool,
    pub dir: Option<PathBuf>,
}
