use crate::domain::vision::ocr_search::SearchRule;
use crate::infrastructure::core::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize,Clone, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag="type")]
pub enum VisionNode{
    /// 增强视觉搜索：支持 OCR + YOLO + 颜色逻辑
    VisionSearch {
        rule: SearchRule,
        output_var: String, // 存储命中结果的变量名 (Vec<SearchHit>)
    },
    FindObject {
        image_var: String, // 输入图片
        query: String, // 查找内容 (文本 regex 或 模板名称)
        output_var: String, // 输出坐标/区域变量
    },
}