use crate::infrastructure::logging::log_trait::Log;
use base64::engine::general_purpose;
use base64::Engine;
use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use std::io::Cursor;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncWriteExt;

lazy_static!(
    pub static ref CAP_WAY: Arc<RwLock< Option<Arc<dyn CapHandler>> >> = Arc::new(RwLock::new( None ) );
);
//static CAP_WAY : OnceCell<Arc<RwLock< Option< Arc<dyn CapHandler> > >>> = OnceCell::new();

pub fn set_cap_way<T : CapHandler>(cap_way: T)->bool{
    // 确保容器（RwLock）只被初始化一次
    //let cap_arc = CAP_WAY.get_or_init(|| Arc::new(RwLock::new(None)));
    // 获取写锁，更新内部数据（动态替换）
    let mut write_guard = if let Ok(w) = CAP_WAY.clone().write(){
        w
    }else {
        Log::error("切换截图方式失败：获取锁失败");
        return false;
    };
    *write_guard = Some(Arc::new(cap_way));
    true
}

pub fn get_capture()-> Option<RgbaImage>{
    if let Ok(read_guard) = CAP_WAY.clone().read(){
        read_guard.as_ref().map(|handler| handler.capture())?
    }else {
        Log::error("截图失败！未初始化截图方式，或获取截图方式锁失败！");
        None
    }
}

pub fn get_base64_capture()-> Option<String>{
    if let Some(image) = get_capture() {
        // 使用Cursor包装Vec<u8>，使其同时实现Write和Seek特征
        let mut cursor = Cursor::new(Vec::new());
        match DynamicImage::ImageRgba8(image).write_to(&mut cursor, image::ImageFormat::Png) {
            Ok(_) => {
                let buffer = cursor.into_inner();
                let base64_string = general_purpose::STANDARD.encode(&buffer);
                Log::info(&format!(
                    "图像已转换为base64编码，大小约为: {} KB",
                    base64_string.len() / 1024
                ));
                Some(base64_string)
            }
            Err(e) => {
                Log::error(&format!("图像转换为base64失败: {:?}", e));
                None
            }
        }
    }else { 
        None
    }
}

pub trait CapHandler: Send + Sync{

    fn capture(&self) -> Option<RgbaImage>;
}