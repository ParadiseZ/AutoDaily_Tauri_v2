use base64::engine::general_purpose;
use base64::Engine;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tauri::command;
use tokio::fs;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "webp"];

fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|allowed| ext.eq_ignore_ascii_case(allowed))
        })
        .unwrap_or(false)
}

fn sanitize_file_name(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();

    if cleaned.is_empty() {
        "vision_capture".to_string()
    } else {
        cleaned
    }
}

fn strip_base64_prefix(value: &str) -> &str {
    value
        .split_once("base64,")
        .map(|(_, base64)| base64)
        .unwrap_or(value)
}

fn ensure_unique_path(mut path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }

    let stem = path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("vision_capture")
        .to_string();
    let ext = path
        .extension()
        .and_then(OsStr::to_str)
        .map(|value| format!(".{}", value))
        .unwrap_or_default();
    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();

    for index in 1..10_000 {
        let candidate = parent.join(format!("{}_{}{}", stem, index, ext));
        if !candidate.exists() {
            return candidate;
        }
    }

    path.set_file_name(format!(
        "{}_{}{}",
        stem,
        chrono::Local::now().format("%Y%m%d%H%M%S"),
        ext
    ));
    path
}

fn build_output_name(file_name: Option<&str>) -> String {
    let raw = file_name
        .map(sanitize_file_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            format!(
                "vision_capture_{}",
                chrono::Local::now().format("%Y%m%d%H%M%S")
            )
        });

    if Path::new(&raw).extension().is_some() {
        raw
    } else {
        format!("{}.png", raw)
    }
}

fn vision_stage_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("auto_daily").join("vision_lab");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建视觉测试缓存目录失败: {}", e))?;
    Ok(dir)
}

#[command]
pub async fn vision_list_image_files_cmd(dir_path: String) -> Result<Vec<String>, String> {
    let dir = PathBuf::from(&dir_path);
    if !dir.exists() {
        return Err(format!("目录不存在: {}", dir_path));
    }
    if !dir.is_dir() {
        return Err(format!("路径不是目录: {}", dir_path));
    }

    let mut files = std::fs::read_dir(&dir)
        .map_err(|e| format!("读取目录失败: {}", e))?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.is_file() && is_supported_image(path))
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    files.sort_by(|left, right| left.to_lowercase().cmp(&right.to_lowercase()));
    Ok(files)
}

#[command]
pub async fn vision_stage_capture_image_cmd(
    image_data: String,
    suggested_name: Option<String>,
) -> Result<String, String> {
    let stage_dir = vision_stage_dir()?;
    let file_name = build_output_name(suggested_name.as_deref());
    let file_path = ensure_unique_path(stage_dir.join(file_name));
    let bytes = general_purpose::STANDARD
        .decode(strip_base64_prefix(&image_data))
        .map_err(|e| format!("截图数据解码失败: {}", e))?;

    fs::write(&file_path, bytes)
        .await
        .map_err(|e| format!("写入缓存图片失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[command]
pub async fn vision_save_staged_image_cmd(
    staged_path: String,
    save_dir: String,
    file_name: Option<String>,
) -> Result<String, String> {
    let source = PathBuf::from(&staged_path);
    if !source.exists() || !source.is_file() {
        return Err(format!("暂存文件不存在: {}", staged_path));
    }

    let target_dir = PathBuf::from(&save_dir);
    std::fs::create_dir_all(&target_dir).map_err(|e| format!("创建保存目录失败: {}", e))?;

    let resolved_name = file_name
        .as_deref()
        .map(Some)
        .unwrap_or_else(|| source.file_name().and_then(OsStr::to_str));
    let target = ensure_unique_path(target_dir.join(build_output_name(resolved_name)));

    fs::copy(&source, &target)
        .await
        .map_err(|e| format!("保存图片失败: {}", e))?;

    Ok(target.to_string_lossy().to_string())
}
