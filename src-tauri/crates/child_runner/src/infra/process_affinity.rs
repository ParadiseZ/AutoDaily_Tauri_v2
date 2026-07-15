pub(crate) fn set_process_affinity(cpu_ids: &[usize]) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, SetProcessAffinityMask};

        if cpu_ids.is_empty() {
            return Err("至少需要指定一个 CPU 核心".to_string());
        }

        let mut process_mask = 0usize;
        for &cpu_id in cpu_ids {
            if cpu_id >= 64 {
                return Err(format!("CPU ID {cpu_id} 超出最大值 63"));
            }
            process_mask |= 1usize << cpu_id;
        }

        unsafe {
            if SetProcessAffinityMask(GetCurrentProcess(), process_mask) == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = cpu_ids;
        Ok(())
    }
}
