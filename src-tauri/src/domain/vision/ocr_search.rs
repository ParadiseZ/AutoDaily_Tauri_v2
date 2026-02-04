use crate::domain::vision::result::{OcrResult, DetResult, BoundingBox};
use crate::infrastructure::vision::color_analyzer::ColorAnalyzer;
use crate::infrastructure::vision::vision_error::VisionResult;
use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use image::DynamicImage;
use regex::Regex;

/// 视觉对象的统一包装，用于搜索命中后的回溯
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisionItem {
    Ocr(OcrResult),
    Det(DetResult),
}

impl VisionItem {
    pub fn bounding_box(&self) -> &BoundingBox {
        match self {
            VisionItem::Ocr(r) => &r.bounding_box,
            VisionItem::Det(r) => &r.bounding_box,
        }
    }

    pub fn txt(&self) -> String {
        match self {
            VisionItem::Ocr(r) => r.txt.clone(),
            VisionItem::Det(r) => r.label.clone(),
        }
    }
}

/// 视觉快照：包含脱敏/标记后的搜索缓冲区及元数据映射
/// 每一帧图像处理后只需构建一次
#[derive(Debug)]
pub struct VisionSnapshot {
    /// 搜索缓冲区：`__BG:YEL__ __FG:BLK__ 确认 \n ...`
    pub buffer: String,
    /// 字符偏移量到 VisionItem 索引的映射 (sorted by offset)
    pub offset_map: Vec<(usize, usize)>,
    /// 原始视觉项集合
    pub items: Vec<VisionItem>,
}

impl VisionSnapshot {
    pub fn new(
        ocr_results: Vec<OcrResult>,
        det_results: Vec<DetResult>,
        image: Option<&DynamicImage>,
    ) -> VisionResult<Self> {
        let mut buffer = String::new();
        let mut offset_map = Vec::new();
        let mut items = Vec::new();

        // 1. 合并并包装所有原始结果
        for r in ocr_results { items.push(VisionItem::Ocr(r)); }
        for r in det_results { items.push(VisionItem::Det(r)); }

        // 2. 构建缓冲区
        for (idx, item) in items.iter().enumerate() {
            let start_offset = buffer.len();
            
            // 写入颜色标记 (如果有图像)
            if let Some(img) = image {
                if let Ok((bg, fg)) = ColorAnalyzer::analyze_box(img, item.bounding_box()) {
                    buffer.push_str(&format!("__BG:{}__ __FG:{}__ ", bg.as_str(), fg.as_str()));
                }
            }

            // 写入内容标记
            match item {
                VisionItem::Ocr(r) => {
                    buffer.push_str(&r.txt);
                }
                VisionItem::Det(r) => {
                    // YOLO 物体使用特定 ID 标记：__OBJ:index__
                    buffer.push_str(&format!("__OBJ:{}__", r.index));
                }
            }
            
            buffer.push('\n'); // 换行作为分隔符
            offset_map.push((start_offset, idx));
        }

        Ok(Self { buffer, offset_map, items })
    }

    /// 根据字符偏移量查找关联的 VisionItem
    pub fn find_item_at(&self, offset: usize) -> Option<&VisionItem> {
        let idx = self.offset_map.binary_search_by(|(off, _)| off.cmp(&offset))
            .unwrap_or_else(|x| if x > 0 { x - 1 } else { 0 });
        
        self.offset_map.get(idx).map(|(_, item_idx)| &self.items[*item_idx])
    }
}

/// 逻辑判定操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

/// 搜索作用域
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchScope {
    /// 全局匹配：只要画面中存在这些模式即可（不要求在同一个框内）
    Global,
    /// 元素匹配：要求所有子条件必须在同一个视觉元素（框）内满足
    Item,
}

/// 逻辑规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchRule {
    /// 关键字包含
    Keyword { 
        #[serde(rename = "text")]
        pattern: String 
    },
    /// 正则匹配
    Regex {
        pattern: String,
    },
    /// 逻辑组
    Group {
        op: LogicOp,
        scope: SearchScope,
        items: Vec<SearchRule>,
    },
}

/// 搜索命中结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub pattern: String,
    pub item: VisionItem,
}

pub struct OcrSearcher {
    automaton: AhoCorasick,
    patterns: Vec<String>,
}

impl OcrSearcher {
    pub fn new(keywords: Vec<String>) -> Self {
        let automaton = AhoCorasick::new(&keywords).unwrap();
        Self { automaton, patterns: keywords }
    }

