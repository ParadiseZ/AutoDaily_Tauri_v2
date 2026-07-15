use crate::{BoundingBox, DetResult, OcrResult, StablePoint};
use std::collections::HashMap;

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    ts_rs::TS,
)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum VisionLayoutSource {
    Ocr,
    Det,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeAnchorType {
    OcrText,
    DetLabel,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum RelativeTargetKind {
    OcrText,
    DetLabel,
    Any,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, ts_rs::TS)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, ts_rs::TS)]
#[ts(export)]
pub enum SearchScope {
    Global,
    Item,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SearchRule {
    Txt {
        pattern: String,
    },
    DetLabel {
        idx: i32,
    },
    Group {
        op: LogicOp,
        scope: SearchScope,
        items: Vec<SearchRule>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SearchHit {
    pub pattern: String,
    pub ocr_index: usize,
    pub ocr_item: OcrResult,
}

impl SearchRule {
    pub fn evaluate(&self, hits: &[SearchHit], det_results: &[DetResult]) -> bool {
        match self {
            Self::Txt { pattern } => normalized_pattern(pattern)
                .is_some_and(|pattern| hits.iter().any(|hit| hit.pattern == pattern)),
            Self::DetLabel { idx } => det_results.iter().any(|result| result.index == *idx),
            Self::Group { op, scope, items } => match scope {
                SearchScope::Global => match op {
                    LogicOp::And => items.iter().all(|rule| rule.evaluate(hits, det_results)),
                    LogicOp::Or => items.iter().any(|rule| rule.evaluate(hits, det_results)),
                    LogicOp::Not => items
                        .first()
                        .is_none_or(|rule| !rule.evaluate(hits, det_results)),
                },
                SearchScope::Item => {
                    let mut item_hits: HashMap<&str, Vec<SearchHit>> = HashMap::new();
                    for hit in hits {
                        item_hits
                            .entry(&hit.ocr_item.txt)
                            .or_default()
                            .push(hit.clone());
                    }
                    match op {
                        LogicOp::And => item_hits.values().any(|subset| {
                            items.iter().all(|rule| rule.evaluate(subset, det_results))
                        }),
                        LogicOp::Or => items.iter().any(|rule| rule.evaluate(hits, det_results)),
                        LogicOp::Not => !item_hits.values().any(|subset| {
                            items.iter().any(|rule| rule.evaluate(subset, det_results))
                        }),
                    }
                }
            },
        }
    }

    pub(crate) fn get_all_keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();
        self.collect_keywords(&mut keywords);
        keywords.sort();
        keywords.dedup();
        keywords
    }

    fn collect_keywords(&self, keywords: &mut Vec<String>) {
        match self {
            Self::Txt { pattern } => {
                if let Some(pattern) = normalized_pattern(pattern) {
                    keywords.push(pattern.to_string());
                }
            }
            Self::DetLabel { .. } => {}
            Self::Group { items, .. } => {
                for item in items {
                    item.collect_keywords(keywords);
                }
            }
        }
    }
}

fn normalized_pattern(pattern: &str) -> Option<&str> {
    let pattern = pattern.trim();
    (!pattern.is_empty()).then_some(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_deduplicates_nested_keywords() {
        let rule = SearchRule::Group {
            op: LogicOp::And,
            scope: SearchScope::Global,
            items: vec![
                SearchRule::Txt {
                    pattern: " a ".to_string(),
                },
                SearchRule::Txt {
                    pattern: "a".to_string(),
                },
                SearchRule::DetLabel { idx: 1 },
            ],
        };

        assert_eq!(rule.get_all_keywords(), vec!["a"]);
    }
}
