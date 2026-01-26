use crate::domain::scripts::point::Point;

#[derive(Clone, Debug)]
pub enum Swipe {
    Percent {
        from: Point<f32>,
        to: Point<f32>,
        duration: u64,
    },
    Point{
        from : Point<u16>,
        to: Point<u16>,
        duration: u64
    },
    Label{
        label: String,
        label_idx: u16,
        off: u16,
        off_add: bool
    },
    Txt{
        txt:String,
        off:u16,
        off_add: bool
    },
    Var(String)
}