use crate::{
    DetResult, OcrResult, SearchHit, SearchRule, StablePoint, VisionLayoutItem, VisionLayoutSource,
};
use aho_corasick::AhoCorasick;
use std::cmp::Ordering;
use std::collections::HashSet;

type VisionResult<T> = Result<T, String>;

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
    /// 按稳定坐标聚合排序后的视觉条目，用于位置关系查询与后续签名生成。
    pub layout_items: Vec<VisionLayoutItem>,
    /// 位置比较使用的网格挡位。
    pub signature_grid_size: u16,
}

impl VisionSnapshot {
    pub fn new(det_results: Vec<DetResult>, signature_grid_size: u16) -> VisionResult<Self> {
        let grid_size = signature_grid_size.max(1);
        let mut snapshot = Self {
            buffer: String::new(),
            offset_map: Vec::new(),
            layout_items: Vec::new(),
            ocr_items: Vec::new(),
            det_items: Vec::new(),
            signature_grid_size: grid_size,
        };
        snapshot.set_det_results(det_results);
        Ok(snapshot)
    }

    pub fn with_ocr_results(mut self, ocr_results: Vec<OcrResult>) -> VisionResult<Self> {
        self.set_ocr_results(ocr_results)?;
        Ok(self)
    }

    pub fn set_ocr_results(&mut self, mut ocr_results: Vec<OcrResult>) -> VisionResult<()> {
        normalize_ocr_items(&mut ocr_results, self.signature_grid_size);
        ocr_results.sort_by(compare_ocr_items);
        let (buffer, offset_map) = build_ocr_buffer(&ocr_results);
        self.layout_items = build_layout_items(&ocr_results, &self.det_items);
        self.buffer = buffer;
        self.offset_map = offset_map;
        self.ocr_items = ocr_results;
        Ok(())
    }

    pub fn set_det_results(&mut self, mut det_results: Vec<DetResult>) {
        normalize_det_items(&mut det_results, self.signature_grid_size);
        det_results.sort_by(compare_det_items);
        self.layout_items = build_layout_items(&self.ocr_items, &det_results);
        self.det_items = det_results;
    }
}

fn find_ocr_at<'a>(
    offset_map: &[(usize, usize)],
    ocr_items: &'a [OcrResult],
    offset: usize,
) -> Option<(usize, &'a OcrResult)> {
    let idx = offset_map
        .binary_search_by(|(off, _)| off.cmp(&offset))
        .unwrap_or_else(|x| if x > 0 { x - 1 } else { 0 });

    offset_map
        .get(idx)
        .and_then(|(_, item_idx)| ocr_items.get(*item_idx).map(|item| (*item_idx, item)))
}

fn normalize_ocr_items(items: &mut [OcrResult], grid_size: u16) {
    for item in items {
        item.stable_box = item.bounding_box.to_stable_box(grid_size);
        item.stable_center = item.bounding_box.to_stable_center(grid_size);
    }
}

fn normalize_det_items(items: &mut [DetResult], grid_size: u16) {
    for item in items {
        item.stable_box = item.bounding_box.to_stable_box(grid_size);
        item.stable_center = item.bounding_box.to_stable_center(grid_size);
    }
}

fn build_ocr_buffer(ocr_results: &[OcrResult]) -> (String, Vec<(usize, usize)>) {
    let mut buffer = String::new();
    let mut offset_map = Vec::new();

    for (idx, ocr) in ocr_results.iter().enumerate() {
        let start_offset = buffer.len();
        buffer.push_str(&ocr.txt);
        buffer.push('\n');
        offset_map.push((start_offset, idx));
    }

    (buffer, offset_map)
}

/// OCR 文本搜索器：仅处理第一阶段 OCR 文本召回条件，
/// 通过 Aho-Corasick 自动机实现多模式一次遍历。
pub struct OcrSearcher {
    automaton: Option<AhoCorasick>,
    patterns: Vec<String>,
}

impl OcrSearcher {
    /// 从规则集中提取文本条件，构建搜索自动机。
    /// DetLabel 变体会被跳过，不参与文本搜索。
    pub fn new(rules: &[SearchRule]) -> Self {
        let mut keywords: Vec<String> = rules
            .iter()
            .flat_map(SearchRule::get_all_keywords)
            .collect();

        keywords.sort();
        keywords.dedup();

        let automaton = if keywords.is_empty() {
            None
        } else {
            Some(AhoCorasick::new(&keywords).unwrap())
        };
        Self {
            automaton,
            patterns: keywords,
        }
    }