    pub fn search(&self, snapshot: &VisionSnapshot) -> Vec<SearchHit> {
        let mut hits = Vec::new();
        for mat in self.automaton.find_iter(&snapshot.buffer) {
            if let Some(item) = snapshot.find_item_at(mat.start()) {
                hits.push(SearchHit {
                    pattern: self.patterns[mat.pattern().as_usize()].clone(),
                    item: item.clone(),
                });
            }
        }
        hits
    }
}

impl SearchRule {
    /// 判定判定结果集是否满足规则
    /// hits: 搜索到的所有命中所涉及的 pattern
    pub fn evaluate(&self, hits: &[SearchHit]) -> bool {
        match self {
            SearchRule::Keyword { pattern } => hits.iter().any(|h| &h.pattern == pattern),
            SearchRule::Regex { pattern } => {
                if let Ok(re) = Regex::new(pattern) {
                    hits.iter().any(|h| re.is_match(&h.pattern))
                } else {
                    false
                }
            }
            SearchRule::Group { op, scope, items } => {
                match scope {
                    SearchScope::Global => {
                        // 全局模式：所有子项的 eval 仅基于 hits 全集
                        match op {
                            LogicOp::And => items.iter().all(|r| r.evaluate(hits)),
                            LogicOp::Or => items.iter().any(|r| r.evaluate(hits)),
                            LogicOp::Not => {
                                if items.is_empty() { true }
                                else { !items[0].evaluate(hits) }
                            }
                        }
                    }
                    SearchScope::Item => {
                        // 元素模式：需要存在【至少一个视觉项】，其内部命中的模式集合满足逻辑
                        use std::collections::HashMap;
                        let mut item_hits: HashMap<String, Vec<SearchHit>> = HashMap::new();
                        for hit in hits {
                            // 使用 VisionItem 的 id 作为 key 来分组
                            let key = hit.item.txt();
                            item_hits.entry(key).or_default().push(hit.clone());
                        }

                        match op {
                            LogicOp::And => {
                                item_hits.values().any(|sub_hits| {
                                    items.iter().all(|r| r.evaluate(sub_hits))
                                })
                            }
                            LogicOp::Or => {
                                // 对于 OR，只要整体命中了其中一个即可（等同于 Global）
                                hits.iter().any(|h| {
                                    items.iter().any(|r| {
                                        if let SearchRule::Keyword { pattern } = r { h.pattern == *pattern }
                                        else { r.evaluate(hits) }
                                    })
                                })
                            }
                            LogicOp::Not => {
                                !item_hits.values().any(|sub_hits| {
                                    items.iter().any(|r| r.evaluate(sub_hits))
                                })
                            }
                        }
                    }
                }
            },
        }
    }

    pub fn get_all_keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();
        self.collect_keywords(&mut keywords);
        keywords.sort();
        keywords.dedup();
        keywords
    }

    fn collect_keywords(&self, keywords: &mut Vec<String>) {
        match self {
            SearchRule::Keyword { pattern } => keywords.push(pattern.clone()),
            SearchRule::Regex { pattern } => keywords.push(pattern.clone()),
            SearchRule::Group { items, .. } => {
                for item in items {
                    item.collect_keywords(keywords);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::vision::result::BoundingBox;

    #[test]
    fn test_fusion_search() {
        let ocr = vec![
            OcrResult {
                id: 1, pre_id: 0, next_id: 0,
                bounding_box: BoundingBox::new(10, 10, 50, 30),
                txt: "Confirm".to_string(),
                score: vec![0.9], index: vec![0], txt_char: vec!["C".into()]
            }
        ];
        let det = vec![
            DetResult {
                id: 1, pre_id: 0, next_id: 0,
                bounding_box: BoundingBox::new(100, 100, 150, 150),
                index: 5, // e.g. "button" label
                label: "button".into(), score: 0.8
            }
        ];

        // 构建快照 (不带图像则不分析颜色)
        let snapshot = VisionSnapshot::new(ocr, det, None).unwrap();
        
        // 规则：包含 "Confirm" 且包含 5号物体 (button)
        let rule = SearchRule::Group {
            op: LogicOp::And,
            scope: SearchScope::Global,
            items: vec![
                SearchRule::Keyword { pattern: "Confirm".into() },
                SearchRule::Keyword { pattern: "__OBJ:5__".into() },
            ]
        };

        let searcher = OcrSearcher::new(rule.get_all_keywords());
        let hits = searcher.search(&snapshot);
        
        assert!(rule.evaluate(&hits));
    }
}
