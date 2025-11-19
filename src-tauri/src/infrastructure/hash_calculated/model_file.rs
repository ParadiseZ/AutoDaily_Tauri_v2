use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::BufReader;

use twox_hash::XxHash64;
use std::hash::Hasher;
use crate::infrastructure::hash_calculated::hash_error::{HashError, HashResult};
use crate::infrastructure::logging::log_trait::Log;

macro_rules! compute_hash {
    ($reader:expr, $hasher:ty) => {{
        let mut hasher = <$hasher>::default();  // 使用 default() 而不是 new()
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = $reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.write(&buffer[..bytes_read]);  // 使用 write() 而不是 update()
        }

        Ok(format!("{:x}", hasher.finish()))  // 使用 finish() 而不是 finalize()
    }};
}


pub fn get_hasher(model_path: &str, length: usize) -> HashResult<String> {
    if !PathBuf::from(model_path).exists() {
        Log::error(&format!("文件{}不存在，计算hash错误！", model_path));
        return Err(HashError::FileNotFound { path: model_path.to_string() });
    }

    let file = File::open(&model_path)?;
    let mut reader = BufReader::new(file);

    match length {
        64 => compute_hash!(reader, XxHash64),  // twox_hash::XxHash64 输出64位
        // 如果需要其他位数的哈希，可以考虑使用：
        // 128 => compute_hash!(reader, twox_hash::XxHash128),
        _ => {
            Log::warn("不支持的hash长度限制，使用默认的64位xxhash");
            compute_hash!(reader, XxHash64)
        },
    }
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