    /// 在文本快照中搜索，返回所有文本命中结果
    pub fn search(&self, snapshot: &VisionSnapshot) -> Vec<SearchHit> {
        self.search_buffer(&snapshot.buffer, &snapshot.offset_map, &snapshot.ocr_items)
    }

    /// 在独立 OCR 步骤输出中搜索，不创建或更新帧快照。
    pub fn search_ocr_items(&self, ocr_items: &[OcrResult]) -> Vec<SearchHit> {
        let (buffer, offset_map) = build_ocr_buffer(ocr_items);
        self.search_buffer(&buffer, &offset_map, ocr_items)
    }

    fn search_buffer(
        &self,
        buffer: &str,
        offset_map: &[(usize, usize)],
        ocr_items: &[OcrResult],
    ) -> Vec<SearchHit> {
        let mut hits = Vec::new();
        let mut seen = HashSet::new();

        // 1. Aho-Corasick 关键字匹配
        if let Some(automaton) = &self.automaton {
            for mat in automaton.find_iter(buffer) {
                if let Some((ocr_index, ocr)) = find_ocr_at(offset_map, ocr_items, mat.start()) {
                    let pattern = self.patterns[mat.pattern().as_usize()].clone();
                    if seen.insert((pattern.clone(), ocr_index)) {
                        hits.push(SearchHit {
                            pattern,
                            ocr_index,
                            ocr_item: ocr.clone(),
                        });
                    }
                }
            }
        }

        hits
    }
}

fn compare_ocr_items(left: &OcrResult, right: &OcrResult) -> Ordering {
    compare_layout(
        &left.stable_center,
        &right.stable_center,
        &left.txt,
        &right.txt,
    )
}

fn compare_det_items(left: &DetResult, right: &DetResult) -> Ordering {
    compare_layout(
        &left.stable_center,
        &right.stable_center,
        &left.label,
        &right.label,
    )
}

fn compare_layout(
    left: &StablePoint,
    right: &StablePoint,
    left_tail: &str,
    right_tail: &str,
) -> Ordering {
    left.y
        .cmp(&right.y)
        .then_with(|| left.x.cmp(&right.x))
        .then_with(|| left_tail.cmp(right_tail))
}

