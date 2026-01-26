use crate::domain::scripts::point::Point;
use crate::infrastructure::core::StepId;

/// 点击操作枚举
///
#[derive(Clone, Debug)]
pub enum Click {
    Percent {
        x:f32,
        y:f32
    },
    Point(Point<u16>),
    Label{
        label: String,
        source_var:  String,
        pos_source: StepId
    },
    Txt{
        txt:String,
        source:  String,
        pos_source: StepId
    },
    Var(String)
}

pub struct StepInfo{
    pub id : StepId,
}