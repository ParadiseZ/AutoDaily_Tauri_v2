use crate::domain::vision::result::{BoundingBox, DetResult, OcrResult, StablePoint};
use crate::infrastructure::vision::vision_error::VisionResult;
use aho_corasick::AhoCorasick;
use image::RgbaImage;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum VisionLayoutSource {
    Ocr,
    Det,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct VisionLayoutItem {
    pub source: VisionLayoutSource,
    pub item_index: usize,
    pub bounding_box: BoundingBox,
    pub stable_box: BoundingBox,
    pub stable_center: StablePoint,
    pub text: Option<String>,
    pub label: Option<String>,
    pub label_index: Option<i32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeAnchorType {
    OcrText,
    DetLabel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeDirection {
    Left,
    LeftAbove,
    Right,
    RightAbove,
    Above,
    RightBelow,
    Below,
    LeftBelow,
    Near,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeTargetKind {
    OcrText,
    DetLabel,
    Any,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeValueType {
    Text,
    Number,
    FractionLeftNumber,
    FractionRightNumber,
    Label,
    LabelIndex,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeCompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Contains,
    NotContains,
}

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

    pub capture: Option<RgbaImage>,
}

impl VisionSnapshot {
    pub fn new(
        mut ocr_results: Vec<OcrResult>,
        mut det_results: Vec<DetResult>,
        image: Option<RgbaImage>,
        signature_grid_size: u16,
    ) -> VisionResult<Self> {
        let grid_size = signature_grid_size.max(1);
        for item in &mut ocr_results {
            item.stable_box = item.bounding_box.to_stable_box(grid_size);
            item.stable_center = item.bounding_box.to_stable_center(grid_size);
        }
        for item in &mut det_results {
            item.stable_box = item.bounding_box.to_stable_box(grid_size);
            item.stable_center = item.bounding_box.to_stable_center(grid_size);
        }

        ocr_results.sort_by(compare_ocr_items);
        det_results.sort_by(compare_det_items);

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
            layout_items: build_layout_items(&ocr_results, &det_results),
            ocr_items: ocr_results,
            det_items: det_results,
            signature_grid_size: grid_size,
            capture: image,
        })
    }

    /// 根据字符偏移量查找关联的 OcrResult
    pub fn find_ocr_at(&self, offset: usize) -> Option<(usize, &OcrResult)> {
        let idx = self
            .offset_map
            .binary_search_by(|(off, _)| off.cmp(&offset))
            .unwrap_or_else(|x| if x > 0 { x - 1 } else { 0 });

        self.offset_map
            .get(idx)
            .and_then(|(_, item_idx)| self.ocr_items.get(*item_idx).map(|item| (*item_idx, item)))
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

/// 搜索规则定义：仅用于第一阶段视觉召回。
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SearchRule {
    /// OCR 文本包含
    Txt { pattern: String },
    /// 检测标签匹配
    DetLabel { idx: i32 },
    /// 逻辑组
    Group {
        op: LogicOp,
        scope: SearchScope,
        items: Vec<SearchRule>,
    },
}

/// 第二阶段策略精判规则：只在具备视觉上下文时执行。
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PolicyConditionRule {
    Regex {
        pattern: String,
    },
    Relative {
        anchor_type: RelativeAnchorType,
        anchor_text: String,
        anchor_idx: i32,
        direction: RelativeDirection,
        target_kind: RelativeTargetKind,
        value_type: RelativeValueType,
        compare: RelativeCompareOp,
        value: String,
        #[serde(default)]
        max_offset_x: Option<i32>,
        #[serde(default)]
        max_offset_y: Option<i32>,
        #[serde(default)]
        target_index: Option<usize>,
    },
    Group {
        op: LogicOp,
        items: Vec<PolicyConditionRule>,
    },
}

/// 文本搜索命中结果（仅包含 OCR 文本命中）
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SearchHit {
    pub pattern: String,
    pub ocr_index: usize,
    pub ocr_item: OcrResult,
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
        let mut keywords = Vec::new();

        fn collect(rule: &SearchRule, keywords: &mut Vec<String>) {
            match rule {
                SearchRule::Txt { pattern } => keywords.push(pattern.clone()),
                SearchRule::DetLabel { .. } => {}
                SearchRule::Group { items, .. } => {
                    for item in items {
                        collect(item, keywords);
                    }
                }
            }
        }

        for rule in rules {
            collect(rule, &mut keywords);
        }

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
        let mut hits = Vec::new();
        let mut seen = HashSet::new();

        // 1. Aho-Corasick 关键字匹配
        if let Some(automaton) = &self.automaton {
            for mat in automaton.find_iter(&snapshot.buffer) {
                if let Some((ocr_index, ocr)) = snapshot.find_ocr_at(mat.start()) {
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

impl SearchRule {
    /// 判定搜索结果是否满足规则。
    ///
    /// - `hits`: OcrSearcher 搜索到的文本命中结果
    /// - `det_results`: 检测结果列表，用于 DetLabel 条件的判断
    ///
    /// 文本条件从 hits 中查找；
    /// 标签条件从 det_results 中查找；
    /// 两者在 Group 逻辑中可自由组合。
    pub fn evaluate(&self, hits: &[SearchHit], det_results: &[DetResult]) -> bool {
        match self {
            SearchRule::Txt { pattern } => hits.iter().any(|h| &h.pattern == pattern),
            SearchRule::DetLabel { idx } => det_results.iter().any(|d| d.index == *idx),
            SearchRule::Group { op, scope, items } => match scope {
                SearchScope::Global => match op {
                    LogicOp::And => items.iter().all(|r| r.evaluate(hits, det_results)),
                    LogicOp::Or => items.iter().any(|r| r.evaluate(hits, det_results)),
                    LogicOp::Not => {
                        if items.is_empty() {
                            true
                        } else {
                            !items[0].evaluate(hits, det_results)
                        }
                    }
                },
                SearchScope::Item => {
                    use std::collections::HashMap;
                    let mut item_hits: HashMap<String, Vec<SearchHit>> = HashMap::new();
                    for hit in hits {
                        let key = hit.ocr_item.txt.clone();
                        item_hits.entry(key).or_default().push(hit.clone());
                    }

                    match op {
                        LogicOp::And => item_hits.values().any(|sub_hits| {
                            items.iter().all(|r| r.evaluate(sub_hits, det_results))
                        }),
                        LogicOp::Or => items.iter().any(|r| r.evaluate(hits, det_results)),
                        LogicOp::Not => !item_hits.values().any(|sub_hits| {
                            items.iter().any(|r| r.evaluate(sub_hits, det_results))
                        }),
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
            SearchRule::Txt { pattern } => keywords.push(pattern.clone()),
            SearchRule::DetLabel { .. } => {}
            SearchRule::Group { items, .. } => {
                for item in items {
                    item.collect_keywords(keywords);
                }
            }
        }
    }
}

impl PolicyConditionRule {
    pub fn evaluate(&self, snapshot: &VisionSnapshot) -> bool {
        match self {
            PolicyConditionRule::Regex { pattern } => Regex::new(pattern)
                .ok()
                .map(|re| re.is_match(&snapshot.buffer))
                .unwrap_or(false),
            PolicyConditionRule::Relative {
                anchor_type,
                anchor_text,
                anchor_idx,
                direction,
                target_kind,
                value_type,
                compare,
                value,
                max_offset_x,
                max_offset_y,
                target_index,
            } => evaluate_relative_rule(
                snapshot,
                *anchor_type,
                anchor_text,
                *anchor_idx,
                *direction,
                *target_kind,
                *value_type,
                *compare,
                value,
                *max_offset_x,
                *max_offset_y,
                *target_index,
            ),
            PolicyConditionRule::Group { op, items } => match op {
                LogicOp::And => items.iter().all(|item| item.evaluate(snapshot)),
                LogicOp::Or => items.iter().any(|item| item.evaluate(snapshot)),
                LogicOp::Not => {
                    if items.is_empty() {
                        true
                    } else {
                        !items[0].evaluate(snapshot)
                    }
                }
            },
        }
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

fn evaluate_relative_rule(
    snapshot: &VisionSnapshot,
    anchor_type: RelativeAnchorType,
    anchor_text: &str,
    anchor_idx: i32,
    direction: RelativeDirection,
    target_kind: RelativeTargetKind,
    value_type: RelativeValueType,
    compare: RelativeCompareOp,
    value: &str,
    max_offset_x: Option<i32>,
    max_offset_y: Option<i32>,
    target_index: Option<usize>,
) -> bool {
    snapshot
        .layout_items
        .iter()
        .filter(|item| match_anchor(item, anchor_type, anchor_text, anchor_idx))
        .any(|anchor| {
            select_relative_target(
                snapshot,
                anchor,
                direction,
                target_kind,
                max_offset_x,
                max_offset_y,
                target_index,
            )
                .map(|candidate| compare_relative_value(candidate, value_type, compare, value))
                .unwrap_or(false)
        })
}

fn match_anchor(
    item: &VisionLayoutItem,
    anchor_type: RelativeAnchorType,
    anchor_text: &str,
    anchor_idx: i32,
) -> bool {
    match anchor_type {
        RelativeAnchorType::OcrText => {
            item.source == VisionLayoutSource::Ocr
                && item
                    .text
                    .as_deref()
                    .map(|text| text.contains(anchor_text))
                    .unwrap_or(false)
        }
        RelativeAnchorType::DetLabel => {
            item.source == VisionLayoutSource::Det && item.label_index == Some(anchor_idx)
        }
    }
}

fn select_relative_target<'a>(
    snapshot: &'a VisionSnapshot,
    anchor: &VisionLayoutItem,
    direction: RelativeDirection,
    target_kind: RelativeTargetKind,
    max_offset_x: Option<i32>,
    max_offset_y: Option<i32>,
    target_index: Option<usize>,
) -> Option<&'a VisionLayoutItem> {
    let mut candidates: Vec<_> = snapshot
        .layout_items
        .iter()
        .filter(|candidate| {
            candidate.item_index != anchor.item_index || candidate.source != anchor.source
        })
        .filter(|candidate| matches_target_kind(candidate, target_kind))
        .filter_map(|candidate| {
            relative_score(anchor, candidate, direction, max_offset_x, max_offset_y)
                .map(|score| (score, candidate))
        })
        .collect();
    candidates.sort_by(|left, right| left.0.cmp(&right.0));
    candidates
        .get(target_index.unwrap_or(0))
        .map(|(_, candidate)| *candidate)
}

fn matches_target_kind(candidate: &VisionLayoutItem, target_kind: RelativeTargetKind) -> bool {
    match target_kind {
        RelativeTargetKind::OcrText => candidate.source == VisionLayoutSource::Ocr,
        RelativeTargetKind::DetLabel => candidate.source == VisionLayoutSource::Det,
        RelativeTargetKind::Any => true,
    }
}

fn relative_score(
    anchor: &VisionLayoutItem,
    candidate: &VisionLayoutItem,
    direction: RelativeDirection,
    max_offset_x: Option<i32>,
    max_offset_y: Option<i32>,
) -> Option<(i32, i32)> {
    let dx = candidate.stable_center.x - anchor.stable_center.x;
    let dy = candidate.stable_center.y - anchor.stable_center.y;
    let abs_dx = dx.abs();
    let abs_dy = dy.abs();
    if max_offset_x.is_some_and(|max| abs_dx > max.max(0)) {
        return None;
    }
    if max_offset_y.is_some_and(|max| abs_dy > max.max(0)) {
        return None;
    }

    match direction {
        RelativeDirection::Right if dx > 0 => Some((dx, abs_dy)),
        RelativeDirection::Left if dx < 0 => Some((abs_dx, abs_dy)),
        RelativeDirection::Below if dy > 0 => Some((dy, abs_dx)),
        RelativeDirection::Above if dy < 0 => Some((abs_dy, abs_dx)),
        RelativeDirection::LeftAbove if dx < 0 && dy < 0 => Some((abs_dx + abs_dy, abs_dx)),
        RelativeDirection::RightAbove if dx > 0 && dy < 0 => Some((abs_dx + abs_dy, abs_dx)),
        RelativeDirection::RightBelow if dx > 0 && dy > 0 => Some((abs_dx + abs_dy, abs_dx)),
        RelativeDirection::LeftBelow if dx < 0 && dy > 0 => Some((abs_dx + abs_dy, abs_dx)),
        RelativeDirection::Near => Some((abs_dx + abs_dy, abs_dx.min(abs_dy))),
        _ => None,
    }
}

fn compare_relative_value(
    candidate: &VisionLayoutItem,
    value_type: RelativeValueType,
    compare: RelativeCompareOp,
    expected: &str,
) -> bool {
    match value_type {
        RelativeValueType::Text => compare_text(
            candidate.text.as_deref().unwrap_or_default(),
            compare,
            expected,
        ),
        RelativeValueType::Label => compare_text(
            candidate.label.as_deref().unwrap_or_default(),
            compare,
            expected,
        ),
        RelativeValueType::LabelIndex => compare_number(
            candidate.label_index.map(|item| item as f64),
            compare,
            expected,
        ),
        RelativeValueType::Number => compare_number(
            extract_number(candidate.text.as_deref().unwrap_or_default()),
            compare,
            expected,
        ),
        RelativeValueType::FractionLeftNumber => compare_number(
            extract_fraction_number(candidate.text.as_deref().unwrap_or_default(), true),
            compare,
            expected,
        ),
        RelativeValueType::FractionRightNumber => compare_number(
            extract_fraction_number(candidate.text.as_deref().unwrap_or_default(), false),
            compare,
            expected,
        ),
    }
}

fn compare_text(actual: &str, compare: RelativeCompareOp, expected: &str) -> bool {
    match compare {
        RelativeCompareOp::Eq => actual == expected,
        RelativeCompareOp::Ne => actual != expected,
        RelativeCompareOp::Contains => actual.contains(expected),
        RelativeCompareOp::NotContains => !actual.contains(expected),
        _ => false,
    }
}

fn compare_number(actual: Option<f64>, compare: RelativeCompareOp, expected: &str) -> bool {
    let Some(actual) = actual else {
        return false;
    };
    let Ok(expected) = expected.trim().parse::<f64>() else {
        return false;
    };

    match compare {
        RelativeCompareOp::Eq => actual == expected,
        RelativeCompareOp::Ne => actual != expected,
        RelativeCompareOp::Lt => actual < expected,
        RelativeCompareOp::Le => actual <= expected,
        RelativeCompareOp::Gt => actual > expected,
        RelativeCompareOp::Ge => actual >= expected,
        _ => false,
    }
}

fn extract_number(text: &str) -> Option<f64> {
    let normalized = text.trim().replace(',', "");
    if normalized.is_empty() {
        return None;
    }

    let matched = Regex::new(r"-?\d+(?:\.\d+)?").ok()?.find(&normalized)?;
    matched.as_str().parse::<f64>().ok()
}

fn extract_fraction_number(text: &str, left: bool) -> Option<f64> {
    let normalized = text.trim().replace(' ', "");
    let mut parts = normalized.split('/');
    let lhs = parts.next()?.parse::<f64>().ok()?;
    let rhs = parts.next()?.parse::<f64>().ok()?;
    if left {
        Some(lhs)
    } else {
        Some(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::vision::result::BoundingBox;

    #[test]
    fn test_text_only_search() {
        let ocr = vec![
            OcrResult::new(
                BoundingBox::new(10, 10, 50, 30),
                "Confirm".to_string(),
                vec![0.9],
                vec![0],
                vec!["C".into()],
                8,
            ),
            OcrResult::new(
                BoundingBox::new(60, 10, 120, 30),
                "Cancel".to_string(),
                vec![0.85],
                vec![1],
                vec!["C".into()],
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
        let snapshot = VisionSnapshot::new(ocr, det.clone(), None, 8).unwrap();

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

        let snapshot = VisionSnapshot::new(ocr, det.clone(), None, 8).unwrap();

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
    fn test_not_yolo_rule() {
        let det = vec![];

        // NOT(存在3号目标) — 在没有检测结果时应为 true
        let rule = SearchRule::Group {
            op: LogicOp::Not,
            scope: SearchScope::Global,
            items: vec![SearchRule::DetLabel { idx: 3 }],
        };

        let _ = VisionSnapshot::new(vec![], det.clone(), None, 8).unwrap();
        assert!(rule.evaluate(&[], &det));
    }

    #[test]
    fn test_deduplication() {
        let ocr = vec![OcrResult::new(
            BoundingBox::new(10, 10, 100, 30),
            "Hello Hello".to_string(),
            vec![0.9, 0.9],
            vec![0, 1],
            vec!["H".into()],
            8,
        )];
        let snapshot = VisionSnapshot::new(ocr, vec![], None, 8).unwrap();

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
    fn test_policy_condition_relative_rule_matches_number_on_right() {
        let ocr = vec![
            OcrResult::new(
                BoundingBox::new(10, 10, 70, 30),
                "结晶".to_string(),
                vec![0.9],
                vec![0],
                vec!["结".into()],
                8,
            ),
            OcrResult::new(
                BoundingBox::new(90, 10, 130, 30),
                "15".to_string(),
                vec![0.95],
                vec![1],
                vec!["1".into()],
                8,
            ),
        ];
        let snapshot = VisionSnapshot::new(ocr, vec![], None, 8).unwrap();
        let rule = PolicyConditionRule::Relative {
            anchor_type: RelativeAnchorType::OcrText,
            anchor_text: "结晶".into(),
            anchor_idx: 0,
            direction: RelativeDirection::Right,
            target_kind: RelativeTargetKind::OcrText,
            value_type: RelativeValueType::Number,
            compare: RelativeCompareOp::Gt,
            value: "5".into(),
            max_offset_x: None,
            max_offset_y: None,
            target_index: None,
        };

        assert!(rule.evaluate(&snapshot));
    }
}
