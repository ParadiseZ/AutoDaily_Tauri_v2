use image::RgbaImage;
use std::time::Duration;

#[cfg(target_os = "windows")]
mod imp {
    use super::{Duration, RgbaImage};
    use crate::monitor_capture;
    use std::{
        collections::HashMap,
        sync::{Mutex, OnceLock, mpsc::channel},
    };
    use windows::{
        Foundation::TypedEventHandler,
        Graphics::{
            Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession},
            DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
        },
        Win32::{
            Foundation::HWND,
            Graphics::{Direct3D11::ID3D11Texture2D, Dxgi::IDXGIDevice},
            System::WinRT::{
                Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess},
                Graphics::Capture::IGraphicsCaptureItemInterop,
            },
        },
        core::{IInspectable, Interface, Ref, factory},
    };

    static WINDOW_ITEMS: OnceLock<Mutex<HashMap<isize, GraphicsCaptureItem>>> = OnceLock::new();

    fn capture_next_frame(
        frame_pool: Ref<'_, Direct3D11CaptureFramePool>,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<RgbaImage, String> {
        let frame_pool = frame_pool
            .as_ref()
            .ok_or_else(|| "WGC frame pool为空".to_string())?;
        let frame = frame_pool
            .TryGetNextFrame()
            .map_err(|error| format!("WGC TryGetNextFrame失败: {error}"))?;

        let result = (|| {
            let surface = frame
                .Surface()
                .map_err(|error| format!("读取WGC帧表面失败: {error}"))?;
            let access = surface
                .cast::<IDirect3DDxgiInterfaceAccess>()
                .map_err(|error| format!("转换WGC帧表面失败: {error}"))?;
            let texture = unsafe { access.GetInterface::<ID3D11Texture2D>() }
                .map_err(|error| format!("读取WGC D3D11纹理失败: {error}"))?;
            monitor_capture::wgc_texture_region_to_rgba(&texture, 0, y, width, height)
        })();
        let _ = frame.Close();
        result
    }

    fn get_or_create_item(hwnd: HWND) -> Result<GraphicsCaptureItem, String> {
        let key = hwnd.0 as isize;
        let cache = WINDOW_ITEMS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut items = cache
            .lock()
            .map_err(|_| "锁定WGC窗口缓存失败".to_string())?;
        if let Some(item) = items.get(&key) {
            return Ok(item.clone());
        }

        let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
            .map_err(|error| format!("创建WGC窗口工厂失败: {error}"))?;
        let item = unsafe { interop.CreateForWindow::<GraphicsCaptureItem>(hwnd) }
            .map_err(|error| format!("WGC CreateForWindow失败: {error}"))?;
        items.insert(key, item.clone());
        Ok(item)
    }

    pub(crate) fn is_supported() -> bool {
        GraphicsCaptureSession::IsSupported().unwrap_or(false)
    }

    pub(crate) fn capture_window(hwnd: isize, timeout: Duration) -> Result<RgbaImage, String> {
        if !is_supported() {
            return Err("当前系统不支持Windows Graphics Capture".to_string());
        }

        let hwnd = HWND(hwnd as *mut core::ffi::c_void);
        let item = get_or_create_item(hwnd)?;
        let item_size = item
            .Size()
            .map_err(|error| format!("读取WGC窗口尺寸失败: {error}"))?;
        let width = u32::try_from(item_size.Width)
            .map_err(|_| format!("WGC窗口宽度无效: {}", item_size.Width))?;
        let full_height = u32::try_from(item_size.Height)
            .map_err(|_| format!("WGC窗口高度无效: {}", item_size.Height))?;
        if width == 0 || full_height == 0 {
            return Err(format!("WGC截图区域无效: {}x{}", width, full_height));
        }

        let d3d_device = monitor_capture::clone_d3d_device()?;
        let dxgi_device = d3d_device
            .cast::<IDXGIDevice>()
            .map_err(|error| format!("转换WGC DXGI设备失败: {error}"))?;
        let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device) }
            .map_err(|error| format!("创建WGC Direct3D设备失败: {error}"))?;
        let direct3d_device = inspectable
            .cast::<IDirect3DDevice>()
            .map_err(|error| format!("转换WGC Direct3D设备失败: {error}"))?;

        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &direct3d_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            item_size,
        )
        .map_err(|error| format!("创建WGC帧池失败: {error}"))?;
        let (sender, receiver) = channel();
        let token = frame_pool
            .FrameArrived(
                &TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(
                    move |frame_pool, _| {
                        let result = capture_next_frame(frame_pool, 0, width, full_height);
                        let _ = sender.send(result);
                        Ok(())
                    },
                ),
            )
            .map_err(|error| format!("注册WGC帧事件失败: {error}"))?;

        let session = frame_pool
            .CreateCaptureSession(&item)
            .map_err(|error| format!("创建WGC截图会话失败: {error}"))?;
        let _ = session.SetIsBorderRequired(false);
        let _ = session.SetIsCursorCaptureEnabled(false);
        if let Err(error) = session.StartCapture() {
            let _ = frame_pool.RemoveFrameArrived(token);
            let _ = session.Close();
            let _ = frame_pool.Close();
            return Err(format!("启动WGC截图失败: {error}"));
        }

        let result = receiver
            .recv_timeout(timeout)
            .map_err(|error| format!("等待WGC截图帧超时: {error}"))
            .and_then(|result| result);
        let _ = frame_pool.RemoveFrameArrived(token);
        let _ = session.Close();
        let _ = frame_pool.Close();
        result
    }
}

#[cfg(all(test, target_os = "windows"))]
pub(crate) fn is_supported() -> bool {
    imp::is_supported()
}

#[cfg(all(test, not(target_os = "windows")))]
pub(crate) fn is_supported() -> bool {
    false
}

#[cfg(target_os = "windows")]
pub(crate) fn capture_window(hwnd: isize, timeout: Duration) -> Result<RgbaImage, String> {
    imp::capture_window(hwnd, timeout)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn capture_window(_hwnd: isize, _timeout: Duration) -> Result<RgbaImage, String> {
    Err("当前平台不支持Windows Graphics Capture".to_string())
}
