use serde::{Deserialize, Serialize};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CaptureBasis{
    Window,
    Adb
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ImageCompression{
    WindowOriginal,
    AdbOriginal,
    ScreenCap
}

pub trait ImgComp{
    fn img_comp(){}
}