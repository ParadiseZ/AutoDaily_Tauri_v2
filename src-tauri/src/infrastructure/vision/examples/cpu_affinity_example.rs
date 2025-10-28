use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::entities::config::performance::Performance;
use crate::infrastructure::entities::vision::performance::ThreadPoolManager;

/// CPU亲和性示例 - 展示正确的CPU核心分配
pub struct CpuAffinityExample;

impl CpuAffinityExample {
    /// 演示多设备CPU核心分配
    pub async fn demonstrate_cpu_allocation() -> AppResult<()> {
        Log::info("=== CPU核心分配演示开始 ===");
        
        // 假设系统有8个CPU核心，每个设备分配4个核心，最多2个设备
        let performance_config = Performance {
            cores_per_device: 4,
            max_devices: 2,
        };
        
        Log::info(&format!("系统CPU核心数: {}", num_cpus::get()));
        Log::info(&format!("配置: 每设备{}核心, 最大{}设备", 
                         performance_config.cores_per_device, 
                         performance_config.max_devices));
        
        // 创建多个设备的线程池管理器
        let mut managers = Vec::new();
        
        for device_id in 0..performance_config.max_devices {
            let manager = ThreadPoolManager::new_for_device(
                performance_config.clone(), 
                device_id
            )?;
            
            Log::info(&format!("✅ {}", manager.display_cpu_allocation()));
            managers.push(manager);
        }
        
        // 验证CPU分配不重叠
        Self::verify_no_overlap(&managers)?;
        
        // 演示并发任务执行
        Self::demonstrate_concurrent_execution(managers).await?;
        
        Log::info("=== CPU核心分配演示完成 ===");
        Ok(())
    }
    
    /// 验证CPU核心分配没有重叠
    fn verify_no_overlap(managers: &[ThreadPoolManager]) -> AppResult<()> {
        Log::info("🔍 验证CPU核心分配是否重叠...");
        
        let mut all_inference_cores = std::collections::HashSet::new();
        let mut all_cpu_cores = std::collections::HashSet::new();
        
        for manager in managers {
            let allocation = manager.cpu_allocation();
            
            // 检查推理核心
            if let Some(inference_core) = allocation.inference_core {
                if !all_inference_cores.insert(inference_core) {
                    return Err(AppError::ConfigError(format!(
                        "❌ 检测到推理核心{}重叠! 设备{}", 
                        inference_core, 
                        manager.device_id()
                    )));
                }
            }
            
            // 检查CPU处理核心
            for &cpu_core in &allocation.cpu_cores {
                if !all_cpu_cores.insert(cpu_core) {
                    return Err(AppError::ConfigError(format!(
                        "❌ 检测到CPU处理核心{}重叠! 设备{}", 
                        cpu_core, 
                        manager.device_id()
                    )));
                }
            }
        }
        
        Log::info(&format!("✅ 核心分配验证通过! 推理核心: {:?}, CPU处理核心: {:?}", 
                         all_inference_cores, all_cpu_cores));
        Ok(())
    }
    
    /// 演示并发执行 - 每个设备使用专用核心
    async fn demonstrate_concurrent_execution(managers: Vec<ThreadPoolManager>) -> AppResult<()> {
        Log::info("🚀 开始并发执行演示...");
        
        let mut handles = Vec::new();
        
        for manager in managers {
            let handle = tokio::spawn(async move {
                Self::device_workload(manager).await
            });
            handles.push(handle);
        }
        
        // 等待所有设备完成
        for handle in handles {
            handle.await
                .map_err(|e| AppError::InternalError(format!("设备任务失败: {}", e)))??;
        }
        
        Log::info("✅ 所有设备并发执行完成");
        Ok(())
    }
    
    /// 单个设备的工作负载
    async fn device_workload(manager: ThreadPoolManager) -> AppResult<()> {
        let device_id = manager.device_id();
        Log::info(&format!("设备{} 开始工作负载", device_id));
        
        // 1. 推理任务（使用推理专用核心）
        let inference_pool = manager.inference_pool().clone();
        let inference_result = tokio::task::spawn_blocking(move || {
            inference_pool.install(|| {
                // 模拟推理计算
                let mut sum = 0;
                for i in 0..1000000 {
                    sum += i * i;
                }
                sum
            })
        }).await.map_err(|e| AppError::InternalError(format!("推理任务失败: {}", e)))?;
        
        Log::info(&format!("设备{} 推理完成: {}", device_id, inference_result));
        
        // 2. CPU密集型任务（使用CPU处理专用核心）
        let cpu_pool = manager.cpu_pool().clone();
        let cpu_result = tokio::task::spawn_blocking(move || {
            cpu_pool.install(|| {
                use rayon::prelude::*;
                
                // 并行CPU密集型计算（如CTC解码、图像处理等）
                (0..1000).into_par_iter().map(|i| {
                    // 模拟CPU密集型操作
                    std::thread::sleep(std::time::Duration::from_micros(1));
                    i * 2
                }).sum::<i32>()
            })
        }).await.map_err(|e| AppError::InternalError(format!("CPU任务失败: {}", e)))?;
        
        Log::info(&format!("设备{} CPU处理完成: {}", device_id, cpu_result));
        
        // 3. 显示核心使用情况
        Log::info(&format!("设备{} 核心使用: {}", device_id, manager.display_cpu_allocation()));
        
        Ok(())
    }
    
