
use crate::infrastructure::core::core_error::{CoreError, CoreResult};

pub struct CurrentProcess {
    pub name: String,
    pub pid: u32,
    pub ppid: u32,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub status: String,
    pub command: String,
}


#[cfg(target_os = "windows")]
pub fn set_process_affinity(cpu_ids: &[usize]) -> CoreResult<()> {
    use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessAffinityMask};
    // 输入验证
    if cpu_ids.is_empty() {
        return Err(CoreError::AffinityMaskErr{
            e: "At least one CPU must be specified".to_string()
        });
    }

    // 使用 u64 确保跨平台一致性
    let mut process_mask: usize = 0;
    for &cpu_id in cpu_ids {
        if cpu_id >= 64 {
            return Err(CoreError::AffinityMaskErr{
                e: format!("CPU ID {} exceeds maximum 63", cpu_id)
            });
        }
        process_mask |= 1usize << cpu_id;
    }

    unsafe {
        SetProcessAffinityMask(GetCurrentProcess(), process_mask)
            .map_err(|e| CoreError::AffinityMaskErr{
                e: format!("Failed to set process affinity: {}", e)
            })?;

        Ok(())
    }
}


#[cfg(target_os = "linux")]
fn set_cpu_affinity(cores: &[usize]) -> Result<(), Box<dyn std::error::Error>> {
    use sched::{CpuSet, sched_setaffinity, Pid};

    let mut cpu_set = CpuSet::new();
    for &core in cores {
        cpu_set.set(core)?;
    }

    // 设置当前进程的 CPU 亲和性
    sched_setaffinity(Pid::from_raw(0), &cpu_set)?;
    Ok(())
}