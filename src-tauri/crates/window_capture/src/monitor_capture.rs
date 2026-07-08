use image::RgbaImage;

#[cfg(target_os = "windows")]
mod imp {
    use image::RgbaImage;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};
    use windows::{
        core::Interface,
        Win32::{
            Foundation::POINT,
            Graphics::{
                Direct3D::D3D_DRIVER_TYPE_HARDWARE,
                Direct3D11::{
                    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Resource,
                    ID3D11Texture2D, D3D11_BOX, D3D11_CPU_ACCESS_READ,
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ,
                    D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
                },
                Dxgi::{
                    IDXGIDevice, IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource,
                    DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO,
                },
                Gdi::{MonitorFromPoint, MONITOR_DEFAULTTONEAREST},
            },
        },
    };
    use xcap::Monitor;

    static D3D_DEVICE: OnceLock<Result<ID3D11Device, String>> = OnceLock::new();
    static D3D_CONTEXT: OnceLock<Result<ID3D11DeviceContext, String>> = OnceLock::new();
    static DXGI_RUNTIMES: OnceLock<Mutex<HashMap<isize, Arc<DxgiRuntime>>>> = OnceLock::new();

    struct StagingTexture {
        width: u32,
        height: u32,
        texture: ID3D11Texture2D,
    }

    struct DxgiRuntime {
        duplication: IDXGIOutputDuplication,
        staging: Mutex<Option<StagingTexture>>,
    }

    fn bgra_to_rgba(mut buffer: Vec<u8>) -> Vec<u8> {
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        buffer
    }

    fn d3d_device() -> Result<&'static ID3D11Device, String> {
        match D3D_DEVICE.get_or_init(|| unsafe {
            let mut d3d_device = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                Default::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )
            .map_err(|error| format!("D3D11CreateDevice failed: {error}"))?;

            d3d_device.ok_or_else(|| "D3D11CreateDevice returned null device".to_string())
        }) {
            Ok(device) => Ok(device),
            Err(error) => Err(error.clone()),
        }
    }

    fn d3d_context() -> Result<&'static ID3D11DeviceContext, String> {
        match D3D_CONTEXT.get_or_init(|| unsafe {
            d3d_device()?
                .GetImmediateContext()
                .map_err(|error| format!("GetImmediateContext failed: {error}"))
        }) {
            Ok(context) => Ok(context),
            Err(error) => Err(error.clone()),
        }
    }

    fn get_or_create_staging_texture(
        runtime: &DxgiRuntime,
        width: u32,
        height: u32,
        source_desc: &D3D11_TEXTURE2D_DESC,
    ) -> Result<ID3D11Texture2D, String> {
        let mut staging = runtime
            .staging
            .lock()
            .map_err(|_| "锁定 staging 纹理失败".to_string())?;
        if let Some(cache) = staging.as_ref() {
            if cache.width == width && cache.height == height {
                return Ok(cache.texture.clone());
            }
        }

        let device = d3d_device()?;
        let mut staging_desc = *source_desc;
        staging_desc.Width = width;
        staging_desc.Height = height;
        staging_desc.BindFlags = 0;
        staging_desc.MiscFlags = 0;
        staging_desc.Usage = D3D11_USAGE_STAGING;
        staging_desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;

        let mut texture = None;
        unsafe {
            device
                .CreateTexture2D(&staging_desc, None, Some(&mut texture))
                .map_err(|error| format!("CreateTexture2D failed: {error}"))?;
        }
        let texture = texture.ok_or_else(|| "CreateTexture2D returned null".to_string())?;
        *staging = Some(StagingTexture {
            width,
            height,
            texture: texture.clone(),
        });
        Ok(texture)
    }

    fn texture_to_rgba_image(
        runtime: &DxgiRuntime,
        source_texture: &ID3D11Texture2D,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<RgbaImage, String> {
        unsafe {
            let context = d3d_context()?;

            let mut src_desc = D3D11_TEXTURE2D_DESC::default();
            source_texture.GetDesc(&mut src_desc);
            if x + width > src_desc.Width || y + height > src_desc.Height {
                return Err(format!(
                    "ROI out of bounds: ({x}, {y}, {width}, {height}) > {}x{}",
                    src_desc.Width, src_desc.Height
                ));
            }
            let staging = get_or_create_staging_texture(runtime, width, height, &src_desc)?;

            let region = D3D11_BOX {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
                front: 0,
                back: 1,
            };

            context.CopySubresourceRegion(
                Some(
                    &staging
                        .cast::<ID3D11Resource>()
                        .map_err(|error| error.to_string())?,
                ),
                0,
                0,
                0,
                0,
                Some(
                    &source_texture
                        .cast::<ID3D11Resource>()
                        .map_err(|error| error.to_string())?,
                ),
                0,
                Some(&region),
            );

            let resource = staging
                .cast::<ID3D11Resource>()
                .map_err(|error| error.to_string())?;
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            context
                .Map(Some(&resource), 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                .map_err(|error| format!("Map staging texture failed: {error}"))?;

            let mut bgra = vec![0u8; (width * height * 4) as usize];
            let src_ptr = mapped.pData as *const u8;
            for row in 0..height {
                let src_offset = (row * mapped.RowPitch) as usize;
                let dst_offset = (row * width * 4) as usize;
                let src_slice =
                    std::slice::from_raw_parts(src_ptr.add(src_offset), (width * 4) as usize);
                bgra[dst_offset..dst_offset + (width * 4) as usize].copy_from_slice(src_slice);
            }

            context.Unmap(Some(&resource), 0);
            RgbaImage::from_raw(width, height, bgra_to_rgba(bgra))
                .ok_or_else(|| "RgbaImage::from_raw failed".to_string())
        }
    }

    fn create_runtime(
        h_monitor: windows::Win32::Graphics::Gdi::HMONITOR,
    ) -> Result<Arc<DxgiRuntime>, String> {
        unsafe {
            let device = d3d_device()?;
            let dxgi_device: IDXGIDevice = device
                .cast()
                .map_err(|error| format!("Cast D3D device to IDXGIDevice failed: {error}"))?;
            let adapter = dxgi_device
                .GetAdapter()
                .map_err(|error| format!("GetAdapter failed: {error}"))?;

            let mut output_index = 0;
            loop {
                let output = match adapter.EnumOutputs(output_index) {
                    Ok(output) => output,
                    Err(error) => return Err(format!("EnumOutputs failed: {error}")),
                };
                output_index += 1;

                let output_desc = output
                    .GetDesc()
                    .map_err(|error| format!("GetDesc failed: {error}"))?;
                if output_desc.Monitor != h_monitor {
                    continue;
                }

                let output1: IDXGIOutput1 = output
                    .cast()
                    .map_err(|error| format!("Cast output to IDXGIOutput1 failed: {error}"))?;
                let duplication = output1
                    .DuplicateOutput(&dxgi_device)
                    .map_err(|error| format!("DuplicateOutput failed: {error}"))?;

                return Ok(Arc::new(DxgiRuntime {
                    duplication,
                    staging: Mutex::new(None),
                }));
            }
        }
    }

    fn get_runtime(
        key: isize,
        h_monitor: windows::Win32::Graphics::Gdi::HMONITOR,
    ) -> Result<Arc<DxgiRuntime>, String> {
        let cache = DXGI_RUNTIMES.get_or_init(|| Mutex::new(HashMap::new()));
        let mut runtimes = cache
            .lock()
            .map_err(|_| "锁定 DXGI_RUNTIMES 失败".to_string())?;
        if let Some(runtime) = runtimes.get(&key) {
            return Ok(runtime.clone());
        }

        let runtime = create_runtime(h_monitor)?;
        runtimes.insert(key, runtime.clone());
        Ok(runtime)
    }

    fn drop_runtime(key: isize) {
        if let Some(cache) = DXGI_RUNTIMES.get() {
            if let Ok(mut runtimes) = cache.lock() {
                runtimes.remove(&key);
            }
        }
    }

    fn acquire_frame(
        runtime: &DxgiRuntime,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        wait_timeout_ms: u32,
    ) -> Result<RgbaImage, String> {
        unsafe {
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource: Option<IDXGIResource> = None;
            runtime
                .duplication
                .AcquireNextFrame(wait_timeout_ms, &mut frame_info, &mut resource)
                .map_err(|error| {
                    if error.code() == DXGI_ERROR_WAIT_TIMEOUT {
                        format!("DXGI AcquireNextFrame timeout: {}ms 内没有等到新的桌面帧", wait_timeout_ms)
                    } else {
                        format!("DXGI AcquireNextFrame failed: {error}")
                    }
                })?;

            let result = (|| {
                if frame_info.LastPresentTime == 0 {
                    return Err("DXGI 没有新帧: 当前没有可取的桌面更新".to_string());
                }

                let resource = resource
                    .ok_or_else(|| "AcquireNextFrame returned null resource".to_string())?;
                let source_texture = resource
                    .cast::<ID3D11Texture2D>()
                    .map_err(|error| format!("Cast resource to ID3D11Texture2D failed: {error}"))?;
                texture_to_rgba_image(runtime, &source_texture, x, y, width, height)
            })();

            let _ = runtime.duplication.ReleaseFrame();
            result
        }
    }

    fn capture_window_region_once(
        key: isize,
        h_monitor: windows::Win32::Graphics::Gdi::HMONITOR,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        wait_timeout_ms: u32,
    ) -> Result<RgbaImage, String> {
        let runtime = get_runtime(key, h_monitor)?;
        match acquire_frame(&runtime, x, y, width, height, wait_timeout_ms) {
            Ok(image) => Ok(image),
            Err(error) => {
                if error.contains("DXGI AcquireNextFrame failed") {
                    drop_runtime(key);
                }
                Err(error)
            }
        }
    }

    pub fn is_supported() -> bool {
        d3d_device().is_ok()
    }

    pub fn capture_window_region(
        window_x: i32,
        window_y: i32,
        window_width: u32,
        window_height: u32,
        wait_timeout_ms: u32,
    ) -> Result<RgbaImage, String> {
        if window_width == 0 || window_height == 0 {
            return Err("窗口尺寸无效".to_string());
        }

        let center_x = window_x + (window_width / 2) as i32;
        let center_y = window_y + (window_height / 2) as i32;
        let monitor = Monitor::from_point(center_x, center_y)
            .map_err(|error| format!("Monitor::from_point failed: {error}"))?;
        let monitor_x = monitor
            .x()
            .map_err(|error| format!("monitor.x failed: {error}"))?;
        let monitor_y = monitor
            .y()
            .map_err(|error| format!("monitor.y failed: {error}"))?;
        let monitor_width = monitor
            .width()
            .map_err(|error| format!("monitor.width failed: {error}"))?
            as i32;
        let monitor_height = monitor
            .height()
            .map_err(|error| format!("monitor.height failed: {error}"))?
            as i32;

        let left = window_x.max(monitor_x);
        let top = window_y.max(monitor_y);
        let right = (window_x + window_width as i32).min(monitor_x + monitor_width);
        let bottom = (window_y + window_height as i32).min(monitor_y + monitor_height);
        if right <= left || bottom <= top {
            return Err("窗口区域不在目标显示器可视范围内".to_string());
        }

        let capture_x = (left - monitor_x) as u32;
        let capture_y = (top - monitor_y) as u32;
        let capture_width = (right - left) as u32;
        let capture_height = (bottom - top) as u32;

        let h_monitor = unsafe {
            MonitorFromPoint(
                POINT {
                    x: center_x,
                    y: center_y,
                },
                MONITOR_DEFAULTTONEAREST,
            )
        };
        if h_monitor.0.is_null() {
            return Err("MonitorFromPoint returned null monitor".to_string());
        }
        let key = h_monitor.0 as isize;

        capture_window_region_once(
            key,
            h_monitor,
            capture_x,
            capture_y,
            capture_width,
            capture_height,
            wait_timeout_ms,
        )
        .or_else(|_| {
            capture_window_region_once(
                key,
                h_monitor,
                capture_x,
                capture_y,
                capture_width,
                capture_height,
                wait_timeout_ms,
            )
        })
    }
}

#[cfg(target_os = "windows")]
pub fn is_supported() -> bool {
    imp::is_supported()
}

#[cfg(not(target_os = "windows"))]
pub fn is_supported() -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn capture_window_region(
    window_x: i32,
    window_y: i32,
    window_width: u32,
    window_height: u32,
    wait_timeout_ms: u32,
) -> Result<RgbaImage, String> {
    imp::capture_window_region(
        window_x,
        window_y,
        window_width,
        window_height,
        wait_timeout_ms,
    )
}

#[cfg(not(target_os = "windows"))]
pub fn capture_window_region(
    _window_x: i32,
    _window_y: i32,
    _window_width: u32,
    _window_height: u32,
    _wait_timeout_ms: u32,
) -> Result<RgbaImage, String> {
    Err("当前平台不支持 DXGI 监视器区域采集".to_string())
}
