use crate::domain::vision::result::OcrResult;
use aho_corasick::{AhoCorasick, MatchKind};
use serde::{Deserialize, Serialize};

/// 搜索结果，包含命中的文本及其对应的 OCR 信息
#[derive(Debug, Clone)]
pub struct SearchHit<'a> {
    pub pattern: String,
    pub start: usize,
    pub end: usize,
    pub ocr_results: Vec<&'a OcrResult>,
}

/// OCR 搜索引擎
pub struct OcrSearcher {
    patterns: Vec<String>,
    ac: AhoCorasick,
}

impl OcrSearcher {
    /// 创建一个新的搜索引擎
    /// patterns: 需要匹配的所有关键字集合
    pub fn new(patterns: Vec<String>) -> Self {
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostFirst)
            .build(&patterns)
            .expect("Failed to build AhoCorasick");
        
        Self { patterns, ac }
    }

    /// 在给定的 OCR 结果列表上执行搜索
    pub fn search<'a>(&self, results: &'a [OcrResult]) -> Vec<SearchHit<'a>> {
        if results.is_empty() {
            return Vec::new();
        }

        // 1. 构建大的文本缓冲区和索引映射
        // mapping 存储 (缓冲区起始偏移量, OcrResult 索引)
        let mut buffer = String::new();
        let mut mapping: Vec<(usize, usize)> = Vec::with_capacity(results.len());

        for (idx, res) in results.iter().enumerate() {
            mapping.push((buffer.len(), idx));
            buffer.push_str(&res.txt);
            buffer.push('\n'); // 增加换行符，防止不同行的文字首尾拼接成莫名其妙的词
        }

        // 2. 执行多模式匹配
        let mut hits = Vec::new();
        for mat in self.ac.find_iter(&buffer) {
            let pattern = self.patterns[mat.pattern()].clone();
            let start = mat.start();
            let end = mat.end();
            
            // 3. 将缓冲区坐标映射回 OcrResult
            // 使用二分查找找到命中起始位置所在的 OcrResult 索引
            let ocr_indices = self.map_range_to_ocr_indices(start, end, &mapping, results.len(), &buffer);
            
            let mut matched_ocr = Vec::new();
            for idx in ocr_indices {
                matched_ocr.push(&results[idx]);
            }

            hits.push(SearchHit {
                pattern,
                start,
                end,
                ocr_results: matched_ocr,
            });
        }

        hits
    }

    /// 将缓冲区中的起始/结束偏移量映射到 OcrResult 的索引列表（跨行处理）
    fn map_range_to_ocr_indices(
        &self,
        start: usize,
        end: usize,
        mapping: &[(usize, usize)],
        total_ocr: usize,
        _buffer: &str
    ) -> Vec<usize> {
        let mut indices = Vec::new();

        // 找到第一个包含或位于 start 之后的 OCR 结果
        let first_idx = mapping.binary_search_by_key(&start, |&(off, _)| off).unwrap_or_else(|idx| idx.saturating_sub(1));

        for i in first_idx..total_ocr {
            let (off, _) = mapping[i];
            if off >= end {
                break;
            }
            // 检查该 OCR 结果是否与 [start, end) 有交集
            // 这里简化处理：只要该 OCR 结果的起始位置在 end 之前，且其后的起始位置在 start 之后（或者它是最后一个）
            let next_off = mapping.get(i + 1).map(|m| m.0).unwrap_or(usize::MAX);
            
            if off < end && next_off > start {
                indices.push(i);
            }
        }

        indices
    }
}

/// 逻辑判定操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

/// 逻辑规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchRule {
    Keyword(String),
    Group {
        op: LogicOp,
        items: Vec<SearchRule>,
    },
}

impl SearchRule {
    /// 判定命中结果是否满足规则
    pub fn evaluate(&self, hits: &[SearchHit]) -> bool {
        match self {
            SearchRule::Keyword(k) => hits.iter().any(|h| &h.pattern == k),
            SearchRule::Group { op, items } => match op {
                LogicOp::And => items.iter().all(|r| r.evaluate(hits)),
                LogicOp::Or => items.iter().any(|r| r.evaluate(hits)),
                LogicOp::Not => {
                    if items.len() != 1 {
                        // NOT 通常只作用于一个子项
                        false
                    } else {
                        !items[0].evaluate(hits)
                    }
                }
            },
        }
    }

    /// 获取规则中涉及的所有关键字，用于初始化 OcrSearcher
    pub fn get_all_keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();
        self.collect_keywords(&mut keywords);
        keywords.sort();
        keywords.dedup();
        keywords
    }

    fn collect_keywords(&self, keywords: &mut Vec<String>) {
        match self {
            SearchRule::Keyword(k) => keywords.push(k.clone()),
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

    fn mock_ocr(text: &str) -> OcrResult {
        OcrResult {
            id: 0,
            pre_id: 0,
            next_id: 0,
            bounding_box: BoundingBox { x1: 0, y1: 0, x2: 0, y2: 0 },
            txt: text.to_string(),
            score: vec![],
            index: vec![],
            txt_char: vec![],
        }
    }

    #[test]
    fn test_ocr_search() {
        let results = vec![
            mock_ocr("请点击确认"),
            mock_ocr("有新提示"),
            mock_ocr("取消操作"),
        ];

        let rule = SearchRule::Group {
            op: LogicOp::And,
            items: vec![
                SearchRule::Keyword("确认".to_string()),
                SearchRule::Keyword("提示".to_string()),
            ],
        };

        let searcher = OcrSearcher::new(rule.get_all_keywords());
        let hits = searcher.search(&results);
        
        assert!(rule.evaluate(&hits));
        
        // 测试 OR
        let rule_or = SearchRule::Group {
            op: LogicOp::Or,
            items: vec![
                SearchRule::Keyword("不存在".to_string()),
                SearchRule::Keyword("取消".to_string()),
            ],
        };
        assert!(rule_or.evaluate(&hits));

        // 测试 NOT
        let rule_not = SearchRule::Group {
            op: LogicOp::Not,
            items: vec![SearchRule::Keyword("不存在".to_string())],
        };
        assert!(rule_not.evaluate(&hits));
    }
}
