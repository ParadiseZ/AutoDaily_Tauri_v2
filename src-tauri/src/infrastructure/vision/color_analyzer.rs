use image::{DynamicImage, GenericImageView, Rgba};
use crate::infrastructure::vision::vision_error::VisionResult;
use crate::domain::vision::result::BoundingBox;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColorTag {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
    Black,
    White,
    Gray,
    Other,
}

impl ColorTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorTag::Red => "RED",
            ColorTag::Orange => "ORANGE",
            ColorTag::Yellow => "YELLOW",
            ColorTag::Green => "GREEN",
            ColorTag::Cyan => "CYAN",
            ColorTag::Blue => "BLUE",
            ColorTag::Purple => "PURPLE",
            ColorTag::Pink => "PINK",
            ColorTag::Black => "BLACK",
            ColorTag::White => "WHITE",
            ColorTag::Gray => "GRAY",
            ColorTag::Other => "OTHER",
        }
    }
}

pub struct ColorAnalyzer;

impl ColorAnalyzer {
    /// 分析给定区域的主导背景色和前景色
    /// 这里采用简化方案：采样区域像素并映射到 HSV 区间
    pub fn analyze_box(image: &DynamicImage, bbox: &BoundingBox) -> VisionResult<(ColorTag, ColorTag)> {
        let (img_w, img_h) = image.dimensions();
        
        // 边界限制
        let x1 = bbox.x1.max(0) as u32;
        let y1 = bbox.y1.max(0) as u32;
        let x2 = (bbox.x2 as u32).min(img_w - 1);
        let y2 = (bbox.y2 as u32).min(img_h - 1);

        if x1 >= x2 || y1 >= y2 {
            return Ok((ColorTag::Other, ColorTag::Other));
        }

        let width = x2 - x1;
        let height = y2 - y1;

        // 采样逻辑：1/3 宽高步长采样 (约 9-16 个点)
        let step_x = (width / 3).max(1);
        let step_y = (height / 3).max(1);

        let mut colors = Vec::new();
        for y in (y1..=y2).step_by(step_y as usize) {
            for x in (x1..=x2).step_by(step_x as usize) {
                if x < img_w && y < img_h {
                    colors.push(image.get_pixel(x, y));
                }
            }
        }

        if colors.is_empty() {
            return Ok((ColorTag::Other, ColorTag::Other));
        }

        // 简化模型：计算平均亮度/颜色或通过直方图找主导色
        // 这里我们找“最频繁”出现的颜色区间作为背景
        let tags: Vec<ColorTag> = colors.iter().map(|p| Self::rgb_to_tag(p)).collect();
        
        // 简单统计
        let bg_tag = Self::most_frequent(&tags);
        
        // 前景色逻辑：找一个逻辑上与背景差异最大的颜色，或者简单取第二频繁
        let fg_candidates: Vec<ColorTag> = tags.iter().cloned().filter(|t| t != &bg_tag).collect();
        let fg_tag = if fg_candidates.is_empty() {
            bg_tag.clone()
        } else {
            Self::most_frequent(&fg_candidates)
        };

        Ok((bg_tag, fg_tag))
    }

    fn rgb_to_tag(pixel: &Rgba<u8>) -> ColorTag {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        // 黑白灰判定 (基于亮度 L 和 饱和度 S)
        let s = if max == 0.0 { 0.0 } else { delta / max };

        if l < 0.15 { return ColorTag::Black; }
        if l > 0.85 && s < 0.1 { return ColorTag::White; }
        if s < 0.1 { return ColorTag::Gray; }

        // 色相 H
        let mut h = if delta == 0.0 {
            0.0
        } else if max == r {
            (g - b) / delta % 6.0
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };

        h *= 60.0;
        if h < 0.0 { h += 360.0; }

        if h < 20.0 || h >= 330.0 { ColorTag::Red }
        else if h < 45.0 { ColorTag::Orange }
        else if h < 75.0 { ColorTag::Yellow }
        else if h < 150.0 { ColorTag::Green }
        else if h < 200.0 { ColorTag::Cyan }
        else if h < 260.0 { ColorTag::Blue }
        else if h < 300.0 { ColorTag::Purple }
        else { ColorTag::Pink }
    }

    fn most_frequent(tags: &[ColorTag]) -> ColorTag {
        use std::collections::HashMap;
        let mut counts = HashMap::new();
        for t in tags {
            *counts.entry(t).or_insert(0) += 1;
        }
        counts.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(t, _)| t.clone())
            .unwrap_or(ColorTag::Other)
    }
}
