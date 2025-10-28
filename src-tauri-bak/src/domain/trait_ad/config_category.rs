use serde::{Deserialize, Serialize};

/// 配置类别标记 trait
pub trait ConfigCategory: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// 创建默认配置（同步版本）
    fn default() -> Self;
}