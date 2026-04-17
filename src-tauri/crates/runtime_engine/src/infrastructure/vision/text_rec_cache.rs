use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::core::{Deserialize, Error, HashMap, ScriptId, Serialize};
use crate::infrastructure::logging::log_trait::Log;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum TextRecCacheError {
    #[error("OCR 文字缓存未启用")]
    Disabled,

    #[error("OCR 文字缓存目录不可用")]
    MissingDir,

    #[error("创建 OCR 文字缓存目录失败: {path}, {e}")]
    CreateDirFailed { path: String, e: String },

    #[error("读取 OCR 文字缓存文件失败: {path}, {e}")]
    ReadFailed { path: String, e: String },

    #[error("写入 OCR 文字缓存文件失败: {path}, {e}")]
    WriteFailed { path: String, e: String },

    #[error("解析 OCR 文字缓存文件失败: {path}, {e}")]
    ParseFailed { path: String, e: String },

    #[error("序列化 OCR 文字缓存文件失败: {e}")]
    SerializeFailed { e: String },
}

pub type TextRecCacheResult<T> = Result<T, TextRecCacheError>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TextRecCacheEntry {
    pub cache_key: String,
    pub det_results: Vec<DetResult>,
    pub ocr_results: Vec<OcrResult>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TextRecCacheDocument {
    pub version: u8,
    pub script_id: String,
    pub script_name: String,
    pub updated_at: String,
    pub entries: Vec<TextRecCacheEntry>,
}

#[derive(Debug, Default, Clone)]
pub struct TextRecCacheSessionStats {
    pub hit_count: u32,
    pub write_count: u32,
}

#[derive(Debug, Default)]
pub struct ScriptTextRecCacheRuntime {
    config: VisionTextCacheRuntimeConfig,
    current_file_path: Option<PathBuf>,
    current_script_id: Option<ScriptId>,
    document: Option<TextRecCacheDocument>,
    session_stats: HashMap<String, TextRecCacheSessionStats>,
    dirty: bool,
}

impl ScriptTextRecCacheRuntime {
    pub fn new(config: VisionTextCacheRuntimeConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.config.dir.is_some()
    }

    pub fn has_active_script(&self) -> bool {
        self.current_script_id.is_some()
    }

    pub fn apply_config(&mut self, config: VisionTextCacheRuntimeConfig) {
        self.config = config;
        if !self.is_enabled() {
            self.reset_runtime_state();
        }
    }

    pub fn load_for_script(
        &mut self,
        script_id: ScriptId,
        script_name: &str,
    ) -> TextRecCacheResult<()> {
        if !self.is_enabled() {
            self.reset_runtime_state();
            return Ok(());
        }

        if self.dirty {
            if let Err(error) = self.flush_current_script() {
                Log::warn(&format!(
                    "OCR 文字缓存写回失败，已放弃旧脚本缓存状态: {}",
                    error
                ));
                self.reset_runtime_state();
            }
        }

        let file_path = self.resolve_cache_file_path(script_id, script_name)?;
        let document = if file_path.exists() {
            match self.read_document(&file_path) {
                Ok(document) => document,
                Err(error) => {
                    Log::warn(&format!(
                        "OCR 文字缓存文件读取失败，已回退为空缓存: {}",
                        error
                    ));
                    TextRecCacheDocument {
                        version: 1,
                        script_id: script_id.to_string(),
                        script_name: script_name.to_string(),
                        updated_at: unix_timestamp_string(),
                        entries: Vec::new(),
                    }
                }
            }
        } else {
            TextRecCacheDocument {
                version: 1,
                script_id: script_id.to_string(),
                script_name: script_name.to_string(),
                updated_at: unix_timestamp_string(),
                entries: Vec::new(),
            }
        };

        self.current_file_path = Some(file_path);
        self.current_script_id = Some(script_id);
        self.session_stats.clear();
        self.document = Some(document);
        self.dirty = false;
        Ok(())
    }

    pub fn flush_current_script(&mut self) -> TextRecCacheResult<()> {
        if !self.is_enabled() {
            self.reset_runtime_state();
            return Ok(());
        }

        let Some(path) = self.current_file_path.clone() else {
            return Ok(());
        };
        let Some(document) = self.document.as_mut() else {
            return Ok(());
        };
        if !self.dirty {
            return Ok(());
        }

        if document.entries.is_empty() {
            self.dirty = false;
            return Ok(());
        }

        ensure_parent_dir(&path)?;
        document.updated_at = unix_timestamp_string();

        let text = serde_json::to_string_pretty(document)
            .map_err(|e| TextRecCacheError::SerializeFailed { e: e.to_string() })?;

        fs::write(&path, text).map_err(|e| TextRecCacheError::WriteFailed {
            path: path.display().to_string(),
            e: e.to_string(),
        })?;

        self.dirty = false;
        Ok(())
    }

    pub fn find_entry(&mut self, cache_key: &str) -> Option<TextRecCacheEntry> {
        let entry = self
            .document
            .as_ref()?
            .entries
            .iter()
            .find(|entry| entry.cache_key == cache_key)
            .cloned();

        if entry.is_some() {
            self.session_stats
                .entry(cache_key.to_string())
                .or_default()
                .hit_count += 1;
        }

        entry
    }

    pub fn record_entry(
        &mut self,
        cache_key: impl Into<String>,
        det_results: Vec<DetResult>,
        ocr_results: Vec<OcrResult>,
    ) -> TextRecCacheResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let Some(document) = self.document.as_mut() else {
            return Ok(());
        };

        let cache_key = cache_key.into();
        let updated_at = unix_timestamp_string();

        if let Some(entry) = document
            .entries
            .iter_mut()
            .find(|entry| entry.cache_key == cache_key)
        {
            entry.det_results = det_results;
            entry.ocr_results = ocr_results;
            entry.updated_at = updated_at;
        } else {
            document.entries.push(TextRecCacheEntry {
                cache_key: cache_key.clone(),
                det_results,
                ocr_results,
                updated_at,
            });
        }

        self.session_stats.entry(cache_key).or_default().write_count += 1;
        self.dirty = true;
        Ok(())
    }

    pub fn current_document(&self) -> Option<&TextRecCacheDocument> {
        self.document.as_ref()
    }

    fn resolve_cache_file_path(
        &self,
        script_id: ScriptId,
        script_name: &str,
    ) -> TextRecCacheResult<PathBuf> {
        let dir = self
            .config
            .dir
            .as_ref()
            .ok_or(TextRecCacheError::MissingDir)?;
        fs::create_dir_all(dir).map_err(|e| TextRecCacheError::CreateDirFailed {
            path: dir.display().to_string(),
            e: e.to_string(),
        })?;

        let base_name = sanitize_script_file_name(script_name);
        let primary = dir.join(format!("{}.json", base_name));

        if !primary.exists() {
            return Ok(primary);
        }

        match self.read_document(&primary) {
            Ok(document) if document.script_id == script_id.to_string() => Ok(primary),
            Ok(_) => Ok(dir.join(format!("{}-{}.json", base_name, short_script_id(script_id)))),
            Err(error) => {
                Log::warn(&format!(
                    "OCR 文字缓存主文件读取失败，改用脚本 ID 后缀文件: {}",
                    error
                ));
                Ok(dir.join(format!("{}-{}.json", base_name, short_script_id(script_id))))
            }
        }
    }

    fn read_document(&self, path: &Path) -> TextRecCacheResult<TextRecCacheDocument> {
        let text = fs::read_to_string(path).map_err(|e| TextRecCacheError::ReadFailed {
            path: path.display().to_string(),
            e: e.to_string(),
        })?;

        serde_json::from_str::<TextRecCacheDocument>(&text).map_err(|e| {
            TextRecCacheError::ParseFailed {
                path: path.display().to_string(),
                e: e.to_string(),
            }
        })
    }

    fn reset_runtime_state(&mut self) {
        self.current_file_path = None;
        self.current_script_id = None;
        self.document = None;
        self.session_stats.clear();
        self.dirty = false;
    }
}

