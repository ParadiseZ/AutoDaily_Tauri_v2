use std::path::PathBuf;
use sha3::{Digest, Sha3_224, Sha3_256};
use sha3::digest::DynDigest;
use tokio::fs::File;
use tokio::io::BufReader;
use crate::infrastructure::hash_calculated::hash_error::{HashError, HashResult};
use crate::infrastructure::logging::log_trait::Log;

macro_rules! compute_hash {
    ($reader:expr, $hasher:ty) => {{
        let mut hasher = <$hasher>::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = $reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }};
}


pub fn get_hasher(model_path: &str, length: usize) -> HashResult<String>{
    if !PathBuf::from(model_path).exists(){
        Log::error(&format!("文件{}不存在，计算hash错误！", model_path.display()));
        return Err(HashError::FileNotFound {path: model_path})
    }
    let file = File::open(&model_path)?;
    let mut reader = BufReader::new(file);
    match length {
        224 => compute_hash!(reader, Sha3_224),
        256 => compute_hash!(reader, Sha3_256),
        //384 => compute_hash!(reader, Sha3_384),
        //512 => compute_hash!(reader, Sha3_512),
        _ => {
            Log::warn("不支持的hash长度限制：【224, 256】");
            compute_hash!(reader, Sha3_224);
        },
    }
}