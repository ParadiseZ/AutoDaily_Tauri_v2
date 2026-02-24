use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult};
use crate::infrastructure::vision::vision_error::VisionResult;
use aho_corasick::AhoCorasick;
use image::RgbaImage;
use regex::Regex;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};


/// 文本快照：仅包含 OCR 文本的搜索缓冲区及元数据映射
/// YOLO 检测结果不参与文本搜索，由 SearchRule::evaluate 单独处理
#[derive(Debug)]
pub struct VisionSnapshot {
    /// 搜索缓冲区：仅包含 OCR 文本内容
    pub buffer: String,
    /// 字符偏移量到 OcrResult 索引的映射 (sorted by offset)
    pub offset_map: Vec<(usize, usize)>,
    /// OCR 结果集合
    pub ocr_items: Vec<OcrResult>,
    /// YOLO 检测结果集合（保留但不参与文本搜索）
    pub det_items: Vec<DetResult>,

    pub capture: Option<RgbaImage>,
}

impl VisionSnapshot {
    pub fn new(
        ocr_results: Vec<OcrResult>,
        det_results: Vec<DetResult>,
        image: Option<RgbaImage>,
    ) -> VisionResult<Self> {
        let mut buffer = String::new();
        let mut offset_map = Vec::new();

        // 仅将 OCR 文本写入搜索缓冲区
        for (idx, ocr) in ocr_results.iter().enumerate() {
            let start_offset = buffer.len();

            // 写入颜色标记 (如果有图像)
            /*if let Some(img) = image {
                if let Ok((bg, fg)) = ColorAnalyzer::analyze_box(img, &ocr.bounding_box) {
                    buffer.push_str(&format!("__BG:{}__ __FG:{}__ ", bg.as_str(), fg.as_str()));
                }
            }*/

            // 写入 OCR 文本内容
            buffer.push_str(&ocr.txt);
            buffer.push('\n'); // 换行作为分隔符
            offset_map.push((start_offset, idx));
        }

        Ok(Self {
            buffer,
            offset_map,
            ocr_items: ocr_results,
            det_items: det_results,
            capture: image,
        })
    }

    /// 根据字符偏移量查找关联的 OcrResult
    pub fn find_ocr_at(&self, offset: usize) -> Option<&OcrResult> {
        let idx = self.offset_map.binary_search_by(|(off, _)| off.cmp(&offset))
            .unwrap_or_else(|x| if x > 0 { x - 1 } else { 0 });

        self.offset_map.get(idx).map(|(_, item_idx)| &self.ocr_items[*item_idx])
    }
}

/// 逻辑判定操作符
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

/// 搜索作用域
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ts_rs::TS)]
#[ts(export)]
pub enum SearchScope {
    /// 全局匹配：只要画面中存在这些模式即可（不要求在同一个框内）
    Global,
    /// 元素匹配：要求所有子条件必须在同一个视觉元素（框）内满足
    Item,
}

/// 搜索规则定义
/// 结构体保留所有变体（Keyword、YoloIdx、Regex、Group）以支持前端展示和数据编辑。
/// 但 OcrSearcher 仅处理文本相关变体（Keyword、Regex），
/// YoloIdx 由 evaluate 方法通过 DetResult 列表单独判断。
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SearchRule {
    /// 关键字包含
    Keyword {
        pattern: String
    },
    /// yolo 标签匹配（不参与文本搜索，由 evaluate 单独处理）
    YoloIdx{
        idx: i32,
    },
    /// 正则匹配
    Regex {
        pattern: String,
        #[serde(skip)]
        #[ts(skip)]
        compiled: Option<Regex>,
    },
    /// 逻辑组
    Group {
        op: LogicOp,
        scope: SearchScope,
        items: Vec<SearchRule>,
    },
}

/// 文本搜索命中结果（仅包含 OCR 文本命中）
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SearchHit {
    pub pattern: String,
    pub ocr_item: OcrResult,
}

/// OCR 文本搜索器：仅处理文本类条件（Keyword、Regex），
/// 通过 Aho-Corasick 自动机实现多模式一次遍历。
pub struct OcrSearcher {
    automaton: AhoCorasick,
    patterns: Vec<String>,
    regexes: Vec<(String, Regex)>,
}