fn build_layout_items(ocr_items: &[OcrResult], det_items: &[DetResult]) -> Vec<VisionLayoutItem> {
    let mut items = Vec::with_capacity(ocr_items.len() + det_items.len());
    for (index, item) in ocr_items.iter().enumerate() {
        items.push(VisionLayoutItem {
            source: VisionLayoutSource::Ocr,
            item_index: index,
            bounding_box: item.bounding_box.clone(),
            stable_box: item.stable_box.clone(),
            stable_center: item.stable_center.clone(),
            text: Some(item.txt.clone()),
            label: None,
            label_index: None,
        });
    }
    for (index, item) in det_items.iter().enumerate() {
        items.push(VisionLayoutItem {
            source: VisionLayoutSource::Det,
            item_index: index,
            bounding_box: item.bounding_box.clone(),
            stable_box: item.stable_box.clone(),
            stable_center: item.stable_center.clone(),
            text: None,
            label: Some(item.label.clone()),
            label_index: Some(item.index),
        });
    }
    items.sort_by(|left, right| {
        compare_layout(
            &left.stable_center,
            &right.stable_center,
            left.text.as_deref().or(left.label.as_deref()).unwrap_or(""),
            right
                .text
                .as_deref()
                .or(right.label.as_deref())
                .unwrap_or(""),
        )
        .then_with(|| left.source.cmp(&right.source))
        .then_with(|| left.item_index.cmp(&right.item_index))
    });
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoundingBox, LogicOp, SearchScope};

    #[test]
    fn test_text_only_search() {
        let ocr = vec![
            OcrResult::new(
                BoundingBox::new(10, 10, 50, 30),
                "Confirm".to_string(),
                vec![0.9],
                vec![0],
                8,
            ),
            OcrResult::new(
                BoundingBox::new(60, 10, 120, 30),
                "Cancel".to_string(),
                vec![0.85],
                vec![1],
                8,
            ),
        ];
        let det = vec![DetResult::new(
            BoundingBox::new(100, 100, 150, 150),
            5,
            "button".into(),
            0.8,
            8,
        )];

        // 构建快照 (不带图像则不分析颜色)
        let snapshot = VisionSnapshot::new(det.clone(), 8)
            .unwrap()
            .with_ocr_results(ocr)
            .unwrap();

        // 验证缓冲区中不包含 YOLO 标记
        assert!(!snapshot.buffer.contains("__OBJ:"));
        assert!(snapshot.buffer.contains("Confirm"));
        assert!(snapshot.buffer.contains("Cancel"));

        // 规则：包含 "Confirm" 且存在 5 号 YOLO 目标
        let rule = SearchRule::Group {
            op: LogicOp::And,
            scope: SearchScope::Global,
            items: vec![
                SearchRule::Txt {
                    pattern: "Confirm".into(),
                },
                SearchRule::DetLabel { idx: 5 },
            ],
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
        let det = vec![DetResult::new(
            BoundingBox::new(100, 100, 150, 150),
            3,
            "icon".into(),
            0.9,
            8,
        )];

        let snapshot = VisionSnapshot::new(det.clone(), 8)
            .unwrap()
            .with_ocr_results(ocr)
            .unwrap();

        // 纯 YOLO 规则
        let rule = SearchRule::DetLabel { idx: 3 };
        let searcher = OcrSearcher::new(&[rule.clone()]);
        let hits = searcher.search(&snapshot);

        // 文本搜索无命中
        assert!(hits.is_empty());
        // 但 YOLO 评估应通过
        assert!(rule.evaluate(&hits, &det));
    }

    #[test]
    fn test_updating_det_results_preserves_ocr_snapshot() {
        let ocr = vec![OcrResult::new(
            BoundingBox::new(10, 10, 50, 30),
            "Confirm".to_string(),
            vec![0.9],
            vec![0],
            8,
        )];
        let mut snapshot = VisionSnapshot::new(Vec::new(), 8)
            .unwrap()
            .with_ocr_results(ocr)
            .unwrap();

        snapshot.set_det_results(vec![DetResult::new(
            BoundingBox::new(100, 100, 150, 150),
            5,
            "button".into(),
            0.8,
            8,
        )]);

        assert_eq!(snapshot.ocr_items.len(), 1);
        assert_eq!(snapshot.det_items.len(), 1);
        assert_eq!(snapshot.layout_items.len(), 2);
        assert!(snapshot.buffer.contains("Confirm"));
    }

    #[test]
    fn test_not_yolo_rule() {
        let det = vec![];

        // NOT(存在3号目标) — 在没有检测结果时应为 true
        let rule = SearchRule::Group {
            op: LogicOp::Not,
            scope: SearchScope::Global,
            items: vec![SearchRule::DetLabel { idx: 3 }],
        };

        let _ = VisionSnapshot::new(det.clone(), 8).unwrap();
        assert!(rule.evaluate(&[], &det));
    }

    #[test]
    fn test_deduplication() {
        let ocr = vec![OcrResult::new(
            BoundingBox::new(10, 10, 100, 30),
            "Hello Hello".to_string(),
            vec![0.9, 0.9],
            vec![0, 1],
            8,
        )];
        let snapshot = VisionSnapshot::new(vec![], 8)
            .unwrap()
            .with_ocr_results(ocr)
            .unwrap();

        let rule = SearchRule::Txt {
            pattern: "Hello".into(),
        };
        let searcher = OcrSearcher::new(&[rule]);
        let hits = searcher.search(&snapshot);

        // Should only have 1 hit despite "Hello" appearing twice in the same box
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].pattern, "Hello");
        assert_eq!(hits[0].ocr_index, 0);
    }

    #[test]
    fn test_search_ocr_items_without_snapshot() {
        let items = vec![OcrResult::new(
            BoundingBox::new(10, 10, 100, 30),
            "Confirm".to_string(),
            vec![0.9],
            vec![0],
            8,
        )];

        let hits = OcrSearcher::new(&[SearchRule::Txt {
            pattern: "Confirm".into(),
        }])
        .search_ocr_items(&items);

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].ocr_index, 0);
    }
}
