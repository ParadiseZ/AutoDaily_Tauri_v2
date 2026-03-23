use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::infrastructure::hash_calculated::hash_error::{HashError, HashResult};
use crate::infrastructure::logging::log_trait::Log;
use std::hash::Hasher;
use twox_hash::XxHash3_64;
pub async fn get_hasher(model_path: &str) -> HashResult<u64> {
    if !PathBuf::from(model_path).exists() {
        Log::error(&format!("文件{}不存在，计算hash错误！", model_path));
        return Err(HashError::FileNotFound {
            path: model_path.to_string(),
        });
    }

    let mut file = File::open(&model_path)
        .await
        .map_err(|_| HashError::FileNotFound { path: model_path.to_string() })?;

    // 缓冲区大小，可以根据需要调整
    let mut buffer = [0u8; 8192];
    let mut hasher = XxHash3_64::default();
    // 逐块读取文件并更新哈希
    loop {
        let bytes_read = file.read(&mut buffer).await
            .map_err(|_| HashError::FileReadFailed { path: model_path.to_string()})?;
        if bytes_read == 0 {
            break; // 文件读取完毕
        }
        hasher.write(&buffer[..bytes_read]);
    }
    Ok(hasher.finish())
}

/*// 支持自定义种子的包装器
pub struct XxHash64WithSeed(u64);

impl XxHash64WithSeed {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
}

impl Hasher for XxHash64WithSeed {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hasher = XxHash64::with_seed(self.0);
        hasher.write(bytes);
        self.0 = hasher.finish(); // 使用上一次的哈希结果作为下一次的种子
    }
}*/