impl OcrSearcher {
    /// 从规则集中提取文本条件，构建搜索自动机。
    /// YoloIdx 变体会被跳过，不参与文本搜索。
    pub fn new(rules: &[SearchRule]) -> Self {
        let mut keywords = Vec::new();
        let mut regexes = Vec::new();

        fn collect(rule: &SearchRule, keywords: &mut Vec<String>, regexes: &mut Vec<(String, Regex)>) {
            match rule {
                SearchRule::Keyword { pattern } => keywords.push(pattern.clone()),
                SearchRule::YoloIdx { .. } => {
                    // YOLO 标签不参与文本搜索，跳过
                }
                SearchRule::Regex { pattern, .. } => {
                    if let Ok(re) = Regex::new(pattern) {
                        regexes.push((pattern.clone(), re));
                    }
                }
                SearchRule::Group { items, .. } => {
                    for item in items {
                        collect(item, keywords, regexes);
                    }
                }
            }
        }

        for rule in rules {
            collect(rule, &mut keywords, &mut regexes);
        }

        keywords.sort();
        keywords.dedup();

        let automaton = AhoCorasick::new(&keywords).unwrap();
        Self { automaton, patterns: keywords, regexes }
    }

    /// 在文本快照中搜索，返回所有文本命中结果
    pub fn search(&self, snapshot: &VisionSnapshot) -> Vec<SearchHit> {
        let mut hits = Vec::new();
        let mut seen = HashSet::new();

        // 1. Aho-Corasick 关键字匹配
        for mat in self.automaton.find_iter(&snapshot.buffer) {
            if let Some(ocr) = snapshot.find_ocr_at(mat.start()) {
                let pattern = self.patterns[mat.pattern().as_usize()].clone();
                if seen.insert((pattern.clone(), ocr.id)) {
                    hits.push(SearchHit {
                        pattern,
                        ocr_item: ocr.clone(),
                    });
                }
            }
        }

        // 2. 正则表达式匹配
        for (pattern_str, re) in &self.regexes {
            for mat in re.find_iter(&snapshot.buffer) {
                if let Some(ocr) = snapshot.find_ocr_at(mat.start()) {
                    if seen.insert((pattern_str.clone(), ocr.id)) {
                        hits.push(SearchHit {
                            pattern: pattern_str.clone(),
                            ocr_item: ocr.clone(),
                        });
                    }
                }
            }
        }

        hits
    }
}

