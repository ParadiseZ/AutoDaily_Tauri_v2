use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
use crate::domain::vision::result::{BoundingBox, OcrResult};
use crate::infrastructure::core::{Deserialize, Error, HashMap, ScriptId, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
}

pub type TextRecCacheResult<T> = Result<T, TextRecCacheError>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextRecCacheEntry {
    pub key: String,
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub txt: String,
}

impl TextRecCacheEntry {
    pub fn cache_key(&self) -> &str {
        &self.key
    }

    pub fn to_ocr_result(&self) -> OcrResult {
        OcrResult::new(
            BoundingBox::new(
                self.x1 as i32,
                self.y1 as i32,
                self.x2 as i32,
                self.y2 as i32,
            ),
            self.txt.clone(),
            Vec::new(),
            Vec::new(),
            1,
        )
    }

    pub fn to_line(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            quote_field(&self.key),
            self.x1,
            self.y1,
            self.x2,
            self.y2,
            quote_field(&self.txt)
        )
    }

    pub fn parse_line(path: &Path, line_no: usize, line: &str) -> TextRecCacheResult<Self> {
        let fields = split_cache_line(path, line_no, line)?;
        if fields.len() != 6 {
            return Err(TextRecCacheError::ParseFailed {
                path: path.display().to_string(),
                e: format!(
                    "第 {} 行字段数量错误，期望 6 个，实际 {}",
                    line_no,
                    fields.len()
                ),
            });
        }

        Ok(Self {
            key: parse_quoted_field(path, line_no, &fields[0], "key")?,
            x1: parse_u32_field(path, line_no, &fields[1], "x1")?,
            y1: parse_u32_field(path, line_no, &fields[2], "y1")?,
            x2: parse_u32_field(path, line_no, &fields[3], "x2")?,
            y2: parse_u32_field(path, line_no, &fields[4], "y2")?,
            txt: parse_quoted_field(path, line_no, &fields[5], "txt")?,
        })
    }

    fn from_ocr_result(cache_key: String, ocr_result: OcrResult) -> Self {
        Self {
            key: cache_key,
            x1: ocr_result.bounding_box.x1.max(0) as u32,
            y1: ocr_result.bounding_box.y1.max(0) as u32,
            x2: ocr_result.bounding_box.x2.max(0) as u32,
            y2: ocr_result.bounding_box.y2.max(0) as u32,
            txt: ocr_result.txt,
        }
    }
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
    entries: Vec<TextRecCacheEntry>,
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

        let file_path = self.resolve_cache_file_path(script_id, script_name)?;
        let entries = if file_path.exists() {
            self.read_entries(&file_path)?
        } else {
            Vec::new()
        };

        self.current_file_path = Some(file_path);
        self.current_script_id = Some(script_id);
        self.entries = entries;
        self.session_stats.clear();
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
        if !self.dirty {
            return Ok(());
        }

        ensure_parent_dir(&path)?;
        let text = self
            .entries
            .iter()
            .map(TextRecCacheEntry::to_line)
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&path, text).map_err(|e| TextRecCacheError::WriteFailed {
            path: path.display().to_string(),
            e: e.to_string(),
        })?;

        self.dirty = false;
        Ok(())
    }

    pub fn find_entry(&mut self, cache_key: &str) -> Option<OcrResult> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.cache_key() == cache_key)
            .map(TextRecCacheEntry::to_ocr_result);

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
        ocr_result: OcrResult,
    ) -> TextRecCacheResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        let cache_key = cache_key.into();

        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.cache_key() == cache_key)
        {
            *entry = TextRecCacheEntry::from_ocr_result(cache_key.clone(), ocr_result);
        } else {
            self.entries.push(TextRecCacheEntry::from_ocr_result(
                cache_key.clone(),
                ocr_result,
            ));
        }

        self.session_stats.entry(cache_key).or_default().write_count += 1;
        self.dirty = true;
        Ok(())
    }

    pub fn current_entries(&self) -> &[TextRecCacheEntry] {
        &self.entries
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

        Ok(dir.join(format!(
            "{}-{}.txt",
            sanitize_script_file_name(script_name),
            short_script_id(script_id)
        )))
    }

    fn read_entries(&self, path: &Path) -> TextRecCacheResult<Vec<TextRecCacheEntry>> {
        let text = fs::read_to_string(path).map_err(|e| TextRecCacheError::ReadFailed {
            path: path.display().to_string(),
            e: e.to_string(),
        })?;

        let mut entries = Vec::new();
        for (index, raw_line) in text.lines().enumerate() {
            let line_no = index + 1;
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            entries.push(TextRecCacheEntry::parse_line(path, line_no, line)?);
        }
        Ok(entries)
    }

    fn reset_runtime_state(&mut self) {
        self.current_file_path = None;
        self.current_script_id = None;
        self.entries.clear();
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

fn quote_field(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

fn parse_u32_field(
    path: &Path,
    line_no: usize,
    value: &str,
    field: &str,
) -> TextRecCacheResult<u32> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|error| TextRecCacheError::ParseFailed {
            path: path.display().to_string(),
            e: format!("第 {} 行字段 {} 不是合法 u32: {}", line_no, field, error),
        })
}

