use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct VisionTextCacheConfig {
    pub enabled: bool,
    pub dir: String,
    pub signature_grid_size: u16,
}

impl VisionTextCacheConfig {
    pub fn to_runtime_config(&self, fallback_dir: PathBuf) -> VisionTextCacheRuntimeConfig {
        let trimmed_dir = self.dir.trim();

        VisionTextCacheRuntimeConfig {
            enabled: self.enabled,
            dir: self.enabled.then(|| {
                if trimmed_dir.is_empty() {
                    fallback_dir
                } else {
                    PathBuf::from(trimmed_dir)
                }
            }),
            signature_grid_size: self.signature_grid_size.max(1),
        }
    }
}

impl Default for VisionTextCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dir: String::new(),
            signature_grid_size: 8,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct VisionTextCacheRuntimeConfig {
    pub enabled: bool,
    pub dir: Option<PathBuf>,
    pub signature_grid_size: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_fallback_dir_and_nonzero_grid_for_enabled_cache() {
        let config = VisionTextCacheConfig {
            enabled: true,
            dir: String::new(),
            signature_grid_size: 0,
        };

        assert_eq!(
            config.to_runtime_config(PathBuf::from("cache")),
            VisionTextCacheRuntimeConfig {
                enabled: true,
                dir: Some(PathBuf::from("cache")),
                signature_grid_size: 1,
            }
        );
    }
}