impl SearchRule {
    /// 判定搜索结果是否满足规则。
    ///
    /// - `hits`: OcrSearcher 搜索到的文本命中结果
    /// - `det_results`: YOLO 检测结果列表，用于 YoloIdx 条件的判断
    ///
    /// 文本条件（Keyword、Regex）从 hits 中查找；
    /// YOLO 条件从 det_results 中查找；
    /// 两者在 Group 逻辑中可自由组合。
    pub fn evaluate(&self, hits: &[SearchHit], det_results: &[DetResult]) -> bool {
        match self {
            SearchRule::Keyword { pattern } => {
                hits.iter().any(|h| &h.pattern == pattern)
            }
            SearchRule::YoloIdx { idx } => {
                // 直接在 YOLO 检测结果中查找对应 index
                det_results.iter().any(|d| d.index == *idx )
            }
            SearchRule::Regex { pattern, .. } => {
                hits.iter().any(|h| &h.pattern == pattern)
            }
            SearchRule::Group { op, scope, items } => {
                match scope {
                    SearchScope::Global => {
                        // 全局模式：所有子项的 eval 基于 hits 全集和 det_results 全集
                        match op {
                            LogicOp::And => items.iter().all(|r| r.evaluate(hits, det_results)),
                            LogicOp::Or => items.iter().any(|r| r.evaluate(hits, det_results)),
                            LogicOp::Not => {
                                if items.is_empty() { true }
                                else { !items[0].evaluate(hits, det_results) }
                            }
                        }
                    }
                    SearchScope::Item => {
                        // 元素模式：需要存在至少一个 OCR 元素，其内部命中的模式集合满足逻辑
                        // 注意：YoloIdx 条件在 Item 模式下按 Global 逻辑处理，
                        // 因为 YOLO 检测结果与 OCR 文本元素无对应关系
                        use std::collections::HashMap;
                        let mut item_hits: HashMap<String, Vec<SearchHit>> = HashMap::new();
                        for hit in hits {
                            let key = hit.ocr_item.txt.clone();
                            item_hits.entry(key).or_default().push(hit.clone());
                        }

                        match op {
                            LogicOp::And => {
                                item_hits.values().any(|sub_hits| {
                                    items.iter().all(|r| r.evaluate(sub_hits, det_results))
                                })
                            }
                            LogicOp::Or => {
                                items.iter().any(|r| r.evaluate(hits, det_results))
                            }
                            LogicOp::Not => {
                                !item_hits.values().any(|sub_hits| {
                                    items.iter().any(|r| r.evaluate(sub_hits, det_results))
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
            SearchRule::YoloIdx { .. } => {},
            SearchRule::Regex { pattern, .. } => keywords.push(pattern.clone()),
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
    fn test_text_only_search() {
        let ocr = vec![
            OcrResult {
                id: 1, pre_id: 0, next_id: 0,
                bounding_box: BoundingBox::new(10, 10, 50, 30),
                txt: "Confirm".to_string(),
                score: vec![0.9], index: vec![0], txt_char: vec!["C".into()]
            },
            OcrResult {
                id: 2, pre_id: 0, next_id: 0,
                bounding_box: BoundingBox::new(60, 10, 120, 30),
                txt: "Cancel".to_string(),
                score: vec![0.85], index: vec![1], txt_char: vec!["C".into()]
            },
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
        let snapshot = VisionSnapshot::new(ocr, det.clone(), None).unwrap();

        // 验证缓冲区中不包含 YOLO 标记
        assert!(!snapshot.buffer.contains("__OBJ:"));
        assert!(snapshot.buffer.contains("Confirm"));
        assert!(snapshot.buffer.contains("Cancel"));

        // 规则：包含 "Confirm" 且存在 5 号 YOLO 目标
        let rule = SearchRule::Group {
            op: LogicOp::And,
            scope: SearchScope::Global,
            items: vec![
                SearchRule::Keyword { pattern: "Confirm".into() },
                SearchRule::YoloIdx { idx: 5 },
            ]
        };

        let searcher = OcrSearcher::new(&[rule.clone()]);
        let hits = searcher.search(&snapshot);

        // 文本搜索应命中 "Confirm"
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].pattern, "Confirm");

        // 综合评估应通过（文本 + YOLO 都满足）
        assert!(rule.evaluate(&hits, &det));
    }

    #[test]
    fn test_yolo_only_rule_no_text_hit() {
        let ocr = vec![];
        let det = vec![
            DetResult {
                id: 1, pre_id: 0, next_id: 0,
                bounding_box: BoundingBox::new(100, 100, 150, 150),
                index: 3,
                label: "icon".into(), score: 0.9
            }
        ];

        let snapshot = VisionSnapshot::new(ocr, det.clone(), None).unwrap();

        // 纯 YOLO 规则
        let rule = SearchRule::YoloIdx { idx: 3 };
        let searcher = OcrSearcher::new(&[rule.clone()]);
        let hits = searcher.search(&snapshot);

        // 文本搜索无命中
        assert!(hits.is_empty());
        // 但 YOLO 评估应通过
        assert!(rule.evaluate(&hits, &det));
    }

    #[test]
    fn test_not_yolo_rule() {
        let det = vec![];

        // NOT(存在3号目标) — 在没有检测结果时应为 true
        let rule = SearchRule::Group {
            op: LogicOp::Not,
            scope: SearchScope::Global,
            items: vec![SearchRule::YoloIdx { idx: 3 }],
        };

        assert!(rule.evaluate(&[], &det));
    }

    #[test]
    fn test_deduplication() {
        let ocr = vec![
            OcrResult {
                id: 1, pre_id: 0, next_id: 0,
                bounding_box: BoundingBox::new(10, 10, 100, 30),
                txt: "Hello Hello".to_string(), // Contains two "Hello"
                score: vec![0.9, 0.9], index: vec![0, 1], txt_char: vec!["H".into()]
            },
        ];
        let snapshot = VisionSnapshot::new(ocr, vec![], None).unwrap();

        let rule = SearchRule::Keyword { pattern: "Hello".into() };
        let searcher = OcrSearcher::new(&[rule]);
        let hits = searcher.search(&snapshot);

        // Should only have 1 hit despite "Hello" appearing twice in the same box
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].pattern, "Hello");
        assert_eq!(hits[0].ocr_item.id, 1);
    }
}