fn parse_quoted_field(
    path: &Path,
    line_no: usize,
    value: &str,
    field: &str,
) -> TextRecCacheResult<String> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(TextRecCacheError::ParseFailed {
            path: path.display().to_string(),
            e: format!("第 {} 行字段 {} 缺少引号包裹", line_no, field),
        });
    }

    let mut output = String::new();
    let mut escaped = false;
    for ch in value[1..value.len() - 1].chars() {
        if escaped {
            output.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        output.push(ch);
    }

    if escaped {
        return Err(TextRecCacheError::ParseFailed {
            path: path.display().to_string(),
            e: format!("第 {} 行字段 {} 以非法转义结尾", line_no, field),
        });
    }

    Ok(output)
}

fn split_cache_line(path: &Path, line_no: usize, line: &str) -> TextRecCacheResult<Vec<String>> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for ch in line.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_quotes => {
                current.push(ch);
                escaped = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err(TextRecCacheError::ParseFailed {
            path: path.display().to_string(),
            e: format!("第 {} 行存在未闭合引号", line_no),
        });
    }

    fields.push(current.trim().to_string());
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::vision_cache_conf::VisionTextCacheRuntimeConfig;
    use crate::domain::vision::result::{BoundingBox, OcrResult};
    use std::path::PathBuf;

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

    #[test]
    fn record_and_find_single_ocr_entry() {
        let mut cache = ScriptTextRecCacheRuntime::new(VisionTextCacheRuntimeConfig {
            enabled: true,
            dir: Some(PathBuf::from(".")),
            signature_grid_size: 8,
        });
        cache.current_file_path = Some(PathBuf::from("script-1.txt"));
        cache.current_script_id = Some("script-id".into());

        let result = OcrResult::new(
            BoundingBox::new(1, 2, 30, 12),
            "cache".to_string(),
            vec![0.123_456],
            vec![1],
            8,
        );

        cache
            .record_entry("1:abcd", result.clone())
            .expect("record should succeed");

        let cached = cache
            .find_entry("1:abcd")
            .expect("cache entry should exist");
        assert_eq!(cached.bounding_box, result.bounding_box);
        assert_eq!(cached.txt, result.txt);
        assert!(cached.score.is_empty());
        assert!(cached.index.is_empty());
        assert_eq!(cache.find_entry("missing"), None);
    }

    #[test]
    fn cache_line_roundtrip_preserves_quotes_and_commas() {
        let entry = TextRecCacheEntry {
            key: "1:05d784fdefccc1f9".to_string(),
            x1: 1,
            y1: 2,
            x2: 3,
            y2: 4,
            txt: "a,b\"c\\d".to_string(),
        };

        let line = entry.to_line();
        let parsed = TextRecCacheEntry::parse_line(Path::new("test.txt"), 1, &line)
            .expect("line should parse");
        assert_eq!(parsed, entry);
    }
}