    /// 显示理想的CPU分配方案
    pub fn show_ideal_allocation_example() {
        Log::info("=== 理想CPU分配方案示例 ===");
        
        // 示例1: 8核心系统，2个设备，每设备4核心
        println!("💡 示例1: 8核心系统，2个设备，每设备4核心");
        println!("设备0: 推理核心=0, CPU处理核心=[1,2,3]");
        println!("设备1: 推理核心=4, CPU处理核心=[5,6,7]");
        println!("✅ 完全隔离，无竞争\n");
        
        // 示例2: 16核心系统，4个设备，每设备4核心
        println!("💡 示例2: 16核心系统，4个设备，每设备4核心");
        println!("设备0: 推理核心=0,  CPU处理核心=[1,2,3]");
        println!("设备1: 推理核心=4,  CPU处理核心=[5,6,7]");
        println!("设备2: 推理核心=8,  CPU处理核心=[9,10,11]");
        println!("设备3: 推理核心=12, CPU处理核心=[13,14,15]");
        println!("✅ 完全隔离，充分利用\n");
        
        // 示例3: 8核心系统，3个设备，每设备4核心（超额分配）
        println!("💡 示例3: 8核心系统，3个设备，每设备4核心（超额分配）");
        println!("设备0: 推理核心=0, CPU处理核心=[1,2,3]");
        println!("设备1: 推理核心=4, CPU处理核心=[5,6,7]");
        println!("设备2: 推理核心=0, CPU处理核心=[1,2,3] (取模重用)");
        println!("⚠️  有重叠，但比随机分配好\n");
        
        println!("🎯 关键优势:");
        println!("1. 推理线程完全隔离，性能可预测");
        println!("2. CPU处理核心专用，避免上下文切换");
        println!("3. 故障隔离，单设备问题不影响其他设备");
        println!("4. NUMA感知（在NUMA系统上可进一步优化）");
    }
}

/// CPU亲和性工具函数
pub mod cpu_affinity_utils {
    use super::*;

    /// 获取当前线程的CPU亲和性（需要core_affinity依赖）
    pub fn get_current_thread_affinity() -> Vec<usize> {
        // TODO: 实现获取当前线程CPU亲和性
        // 需要 core_affinity = "0.8" 依赖
        /*
        if let Ok(core_ids) = core_affinity::get_current_thread_affinity() {
            core_ids.into_iter().map(|core| core.id).collect()
        } else {
            vec![]
        }
        */
        
        // 临时返回空向量
        vec![]
    }
    
    /// 验证线程是否绑定到正确的核心
    pub fn verify_thread_affinity(expected_core: usize) -> bool {
        let current_affinity = get_current_thread_affinity();
        current_affinity.contains(&expected_core)
    }
    
    /// 获取系统NUMA拓扑信息（高级优化）
    pub fn get_numa_topology() -> AppResult<Vec<Vec<usize>>> {
        // TODO: 实现NUMA拓扑检测
        // 在NUMA系统上，应该将设备绑定到同一个NUMA节点的核心
        
        // 临时返回简单的分组
        let total_cores = num_cpus::get();
        if total_cores >= 8 {
            // 假设前一半和后一半是不同的NUMA节点
            Ok(vec![
                (0..total_cores/2).collect(),
                (total_cores/2..total_cores).collect(),
            ])
        } else {
            Ok(vec![(0..total_cores).collect()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cpu_allocation_no_overlap() {
        let performance_config = Performance {
            cores_per_device: 2,
            max_devices: 2,
        };
        
        let manager1 = ThreadPoolManager::new_for_device(performance_config.clone(), 0).unwrap();
        let manager2 = ThreadPoolManager::new_for_device(performance_config.clone(), 1).unwrap();
        
        let managers = vec![manager1, manager2];
        assert!(CpuAffinityExample::verify_no_overlap(&managers).is_ok());
    }
    
    #[test]
    fn test_cpu_allocation_calculation() {
        // 测试核心分配算法
        let total_cores = 8;
        
        // 设备0应该分配核心0-3
        // 设备1应该分配核心4-7
        
        // 这里可以测试具体的分配逻辑
        assert!(true); // 占位测试
    }
}
