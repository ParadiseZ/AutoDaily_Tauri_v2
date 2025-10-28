use std::io::Cursor;
use std::time::Instant;
use base64::Engine;
use base64::engine::general_purpose;
use image::{DynamicImage, RgbaImage};
use xcap::{Monitor, Window};
use crate::constant::project::SCREENSHOT_DIR;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};

/// 返回格式: (是否找到窗口, 截图的base64编码)
pub fn capture_window(win_name: &str) -> (bool, Option<String>) {
    let start = Instant::now();
    // 获取所有窗口
    let windows = match Window::all() {
        Ok(w) => w,
        Err(e) => {
            let error_msg = format!("获取窗口列表失败: {:?}", e);
            Log::error(&error_msg);
            return (false, None);
        }
    };

    let mut has_found = false;
    let mut target_image: Option<RgbaImage> = None;

    for window in windows {
        // 最小化的窗口不能截屏
        if let Ok(is_min) = window.is_minimized() {
            if is_min {
                continue;
            }
        }

        let title = window.title().unwrap_or_else(|_| "无标题".to_string());
        Log::info(&format!("发现窗口: {}", title));
        // 检查是否是目标窗口
        if title.contains(win_name) {
            has_found = true;
            Log::info(&format!("找到目标窗口: {}", title));

            // 捕获窗口图像
            match window.capture_image() {
                Ok(image) => {
                    target_image = Some(image);
                    Log::info("成功捕获目标窗口图像");
                }
                Err(e) => {
                    Log::error(&format!("捕获窗口图像失败: {:?}", e));
                }
            }

            // 找到并截图后退出循环
            break;
        }
    }

    let elapsed = start.elapsed();
    Log::info(&format!("截图操作耗时: {:?}", elapsed));

    // 将图像转换为base64编码
    let base64_image = if let Some(image) = target_image {
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
    } else {
        None
    };

    (has_found, base64_image)
}

pub fn window_cap_test(method: &str, _device: &str, win_name: &str) -> String {
    Log::fn_begin("window_cap_test");
    Log::info_with_fields(
        "窗口捕获测试",
        vec![
            ("方法", method.to_string()),
            ("窗口名称", win_name.to_string()),
        ],
    );
    Log::fn_begin("capture_window");
    Log::info_with_tag("截图", &format!("开始查找并截取窗口: {}", win_name));

    let (has_found, base64_image) = capture_window(win_name);

    let result = if has_found {
        if let Some(base64_str) = base64_image {
            format!("ok|{}", base64_str)
        } else {
            "ok|no_image".to_string()
        }
    } else {
        "not found".to_string()
    };

    Log::fn_end("window_cap_test");
    result
}

pub fn capture() -> AppResult<()> {
    capture_screen()
}


/// 实际执行截图的函数
pub(crate) fn capture_screen() -> AppResult<()> {
    let monitor_all = Monitor::all()
        .map_err(|e| AppError::CaptureError(format!("截图失败：{}",e)))?;

    let _region_width = 400u32;
    let _region_height = 300u32;

    for monitor in monitor_all {
        // Calculate center of the monitor for region capture
        let _x = 0;
        let _y = 0;

        // Capture the region
        let start = Instant::now();
        let image = monitor.capture_image()
            .map_err(|e|AppError::CaptureError(format!("截图失败:{}",e)) )?;

        // Get monitor name for the filename
        let monitor_name = monitor
            .name()
            .unwrap_or_else(|_| format!("unknown-{}", monitor.id().unwrap_or(0)));
        let is_primary = monitor.is_primary().unwrap_or(false);
        let primary_indicator = if is_primary { "-primary" } else { "" };

        Log::info(
            &format!("Monitor '{}'{}: Time to capture region of size {}x{}: {:?}",
                     monitor_name,
                     primary_indicator,
                     image.width(),
                     image.height(),
                     start.elapsed())
        );

        // Save the image
        /*let filename = format!(
            "target/monitors/monitor-{}{}-region.png",
            normalized(monitor_name),
            primary_indicator
        );*/
        let filename = format!(
            "{}/{}.png",
            SCREENSHOT_DIR,
            chrono::Local::now().format("%Y%m%d%H%M%S").to_string()
        );

        image.save(&filename).unwrap();
        Log::info(&format!("保存图片:{:?}", filename));
    }
    Ok(())
}

// fn normalized(filename: String) -> String {
//     filename.replace(['|', '\\', ':', '/'], "")
// }

/// 获取系统物理CPU核心数
pub fn get_cpu_cores() -> AppResult<u32> {
    let cores = num_cpus::get_physical() as u32;
    if cores == 0 {
        Log::error(&"获取系统CPU核心数失败".to_string());
        return Err(AppError::SystemError("获取系统CPU核心数失败！".to_string()));
    }
    Ok(cores)
}
