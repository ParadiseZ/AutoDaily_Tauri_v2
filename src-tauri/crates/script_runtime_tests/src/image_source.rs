use crate::config::ImageSourceConfig;
use image::RgbaImage;
use std::path::{Path, PathBuf};

pub fn collect_image_paths(
    config_dir: &Path,
    config: &ImageSourceConfig,
) -> Result<Vec<PathBuf>, String> {
    let root = if config.path.is_absolute() {
        config.path.clone()
    } else {
        config_dir.join(&config.path)
    };
    if root.is_file() {
        validate_extension(&root, &config.extensions)?;
        return Ok(vec![root]);
    }
    if !root.is_dir() {
        return Err(format!("图像路径不存在: {}", root.display()));
    }

    let mut paths = Vec::new();
    collect_directory(&root, config, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err(format!("图像目录中没有匹配文件: {}", root.display()));
    }
    Ok(paths)
}

pub fn load_image(path: &Path) -> Result<RgbaImage, String> {
    image::open(path)
        .map(|image| image.to_rgba8())
        .map_err(|error| format!("读取图像失败[{}]: {error}", path.display()))
}

fn collect_directory(
    directory: &Path,
    config: &ImageSourceConfig,
    paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("读取图像目录失败[{}]: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取目录项失败: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            if config.recursive {
                collect_directory(&path, config, paths)?;
            }
            continue;
        }
        if has_supported_extension(&path, &config.extensions) {
            paths.push(path);
        }
    }
    Ok(())
}

fn validate_extension(path: &Path, extensions: &[String]) -> Result<(), String> {
    if has_supported_extension(path, extensions) {
        Ok(())
    } else {
        Err(format!("图像扩展名不在配置允许范围内: {}", path.display()))
    }
}

fn has_supported_extension(path: &Path, extensions: &[String]) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    extensions.iter().any(|candidate| {
        candidate
            .trim_start_matches('.')
            .eq_ignore_ascii_case(extension)
    })
}