fn ensure_parent_dir(path: &Path) -> TextRecCacheResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| TextRecCacheError::CreateDirFailed {
            path: parent.display().to_string(),
            e: e.to_string(),
        })?;
    }
    Ok(())
}

fn unix_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn short_script_id(script_id: ScriptId) -> String {
    script_id.to_string().chars().take(8).collect::<String>()
}

fn sanitize_script_file_name(input: &str) -> String {
    let mut sanitized = input
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | ' ' => ch,
            _ => '_',
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .replace(' ', "_");

    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }

    if sanitized.is_empty() {
        sanitized = "script".to_string();
    }

    let upper = sanitized.to_ascii_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    if reserved.contains(&upper.as_str()) {
        format!("_{}", sanitized)
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_empty_file_name() {
        assert_eq!(sanitize_script_file_name(""), "script");
        assert_eq!(sanitize_script_file_name("   "), "script");
    }

    #[test]
    fn sanitize_windows_reserved_name() {
        assert_eq!(sanitize_script_file_name("con"), "_con");
        assert_eq!(sanitize_script_file_name("AUX"), "_AUX");
    }

    #[test]
    fn sanitize_illegal_chars() {
        assert_eq!(
            sanitize_script_file_name("Daily:Login/Run"),
            "Daily_Login_Run"
        );
    }
}
